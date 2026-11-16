// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2026  Timofei Molokov



use std::{collections::HashMap, time::Duration};

use fyrox_core::futures::{SinkExt, StreamExt};
use matchbox_socket::{
    PeerId,
    PeerState,
    RtcIceServerConfig,
    WebRtcSocket
};
use tokio::{runtime::Handle, sync::mpsc};
use tokio_tungstenite::{connect_async, tungstenite};


pub type Packet = Box<[u8]>;


/// Identifies one connection of the `NetSystem`. Several independent connections can exist at once.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ConnectionHandle(u32);


pub enum ConnectionConfig
{
    /// A WebSocket connection (used for the matchmaking server).
    WebSocket {
        url: String,
        connect_timeout: Duration,
    },
    /// A WebRTC connection through Matchbox (used for the game server).
    /// One reliable channel (0) and one unreliable channel (1); the first connected peer is the server.
    WebRtc {
        url: String,
        ice_servers: Vec<String>,
        turn_server_username: Option<String>,
        turn_server_credential: Option<String>,
    },
}


#[derive(Clone, Debug, PartialEq)]
pub enum ConnectionState
{
    Connecting,
    Connected,
    Disconnected(DisconnectReason),
}


#[derive(Clone, Debug, PartialEq)]
pub enum DisconnectReason
{
    Timeout,
    ClosedByRemote,
    Lost(String),
    /// The handle doesn't exist (closed and forgotten, or never created).
    UnknownConnection,
}


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reliability
{
    Reliable,
    Unreliable,
}


/// The network transport, part of the Engine Shared Systems.
///
/// It creates connections, maintains them, reports their state, and sends/receives raw packets.
/// It knows nothing about the game protocol: game-session logic lives in actors (`SessionController`),
/// which serialize/deserialize protocol messages themselves (`to_packet` / `from_packet`).
pub struct NetSystem
{
    runtime: Handle,
    connections: HashMap<ConnectionHandle, Connection>,
    next_handle: u32,
    /// Always empty; lets `receive` return an empty `Drain` for unknown handles without allocating.
    empty_receive_buffer: Vec<Packet>,
}


struct Connection
{
    state: ConnectionState,
    received: Vec<Packet>,
    transport: Transport,
}


enum Transport
{
    WebSocket {
        events: mpsc::UnboundedReceiver<WebSocketEvent>,
        outgoing: Option<mpsc::UnboundedSender<Packet>>,
    },
    WebRtc {
        socket: WebRtcSocket,
        server: Option<PeerId>,
    },
}


enum WebSocketEvent
{
    Connected,
    Packet(Packet),
    Closed(DisconnectReason),
}


impl NetSystem
{
    pub fn new(runtime: Handle) -> Self
    {
        NetSystem {
            runtime,
            connections: HashMap::new(),
            next_handle: 1,
            empty_receive_buffer: Vec::new(),
        }
    }

    pub fn connect(&mut self, config: ConnectionConfig) -> ConnectionHandle
    {
        let handle = ConnectionHandle(self.next_handle);
        self.next_handle += 1;

        let transport = match config
        {
            ConnectionConfig::WebSocket { url, connect_timeout } =>
            {
                self.start_websocket(url, connect_timeout)
            }

            ConnectionConfig::WebRtc { url, ice_servers, turn_server_username, turn_server_credential } =>
            {
                let (socket, socket_future) = matchbox_socket::WebRtcSocketBuilder::new(url)
                    .reconnect_attempts(Some(3))
                    .signaling_keep_alive_interval(Some(Duration::from_secs(1)))
                    .ice_server(RtcIceServerConfig {
                        urls: ice_servers,
                        username: turn_server_username,
                        credential: turn_server_credential,
                    })
                    .add_reliable_channel()
                    .add_unreliable_channel()
                    .build();

                self.runtime.spawn(socket_future);

                Transport::WebRtc { socket, server: None }
            }
        };

        self.connections.insert(handle, Connection {
            state: ConnectionState::Connecting,
            received: Vec::new(),
            transport,
        });

        handle
    }

    pub fn state(&self, handle: ConnectionHandle) -> ConnectionState
    {
        match self.connections.get(&handle)
        {
            Some(connection) => connection.state.clone(),
            None => ConnectionState::Disconnected(DisconnectReason::UnknownConnection),
        }
    }

    /// Sends a packet. Sends are immediate (no buffering until `flush`), for minimal latency.
    pub fn send(&mut self, handle: ConnectionHandle, packet: Packet, reliability: Reliability)
    {
        let Some(connection) = self.connections.get_mut(&handle) else {
            return;
        };

        if connection.state != ConnectionState::Connected
        {
            return;
        }

        match &mut connection.transport
        {
            Transport::WebSocket { outgoing, .. } =>
            {
                if let Some(outgoing) = outgoing
                {
                    let _ = outgoing.send(packet);
                }
            }

            Transport::WebRtc { socket, server } =>
            {
                if socket.any_channel_closed() {return;}

                if let Some(server) = server
                {
                    let channel = match reliability
                    {
                        Reliability::Reliable => 0,
                        Reliability::Unreliable => 1,
                    };

                    socket.channel_mut(channel).send(packet, *server);
                }
            }
        }
    }

    /// Takes the packets received on this connection since the previous call, in arrival order.
    /// `drain` keeps the receive buffer's capacity (no reallocation); packets not taken are dropped
    /// when the iterator is dropped.
    pub fn receive(&mut self, handle: ConnectionHandle) -> std::vec::Drain<'_, Packet>
    {
        match self.connections.get_mut(&handle)
        {
            Some(connection) => connection.received.drain(..),
            None => self.empty_receive_buffer.drain(..),
        }
    }

