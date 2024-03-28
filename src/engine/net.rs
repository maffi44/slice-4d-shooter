use wasm_bindgen::JsValue;

use matchbox_socket::{
    WebRtcSocket,
    PeerState,
};

use crate::actor::ActorID;

use super::engine_handle::{
    Command,
    EngineHandle,
    CommandType
};


pub enum NetCommand {
    NetSystemIsConnectedAndGetNewPeerID(u128),
    PeerConnected(ActorID),
    PeerDisconnected(ActorID),
}

pub struct NetSystem {
    unreliable_socket: WebRtcSocket,
    reliable_socket: WebRtcSocket,
    connected: bool,
}

impl NetSystem {
    pub async fn new() -> Self {
        
        let (unreliable_socket, unreliable_socket_future) = WebRtcSocket::new_unreliable("ws://localhost:3536/");
        let (reliable_socket, reliable_socket_future) = WebRtcSocket::new_reliable("ws://localhost:3536/");

        let unreliable_promise = wasm_bindgen_futures::future_to_promise(async {
            let _ = unreliable_socket_future.await;

            Result::Ok(JsValue::null())
        });

        let _ = wasm_bindgen_futures::JsFuture::from(unreliable_promise);

        let reliable_promise = wasm_bindgen_futures::future_to_promise(async {
            let _ = reliable_socket_future.await;

            Result::Ok(JsValue::null())
        });

        let _ = wasm_bindgen_futures::JsFuture::from(reliable_promise);

        NetSystem {
            unreliable_socket,
            reliable_socket,
            connected: false
        }
    }

    pub fn tick(&mut self, engine_handle: &mut EngineHandle) {

        if self.unreliable_socket.is_closed() || self.reliable_socket.is_closed(){

            log::warn!("Net system: connection to signaling server is lost");
            self.reconnect();
        }

        if !self.connected {
            if let Some(id) = self.reliable_socket.id() {
                self.connected = true;

                engine_handle.send_command(Command {
                    sender: 0_u128,
                    command_type: CommandType::NetCommand(
                        NetCommand::NetSystemIsConnectedAndGetNewPeerID(id.0.as_u128())
                    ),
                });
            }
        }

        if let Ok(vec) = self.reliable_socket.try_update_peers() {
            for (peer, state) in vec {
                match state {
                    PeerState::Connected => {
                        engine_handle.send_command(Command {
                            sender: 0_u128,
                            command_type: CommandType::NetCommand(
                                NetCommand::PeerConnected(peer.0.as_u128())
                            ),
                        });
                    }
                    PeerState::Disconnected => {
                        engine_handle.send_command(Command {
                            sender: 0_u128,
                            command_type: CommandType::NetCommand(
                                NetCommand::PeerDisconnected(peer.0.as_u128())
                            ),
                        });
                    }
                }
            }
        }


    }

    fn reconnect(&mut self) {
        
        log::info!("trying to reconnect");

        self.reliable_socket.close();
        self.unreliable_socket.close();
        
        let (unreliable_socket, unreliable_socket_future) = WebRtcSocket::new_unreliable("ws://localhost:3536/");
        let (reliable_socket, reliable_socket_future) = WebRtcSocket::new_reliable("ws://localhost:3536/");

        let unreliable_promise = wasm_bindgen_futures::future_to_promise(async {
            let _ = unreliable_socket_future.await;

            Result::Ok(JsValue::null())
        });

        let _ = wasm_bindgen_futures::JsFuture::from(unreliable_promise);

        let reliable_promise = wasm_bindgen_futures::future_to_promise(async {
            let _ = reliable_socket_future.await;

            Result::Ok(JsValue::null())
        });

        let _ = wasm_bindgen_futures::JsFuture::from(reliable_promise);

        self.unreliable_socket = unreliable_socket;
        self.reliable_socket = reliable_socket;
        self.connected = false;
    }
}