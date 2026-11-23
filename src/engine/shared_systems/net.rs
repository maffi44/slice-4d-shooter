// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2026  Timofei Molokov



use std::{
    collections::HashMap,
    sync::{Arc, Weak},
    time::Duration,
};

use fyrox_core::futures::{SinkExt, StreamExt};
use matchbox_socket::{
    PeerId,
    PeerState,
    RtcIceServerConfig,
    WebRtcSocket
};
use tokio::{runtime::Handle, sync::mpsc};
use tokio_tungstenite::{connect_async, tungstenite};


pub use matchbox_socket::Packet;


/// Each clone keeps the same connection registered. After the last handle is dropped,
/// the next network tick removes the entry before polling it.
#[derive(Clone, Debug)]
pub struct ConnectionHandle(Arc<()>);

impl ConnectionHandle
{
    fn key(&self) -> usize
    {
        // The registry's Weak keeps this allocation distinct until the entry is removed.
        Arc::as_ptr(&self.0).addr()
    }
}


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


/// What the connection is doing now. A query for the owner's state machine (`NetSystem::state`);
/// operations on the connection report the same situation through `ConnectionError`.
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
    /// The handle belongs to another `NetSystem`. Owning handles cannot outlive their connection
    /// in their own `NetSystem`, so nothing else produces this reason.
    UnknownConnection,
}


/// Why an operation on a connection (`send`, `receive`) could not be performed.
#[derive(Clone, Debug, PartialEq)]
pub enum ConnectionError
{
    /// The connection is not established yet; nothing can be sent or received.
    Connecting,
    /// The connection is closed. For `receive` this is returned only after every packet
    /// that arrived before the disconnect has been taken.
    Disconnected(DisconnectReason),
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
    connections: HashMap<usize, Connection>,
}


struct Connection
{
    /// Expires when the last handle is dropped.
    owner: Weak<()>,
    state: ConnectionState,
    received: Vec<Packet>,
    transport: Transport,
}


/// Dropping the event receiver cancels the WebSocket task, including a blocked write.
/// The WebRTC registry entry owns its socket task, including the initial signaling handshake.
enum Transport
{
    WebSocket {
        events: mpsc::UnboundedReceiver<WebSocketEvent>,
        outgoing: Option<mpsc::UnboundedSender<Packet>>,
    },
    WebRtc {
        socket: WebRtcSocket,
        server: Option<PeerId>,
        task: tokio::task::JoinHandle<Result<(), matchbox_socket::Error>>,
    },
}