    /// WebRTC only: this client's id in the Matchbox network (the game uses it as the main player's actor id).
    pub fn local_peer_id(&mut self, handle: ConnectionHandle) -> Option<u128>
    {
        match self.connections.get_mut(&handle)
        {
            Some(Connection { transport: Transport::WebRtc { socket, .. }, .. }) =>
            {
                socket.id().map(|id| id.0.as_u128())
            }
            _ => None,
        }
    }

    /// Closes the connection and forgets the handle.
    pub fn close(&mut self, handle: ConnectionHandle)
    {
        // Dropping the transport closes it: the WebSocket writer task ends when its sender is dropped,
        // and dropping the WebRtcSocket closes its channels.
        self.connections.remove(&handle);
    }

    /// Pre-simulate: polls all connections, updates their states and receive buffers.
    pub fn tick(&mut self)
    {
        for (_, connection) in self.connections.iter_mut()
        {
            poll_connection(connection);
        }
    }

    /// Post-simulate. Sends are immediate, so there is nothing to flush now. This is the place
    /// for future batching/packing (e.g. a UDP transport).
    pub fn flush(&mut self) {}


    fn start_websocket(&self, url: String, connect_timeout: Duration) -> Transport
    {
        let (event_sender, event_receiver) = mpsc::unbounded_channel::<WebSocketEvent>();
        let (outgoing_sender, mut outgoing_receiver) = mpsc::unbounded_channel::<Packet>();

        let runtime = self.runtime.clone();

        self.runtime.spawn(async move {
            let ws_stream = match tokio::time::timeout(connect_timeout, connect_async(url)).await
            {
                Err(_) =>
                {
                    let _ = event_sender.send(WebSocketEvent::Closed(DisconnectReason::Timeout));
                    return;
                }
                Ok(Err(e)) =>
                {
                    let _ = event_sender.send(WebSocketEvent::Closed(DisconnectReason::Lost(e.to_string())));
                    return;
                }
                Ok(Ok((ws_stream, _))) => ws_stream,
            };

            let _ = event_sender.send(WebSocketEvent::Connected);

            let (mut sink, mut stream) = ws_stream.split();

            let writer_event_sender = event_sender.clone();

            runtime.spawn(async move {
                while let Some(packet) = outgoing_receiver.recv().await
                {
                    if let Err(e) = sink.send(tungstenite::Message::binary(packet)).await
                    {
                        let _ = writer_event_sender.send(WebSocketEvent::Closed(DisconnectReason::Lost(e.to_string())));
                        return;
                    }
                }

                // the connection was closed locally (the sender was dropped)
                let _ = sink.close().await;
            });

            loop
            {
                match stream.next().await
                {
                    Some(Ok(message)) =>
                    {
                        match message
                        {
                            tungstenite::Message::Binary(data) =>
                            {
                                // no copy when the received buffer isn't shared
                                let _ = event_sender.send(WebSocketEvent::Packet(Vec::<u8>::from(data).into_boxed_slice()));
                            }
                            tungstenite::Message::Text(text) =>
                            {
                                let _ = event_sender.send(WebSocketEvent::Packet(Vec::<u8>::from(tungstenite::Bytes::from(text)).into_boxed_slice()));
                            }
                            tungstenite::Message::Close(_) =>
                            {
                                let _ = event_sender.send(WebSocketEvent::Closed(DisconnectReason::ClosedByRemote));
                                return;
                            }
                            _ => {}
                        }
                    }
                    Some(Err(e)) =>
                    {
                        let _ = event_sender.send(WebSocketEvent::Closed(DisconnectReason::Lost(e.to_string())));
                        return;
                    }
                    None =>
                    {
                        let _ = event_sender.send(WebSocketEvent::Closed(DisconnectReason::ClosedByRemote));
                        return;
                    }
                }
            }
        });

        Transport::WebSocket {
            events: event_receiver,
            outgoing: Some(outgoing_sender),
        }
    }
}


fn poll_connection(connection: &mut Connection)
{
    if let ConnectionState::Disconnected(_) = connection.state
    {
        return;
    }

    match &mut connection.transport
    {
        Transport::WebSocket { events, outgoing } =>
        {
            while let Ok(event) = events.try_recv()
            {
                match event
                {
                    WebSocketEvent::Connected =>
                    {
                        connection.state = ConnectionState::Connected;
                    }
                    WebSocketEvent::Packet(packet) =>
                    {
                        connection.received.push(packet);
                    }
                    WebSocketEvent::Closed(reason) =>
                    {
                        connection.state = ConnectionState::Disconnected(reason);
                        *outgoing = None;
                        return;
                    }
                }
            }
        }

        Transport::WebRtc { socket, server } =>
        {
            if socket.any_channel_closed()
            {
                connection.state = ConnectionState::Disconnected(DisconnectReason::ClosedByRemote);
                return;
            }

            if let Ok(peers) = socket.try_update_peers()
            {
                for (peer_id, peer_state) in peers
                {
                    match peer_state
                    {
                        PeerState::Connected =>
                        {
                            if server.is_none()
                            {
                                *server = Some(peer_id);
                                connection.state = ConnectionState::Connected;
                            }
                            else
                            {
                                log::error!("net system: another peer connected in a client-server connection, ignored");
                            }
                        }
                        PeerState::Disconnected =>
                        {
                            connection.state = ConnectionState::Disconnected(DisconnectReason::ClosedByRemote);
                            return;
                        }
                    }
                }
            }

            if connection.state == ConnectionState::Connected
            {
                for (_, packet) in socket.channel_mut(0).receive()
                {
                    connection.received.push(packet);
                }

                for (_, packet) in socket.channel_mut(1).receive()
                {
                    connection.received.push(packet);
                }
            }
        }
    }
}
