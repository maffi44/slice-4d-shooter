use wasm_bindgen::JsValue;

use matchbox_socket::{
    MultipleChannels, PeerState, RtcIceServerConfig, WebRtcSocket
};

use crate::actor::ActorID;

use super::engine_handle::{
    Command,
    EngineHandle,
    CommandType
};

pub enum NetMessage {
    
}
pub enum NetCommand {
    NetSystemIsConnectedAndGetNewPeerID(u128),
    PeerConnected(ActorID),
    PeerDisconnected(ActorID),
    
    SendDirectNetMessageReliable(NetMessage),
    SendDirectNetMessageUnreliable(NetMessage),
    SendBoardcastNetMessageReliable(NetMessage),
    SendBoardcastNetMessageUnreliable(NetMessage),
}
    
pub struct NetSystem {
    socket: WebRtcSocket<MultipleChannels>,
    connected: bool,
}

impl NetSystem {
    pub async fn new() -> Self {

        let (socket, socket_future) = matchbox_socket::WebRtcSocketBuilder::new("ws://localhost:3536/")
            .ice_server(RtcIceServerConfig::default())
            .add_reliable_channel()
            .add_unreliable_channel()
            .build();

        let promise = wasm_bindgen_futures::future_to_promise(async {
            let _ = socket_future.await;

            Result::Ok(JsValue::null())
        });

        let _ = wasm_bindgen_futures::JsFuture::from(promise);

        NetSystem {
            socket,
            connected: false
        }
    }

    pub fn tick(&mut self, engine_handle: &mut EngineHandle) {

        if self.socket.any_closed() {

            log::warn!("Net system: connection to signaling server is lost");
            self.reconnect();
        }

        if !self.connected {
            if let Some(id) = self.socket.id() {
                self.connected = true;

                engine_handle.send_command(Command {
                    sender: 0_u128,
                    command_type: CommandType::NetCommand(
                        NetCommand::NetSystemIsConnectedAndGetNewPeerID(id.0.as_u128())
                    ),
                });
            }
        }

        if let Ok(vec) = self.socket.try_update_peers() {
            for (peer, state) in vec {
                match state {
                    PeerState::Connected => {
                        engine_handle.send_command(Command {
                            sender: 0_u128,
                            command_type: CommandType::NetCommand(
                                NetCommand::PeerConnected(peer.0.as_u128())
                            ),
                        });
                        log::error!("PEER CONNECTED {}", peer.0.as_u128());
                    }
                    PeerState::Disconnected => {
                        engine_handle.send_command(Command {
                            sender: 0_u128,
                            command_type: CommandType::NetCommand(
                                NetCommand::PeerDisconnected(peer.0.as_u128())
                            ),
                        });
                        log::error!("PEER DISCONNECTED {}", peer.0.as_u128());
                    }
                }
            }
        }


    }

    fn reconnect(&mut self) {
        
        log::info!("trying to reconnect");

        let (socket, socket_future) = matchbox_socket::WebRtcSocketBuilder::new("ws://localhost:3536/")
            .add_reliable_channel()
            .add_unreliable_channel()
            .build();

        let promise = wasm_bindgen_futures::future_to_promise(async {
            let _ = socket_future.await;

            Result::Ok(JsValue::null())
        });

        let _ = wasm_bindgen_futures::JsFuture::from(promise);

        self.socket = socket;
        self.connected = false;
    }
}