impl Drop for Transport
{
    fn drop(&mut self)
    {
        if let Transport::WebRtc { task, .. } = self
        {
            // Matchbox 0.15 awaits the initial signaling handshake before polling its
            // outgoing channels. Cancellation also drops the guards for existing peers.
            task.abort();
        }
    }
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
        }
    }

    pub fn connect(&mut self, config: ConnectionConfig) -> ConnectionHandle
    {
        let handle = ConnectionHandle(Arc::new(()));

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

                let task = self.runtime.spawn(socket_future);

                Transport::WebRtc { socket, server: None, task }
            }
        };

        self.connections.insert(handle.key(), Connection {
            owner: Arc::downgrade(&handle.0),
            state: ConnectionState::Connecting,
            received: Vec::new(),
            transport,
        });

        handle
    }

    pub fn state(&self, handle: &ConnectionHandle) -> ConnectionState
    {
        match self.connections.get(&handle.key())
        {
            Some(connection) => connection.state.clone(),
            None => ConnectionState::Disconnected(DisconnectReason::UnknownConnection),
        }
    }

    /// Sends a packet. Sends are immediate (no buffering until `flush`), for minimal latency.
    ///
    /// `Ok` means the packet was handed to the transport, not that it was delivered. A failure
    /// detected by the transport during the call is returned at once; `state` reports it from the
    /// next `tick`.
    pub fn send(&mut self, handle: &ConnectionHandle, packet: Packet, reliability: Reliability) -> Result<(), ConnectionError>
    {
        let connection = self.connection_mut(handle)?;

        match &connection.state
        {
            ConnectionState::Connected => {}
            ConnectionState::Connecting => return Err(ConnectionError::Connecting),
            ConnectionState::Disconnected(reason) => return Err(ConnectionError::Disconnected(reason.clone())),
        }

        match &mut connection.transport
        {
            Transport::WebSocket { outgoing, .. } =>
            {
                // `outgoing` is removed only together with the switch to `Disconnected`
                let Some(outgoing) = outgoing else {
                    return Err(ConnectionError::Disconnected(DisconnectReason::Lost("the connection is closed".to_owned())));
                };

                outgoing.send(packet).map_err(|_| {
                    // the writer task has stopped; it reports the exact reason, seen on the next tick
                    ConnectionError::Disconnected(DisconnectReason::Lost("the WebSocket writer has stopped".to_owned()))
                })
            }

            Transport::WebRtc { socket, server, .. } =>
            {
                // `server` is set together with the switch to `Connected`
                let Some(server) = server else {
                    return Err(ConnectionError::Connecting);
                };

                let channel = match reliability
                {
                    Reliability::Reliable => 0,
                    Reliability::Unreliable => 1,
                };

                socket.channel_mut(channel).try_send(packet, *server)
                    .map_err(|_| ConnectionError::Disconnected(DisconnectReason::ClosedByRemote))
            }
        }
    }

    /// Takes the packets received on this connection since the previous call, in arrival order.
    ///
    /// After a disconnect, packets that arrived before it are still returned (the last message before
    /// a server closes the connection is often its answer); `Err(Disconnected)` comes once they are
    /// taken. `drain` keeps the receive buffer's capacity (no reallocation); packets not taken are
    /// dropped when the iterator is dropped.
    pub fn receive(&mut self, handle: &ConnectionHandle) -> Result<std::vec::Drain<'_, Packet>, ConnectionError>
    {
        let connection = self.connection_mut(handle)?;

        match &connection.state
        {
            ConnectionState::Connecting => Err(ConnectionError::Connecting),
            ConnectionState::Disconnected(reason) if connection.received.is_empty() =>
            {
                Err(ConnectionError::Disconnected(reason.clone()))
            }
            _ => Ok(connection.received.drain(..)),
        }
    }

    /// WebRTC only: this client's id in the Matchbox network (the game uses it as the main player's actor id).
    /// A property query like `state`, not an operation: `None` while the id is not assigned yet
    /// or for a transport that has no such id.
    pub fn local_peer_id(&mut self, handle: &ConnectionHandle) -> Option<u128>
    {
        match self.connections.get_mut(&handle.key())
        {
            Some(Connection { transport: Transport::WebRtc { socket, .. }, .. }) =>
            {
                socket.id().map(|id| id.0.as_u128())
            }
            _ => None,
        }
    }

    fn connection_mut(&mut self, handle: &ConnectionHandle) -> Result<&mut Connection, ConnectionError>
    {
        self.connections
            .get_mut(&handle.key())
            .ok_or(ConnectionError::Disconnected(DisconnectReason::UnknownConnection))
    }

    /// Pre-simulate: removes connections whose owners dropped their handles,
    /// then polls the rest, updating their states and receive buffers.
    pub fn tick(&mut self)
    {
        self.connections.retain(|_, connection| connection.owner.strong_count() > 0);

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

        self.runtime.spawn(async move {
            let connect = tokio::time::timeout(connect_timeout, connect_async(url));

            let connect_result = tokio::select! {
                // the connection was dropped while connecting
                _ = event_sender.closed() => return,
                connect_result = connect => connect_result,
            };

            let ws_stream = match connect_result
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

            // Both halves belong to this task. Removing the registry entry cancels an
            // in-flight write even when the peer stops reading and the queue still has data.
            let write = async {
                while let Some(packet) = outgoing_receiver.recv().await
                {
                    sink.send(tungstenite::Message::binary(packet)).await?;
                }
                sink.close().await
            };

            let read = async {
                loop
                {
                    let packet = match stream.next().await
                    {
                        Some(Ok(tungstenite::Message::Binary(data))) => data,
                        Some(Ok(tungstenite::Message::Text(text))) => text.into(),
                        Some(Ok(tungstenite::Message::Close(_))) | None => return DisconnectReason::ClosedByRemote,
                        Some(Err(error)) => return DisconnectReason::Lost(error.to_string()),
                        _ => continue,
                    };
                    if event_sender.send(WebSocketEvent::Packet(packet)).is_err()
                    {
                        return DisconnectReason::ClosedByRemote;
                    }
                }
            };

            let reason = tokio::select! {
                _ = event_sender.closed() => return,
                result = write => match result {
                    Ok(()) => return,
                    Err(error) => DisconnectReason::Lost(error.to_string()),
                },
                reason = read => reason,
            };
            let _ = event_sender.send(WebSocketEvent::Closed(reason));
            // A peer's Close queues a reply in tungstenite. Flush it while the owner exists.
            tokio::select! {
                _ = event_sender.closed() => {},
                _ = sink.close() => {},
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

        Transport::WebRtc { socket, server, .. } =>
        {
            if socket.any_channel_closed()
            {
                connection.state = ConnectionState::Disconnected(DisconnectReason::ClosedByRemote);
            }
            else if let Ok(peers) = socket.try_update_peers()
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
                            break;
                        }
                    }
                }
            }

            // Packets that arrived before a disconnect are still delivered (see `NetSystem::receive`);
            // later ticks don't poll a disconnected connection.
            if server.is_some()
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


#[cfg(test)]
mod tests
{
    use std::time::Instant;

    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::{TcpListener, TcpStream},
        runtime::Runtime,
    };

    use super::*;


    const WAIT: Duration = Duration::from_secs(5);
    /// Far longer than `WAIT`: a connect task that is still alive keeps the socket open.
    const LONG_CONNECT_TIMEOUT: Duration = Duration::from_secs(60);


    fn runtime() -> Runtime
    {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .unwrap()
    }

    fn bind(runtime: &Runtime) -> (TcpListener, String)
    {
        let listener = runtime.block_on(TcpListener::bind("127.0.0.1:0")).unwrap();
        let address = listener.local_addr().unwrap().to_string();
        (listener, address)
    }

    fn accept(runtime: &Runtime, listener: &TcpListener) -> TcpStream
    {
        runtime.block_on(async { tokio::time::timeout(WAIT, listener.accept()).await })
            .expect("the client did not connect")
            .unwrap()
            .0
    }

    fn websocket(net: &mut NetSystem, address: &str) -> ConnectionHandle
    {
        net.connect(ConnectionConfig::WebSocket {
            url: format!("ws://{}", address),
            connect_timeout: LONG_CONNECT_TIMEOUT,
        })
    }

    /// Completes the WebSocket handshake by hand, so the server never answers the client's Close frame:
    /// the TCP connection is then closed only when the client drops both halves of its stream.
    async fn handshake(socket: &mut TcpStream)
    {
        let mut request = Vec::new();
        let mut buffer = [0u8; 1024];

        while !request.windows(4).any(|window| window == b"\r\n\r\n")
        {
            let read = socket.read(&mut buffer).await.unwrap();
            assert!(read > 0, "the client closed the socket during the handshake");
            request.extend_from_slice(&buffer[..read]);
        }

        let request = String::from_utf8(request).unwrap();
        let key = request.lines()
            .find_map(|line| {
                let (name, value) = line.split_once(':')?;
                name.trim().eq_ignore_ascii_case("sec-websocket-key").then(|| value.trim().to_owned())
            })
            .expect("no Sec-WebSocket-Key in the handshake request");

        let response = format!(
            "HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: {}\r\n\r\n",
            tungstenite::handshake::derive_accept_key(key.as_bytes()),
        );
        socket.write_all(response.as_bytes()).await.unwrap();
    }

    /// Reads (and discards) until the client closes the TCP connection.
    async fn closed_by_client(socket: &mut TcpStream) -> bool
    {
        let mut buffer = [0u8; 1024];

        let wait_for_eof = async {
            loop
            {
                match socket.read(&mut buffer).await
                {
                    Ok(0) | Err(_) => return,
                    Ok(_) => {}
                }
            }
        };

        tokio::time::timeout(WAIT, wait_for_eof).await.is_ok()
    }

    fn wait_for_state(net: &mut NetSystem, handle: &ConnectionHandle, state: ConnectionState)
    {
        let deadline = Instant::now() + WAIT;

        while net.state(handle) != state
        {
            assert!(Instant::now() < deadline, "the connection is {:?}, expected {:?}", net.state(handle), state);
            std::thread::sleep(Duration::from_millis(5));
            net.tick();
        }
    }


    #[test]
    fn dropped_websocket_connection_closes_while_connecting()
    {
        let runtime = runtime();
        let mut net = NetSystem::new(runtime.handle().clone());
        let (listener, address) = bind(&runtime);

        for _ in 0..3
        {
            let handle = websocket(&mut net, &address);
            let mut socket = accept(&runtime, &listener);
            net.tick();
            assert_eq!(net.state(&handle), ConnectionState::Connecting);

            drop(handle);
            net.tick();

            assert!(net.connections.is_empty());
            assert!(runtime.block_on(closed_by_client(&mut socket)), "the connect task outlived its connection");
        }
    }

    #[test]
    fn dropped_websocket_connection_ends_its_tasks()
    {
        let runtime = runtime();
        let mut net = NetSystem::new(runtime.handle().clone());
        let (listener, address) = bind(&runtime);

        for _ in 0..3
        {
            let handle = websocket(&mut net, &address);
            let mut socket = accept(&runtime, &listener);
            runtime.block_on(handshake(&mut socket));
            wait_for_state(&mut net, &handle, ConnectionState::Connected);

            drop(handle);
            net.tick();

            assert!(net.connections.is_empty());
            assert!(runtime.block_on(closed_by_client(&mut socket)), "the reader or writer task outlived its connection");
        }
    }

    #[test]
    fn only_the_last_handle_expires_the_connection()
    {
        let runtime = runtime();
        let mut net = NetSystem::new(runtime.handle().clone());
        let (listener, address) = bind(&runtime);

        let handle = websocket(&mut net, &address);
        let mut socket = accept(&runtime, &listener);
        runtime.block_on(handshake(&mut socket));
        wait_for_state(&mut net, &handle, ConnectionState::Connected);

        let other_owner = handle.clone();
        drop(handle);
        net.tick();
        assert_eq!(net.state(&other_owner), ConnectionState::Connected);
        net.send(&other_owner, Packet::from_static(&[42]), Reliability::Reliable).unwrap();

        drop(other_owner);
        net.tick();

        assert!(net.connections.is_empty());
        assert!(runtime.block_on(closed_by_client(&mut socket)), "the reader or writer task outlived its connection");
    }

    #[test]
    fn dropping_owner_cancels_a_blocked_websocket_writer()
    {
        let runtime = runtime();
        let mut net = NetSystem::new(runtime.handle().clone());
        let (listener, address) = bind(&runtime);
        let handle = websocket(&mut net, &address);
        let mut socket = accept(&runtime, &listener);
        runtime.block_on(handshake(&mut socket));
        wait_for_state(&mut net, &handle, ConnectionState::Connected);

        // Fill the transport with shared payloads. The server only reads the frame prefix.
        let payload = Packet::from(vec![0; 1024 * 1024]);
        for _ in 0..64
        {
            net.send(&handle, payload.clone(), Reliability::Reliable).unwrap();
        }
        let marker = Arc::<[u8]>::from([42]);
        net.send(&handle, Packet::from_owner(marker.clone()), Reliability::Reliable).unwrap();
        runtime.block_on(async {
            tokio::time::timeout(WAIT, socket.read_u8()).await.unwrap().unwrap();
        });
        assert_eq!(Arc::strong_count(&marker), 2);

        drop(handle);
        net.tick();

        let deadline = Instant::now() + WAIT;
        while Arc::strong_count(&marker) > 1
        {
            assert!(Instant::now() < deadline, "writer kept its queued packet after owner removal");
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    #[test]
    fn surviving_owner_keeps_its_connection()
    {
        let runtime = runtime();
        let mut net = NetSystem::new(runtime.handle().clone());
        let (listener, address) = bind(&runtime);

        let dropped = websocket(&mut net, &address);
        let mut dropped_socket = accept(&runtime, &listener);
        runtime.block_on(handshake(&mut dropped_socket));

        let surviving = websocket(&mut net, &address);
        let mut surviving_socket = accept(&runtime, &listener);
        runtime.block_on(handshake(&mut surviving_socket));

        wait_for_state(&mut net, &dropped, ConnectionState::Connected);
        wait_for_state(&mut net, &surviving, ConnectionState::Connected);

        drop(dropped);
        net.tick();

        assert_eq!(net.connections.len(), 1);
        assert!(runtime.block_on(closed_by_client(&mut dropped_socket)));

        // an unmasked binary frame from the server
        runtime.block_on(surviving_socket.write_all(&[0x82, 3, 1, 2, 3])).unwrap();

        let deadline = Instant::now() + WAIT;
        let packet = loop
        {
            net.tick();
            if let Some(packet) = net.receive(&surviving).unwrap().next()
            {
                break packet;
            }
            assert!(Instant::now() < deadline, "the surviving connection received nothing");
            std::thread::sleep(Duration::from_millis(5));
        };

        assert_eq!(&*packet, &[1, 2, 3]);
        assert_eq!(net.state(&surviving), ConnectionState::Connected);
    }

    #[test]
    fn handles_from_another_system_cannot_access_local_connections()
    {
        let runtime = runtime();
        let mut first = NetSystem::new(runtime.handle().clone());
        let mut second = NetSystem::new(runtime.handle().clone());
        let (_listener, address) = bind(&runtime);
        let a = websocket(&mut first, &address);
        let b = websocket(&mut second, &address);
        let unknown = ConnectionError::Disconnected(DisconnectReason::UnknownConnection);

        assert_eq!(second.state(&a), ConnectionState::Disconnected(DisconnectReason::UnknownConnection));
        assert_eq!(second.send(&a, Packet::from_static(&[1]), Reliability::Reliable), Err(unknown.clone()));
        assert_eq!(first.receive(&b).err(), Some(unknown));

        drop(a);
        first.tick();
        second.tick();
        assert!(first.connections.is_empty());
        assert_eq!(second.state(&b), ConnectionState::Connecting);
    }

    #[test]
    fn operations_report_connecting()
    {
        let runtime = runtime();
        let mut net = NetSystem::new(runtime.handle().clone());
        let (listener, address) = bind(&runtime);

        let handle = websocket(&mut net, &address);
        let _socket = accept(&runtime, &listener);
        net.tick();

        assert_eq!(net.send(&handle, Packet::from_static(&[1]), Reliability::Reliable), Err(ConnectionError::Connecting));
        assert_eq!(net.receive(&handle).err(), Some(ConnectionError::Connecting));
    }

    #[test]
    fn packets_received_before_disconnect_come_before_the_error()
    {
        let runtime = runtime();
        let mut net = NetSystem::new(runtime.handle().clone());
        let (listener, address) = bind(&runtime);

        let handle = websocket(&mut net, &address);
        let mut socket = accept(&runtime, &listener);
        runtime.block_on(handshake(&mut socket));
        wait_for_state(&mut net, &handle, ConnectionState::Connected);

        // the server answers and closes the connection at once, as the matchmaking server does on a wrong version
        runtime.block_on(socket.write_all(&[0x82, 3, 1, 2, 3])).unwrap();
        drop(socket);

        let deadline = Instant::now() + WAIT;
        while !matches!(net.state(&handle), ConnectionState::Disconnected(_))
        {
            assert!(Instant::now() < deadline, "the disconnect was not detected");
            std::thread::sleep(Duration::from_millis(5));
            net.tick();
        }

        let packets: Vec<Packet> = net.receive(&handle).unwrap().collect();
        assert_eq!(packets.len(), 1);
        assert_eq!(&*packets[0], &[1, 2, 3]);

        assert!(matches!(net.receive(&handle), Err(ConnectionError::Disconnected(_))));
        assert!(matches!(net.send(&handle, Packet::from_static(&[1]), Reliability::Reliable), Err(ConnectionError::Disconnected(_))));
    }

    #[test]
    fn dropped_webrtc_connection_closes_signaling_while_connecting()
    {
        let runtime = runtime();
        let mut net = NetSystem::new(runtime.handle().clone());
        let (listener, address) = bind(&runtime);

        for _ in 0..3
        {
            let handle = net.connect(ConnectionConfig::WebRtc {
                url: format!("ws://{}/room", address),
                ice_servers: Vec::new(),
                turn_server_username: None,
                turn_server_credential: None,
            });
            let mut socket = accept(&runtime, &listener);
            net.tick();
            assert_eq!(net.state(&handle), ConnectionState::Connecting);

            drop(handle);
            net.tick();

            assert!(net.connections.is_empty());
            assert!(runtime.block_on(closed_by_client(&mut socket)), "the Matchbox socket future outlived its connection");
        }
    }
}
