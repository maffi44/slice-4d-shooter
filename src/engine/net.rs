use glam::{Vec3, Vec4};
#[cfg(not(target_arch = "wasm32"))]
use tokio::runtime::Runtime;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;

use matchbox_socket::{
    MultipleChannels, PeerId, PeerState, RtcIceServerConfig, WebRtcSocket
};

use crate::{
    actor::{
        player::PlayerMessages,
        players_death_explosion::PlayersDeathExplosion,
        players_doll::{
            PlayersDoll,
            PlayersDollMessages},
        ActorID,
        ActorWrapper,
        CommonActorsMessages,
        Message,
        MessageType,
        SpecificActorMessage
    },
    transform::{
        SerializableTransform,
        Transform
    }
};

use super::{
    audio::AudioSystem,
    engine_handle::{
        Command,
        CommandType,
        EngineHandle
    }
};

use alkahest::{alkahest, Serialize};

type Packet = Box<[u8]>;

#[repr(C)]
#[alkahest(Formula, Serialize, Deserialize)]
pub enum NetMessage {
    RemoteCommand(RemoteCommand),
    RemoteDirectMessage(ActorID, RemoteMessage),
    RemoteBoardCastMessage(RemoteMessage),
}

#[repr(C)]
#[alkahest(Formula, Serialize, Deserialize)]
pub enum RemoteCommand {
    // transform, radius, is_alive status
    SpawnPlayersDollActor(SerializableTransform, f32, bool),
    SpawnPlayerDeathExplode([f32;4]),
    RemoveActor(ActorID),
}

#[repr(C)]
#[alkahest(Formula, Serialize, Deserialize)]
pub enum RemoteMessage {
    DealDamageAndAddForce(u32, [f32;4], [f32;4]),
    DieImmediately,
    DieSlowly,
    PlayerRespawn([f32;4]),
    Enable(bool),
    SetTransform(SerializableTransform),
    SpawnHoleGunShotActor([f32;4], [f32;4], f32, [f32;3], f32),
    SpawHoleGunMissActor([f32;4], [f32;4], f32, [f32;3], f32),
    HoleGunStartCharging,
    SpawnMachineGunShot([f32;4], bool)
}

impl NetMessage {
    pub fn to_packet(self) -> Packet {
        
        let size = <NetMessage as Serialize<NetMessage>>::size_hint(&self).unwrap();
        
        let mut packet: Vec<u8> = Vec::with_capacity(size.heap);

        alkahest::serialize_to_vec::<NetMessage, NetMessage>(self, &mut packet);

        packet.into_boxed_slice()
    }

    pub fn from_packet(packet: Packet) -> Option<Self> {
        if let Ok(message) = alkahest::deserialize::<NetMessage, NetMessage>(&packet) {
            Some(message)
        } else {
            None
        }
    }
}


pub enum NetCommand {
    NetSystemIsConnectedAndGetNewPeerID(u128),
    PeerConnected(PeerId),
    PeerDisconnected(PeerId),

    SendDirectNetMessageReliable(NetMessage, PeerId),
    SendDirectNetMessageUnreliable(NetMessage, PeerId),
    SendBoardcastNetMessageReliable(NetMessage),
    SendBoardcastNetMessageUnreliable(NetMessage),
}
    
pub struct NetSystem {
    socket: WebRtcSocket<MultipleChannels>,
    connected: bool,
    peers: Vec<PeerId>,
}

impl NetSystem {
    pub async fn new(
        #[cfg(not(target_arch = "wasm32"))]
        async_runtime: &mut Runtime
    ) -> Self {

        let (socket, socket_future) = matchbox_socket::WebRtcSocketBuilder::new("ws://localhost:3536/")
            .ice_server(RtcIceServerConfig::default())
            .add_reliable_channel()
            .add_unreliable_channel()
            .build();

        #[cfg(target_arch = "wasm32")]
        {
            let promise = wasm_bindgen_futures::future_to_promise(async {
                let _ = socket_future.await;
    
                Result::Ok(JsValue::null())
            });

            let _ = wasm_bindgen_futures::JsFuture::from(promise);
        }
        #[cfg(not(target_arch = "wasm32"))]
        async_runtime.spawn(socket_future);



        NetSystem {
            socket,
            connected: false,
            peers: Vec::new(),
        }
    }

    pub fn tick(
        &mut self,
        engine_handle: &mut EngineHandle,
        #[cfg(not(target_arch = "wasm32"))]
        async_runtime: &mut Runtime,
        audio_system: &mut AudioSystem
    ) {

        if self.socket.any_closed() {

            log::warn!("Net system: connection to signaling server is lost");
            #[cfg(not(target_arch = "wasm32"))]
            self.reconnect(async_runtime);
            #[cfg(target_arch = "wasm32")]
            self.reconnect();
            return;
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
                                NetCommand::PeerConnected(peer)
                            ),
                        });
                        self.peers.push(peer);
                    }

                    PeerState::Disconnected => {
                        let mut index = 0usize;
                        let mut finded = false;
                        for stored_peer in self.peers.iter() {
                            if stored_peer.0.as_u128() == peer.0.as_u128() {
                                finded = true;
                                break;
                            }
                            index += 1;
                        }

                        if finded {
                            self.peers.remove(index);

                            engine_handle.send_command(Command {
                                sender: 0_u128,
                                command_type: CommandType::NetCommand(
                                    NetCommand::PeerDisconnected(peer)
                                ),
                            });
                        }
                    }
                }
            }
        }

        for (peer, packet) in self.socket.channel_mut(0).receive() {
            
            if let Some(message) = NetMessage::from_packet(packet) {
                process_message(peer, message, engine_handle, audio_system);
            }
        }

        for (peer, packet) in self.socket.channel_mut(1).receive() {
            
            if let Some(message) = NetMessage::from_packet(packet) {
                process_message(peer, message, engine_handle, audio_system);
            }
        }
    }

    fn reconnect(
        &mut self,
        #[cfg(not(target_arch = "wasm32"))]
        async_runtime: &mut Runtime,
    ) {
        
        log::info!("trying to reconnect");

        let (socket, socket_future) = matchbox_socket::WebRtcSocketBuilder::new("ws://localhost:3536/")
            .add_reliable_channel()
            .add_unreliable_channel()
            .build();

        #[cfg(target_arch = "wasm32")]
        {
            let promise = wasm_bindgen_futures::future_to_promise(async {
                let _ = socket_future.await;
    
                Result::Ok(JsValue::null())
            });

            let _ = wasm_bindgen_futures::JsFuture::from(promise);
        }
        #[cfg(not(target_arch = "wasm32"))]
        async_runtime.spawn(socket_future);

        self.socket = socket;
        self.connected = false;
    }

    pub fn send_boardcast_message_reliable(&mut self, message: NetMessage) {
        let packet = message.to_packet();

        for peer in self.peers.iter() {
            self.socket.channel_mut(0).send(packet.clone(), *peer);
        }
    }

    pub fn send_boardcast_message_unreliable(&mut self, message: NetMessage) {
        let packet = message.to_packet();

        for peer in self.peers.iter() {
            self.socket.channel_mut(1).send(packet.clone(), *peer);
        }
    }
    
    pub fn send_direct_message_reliable(&mut self, message: NetMessage, peer: PeerId) {
        let packet = message.to_packet();

        self.socket.channel_mut(0).send(packet, peer);
    }

    pub fn send_direct_message_unreliable(&mut self, message: NetMessage, peer: PeerId) {
        let packet = message.to_packet();

        self.socket.channel_mut(1).send(packet, peer);
    }
}

fn process_message(peer_id: PeerId, message: NetMessage, engine_handle: &mut EngineHandle, audio_system: &mut AudioSystem) {
    match message {
        NetMessage::RemoteCommand(command) => {
            match command {
                RemoteCommand::RemoveActor(actor_id) => {
                    engine_handle.send_command(Command {
                        sender: 0u128,
                        command_type: CommandType::RemoveActor(actor_id)
                    })
                },
                RemoteCommand::SpawnPlayerDeathExplode(pos) => {
                    let position = Vec4::from_array(pos);

                    let player_death_explode = PlayersDeathExplosion::new(position);

                    engine_handle.send_command(Command {
                        sender: 0u128,
                        command_type: CommandType::SpawnActor(
                            ActorWrapper::PlayersDeathExplosion(player_death_explode)
                        )
                    });
                },
                
                RemoteCommand::SpawnPlayersDollActor(tr, player_sphere_radius, is_alive) => {
                    let transform = Transform::from_serializable_transform(tr);

                    let players_doll = PlayersDoll::new(
                        peer_id,
                        peer_id.0.as_u128(),
                        player_sphere_radius,
                        transform,
                        is_alive,
                        audio_system
                    );

                    let actor = ActorWrapper::PlayersDoll(players_doll);

                    engine_handle.send_command(Command {
                        sender: 0u128,
                        command_type: CommandType::SpawnActor(actor)
                    })
                }
            }

        },

        NetMessage::RemoteDirectMessage(actor_id, message) => {
            match message {
                RemoteMessage::SpawnMachineGunShot(pos, is_miss) => {
                    engine_handle.send_direct_message(
                        actor_id,
                        Message {
                            from: 0u128,
                            message: MessageType::SpecificActorMessage(
                                SpecificActorMessage::PlayersDollMessages(
                                    PlayersDollMessages::SpawnMachineGunShot(
                                        Vec4::from_array(pos),
                                        is_miss
                                    )
                                )
                            )
                        }
                    );
                }
                RemoteMessage::PlayerRespawn(position) => {
                    engine_handle.send_direct_message(
                        actor_id,
                        Message {
                            from: 0u128,
                            message: MessageType::SpecificActorMessage(
                                SpecificActorMessage::PlayersDollMessages(
                                    PlayersDollMessages::Respawn(
                                        Vec4::from_array(position)
                                    )
                                )
                            )
                        }
                    )
                }
                RemoteMessage::HoleGunStartCharging => {
                    engine_handle.send_direct_message(
                        actor_id,
                        Message {
                            from: 0u128,
                            message: MessageType::SpecificActorMessage(
                                SpecificActorMessage::PlayersDollMessages(
                                    PlayersDollMessages::HoleGunStartCharging
                                )
                            )
                        }
                    )
                }
                RemoteMessage::DieImmediately => {
                    engine_handle.send_direct_message(
                        actor_id,
                        Message {
                            from: 0u128,
                            message: MessageType::SpecificActorMessage(
                                SpecificActorMessage::PLayerMessages(
                                    PlayerMessages::DieImmediately
                                )
                            )
                        }
                    )
                },
                RemoteMessage::DieSlowly => {
                    engine_handle.send_direct_message(
                        actor_id,
                        Message {
                            from: 0u128,
                            message: MessageType::SpecificActorMessage(
                                SpecificActorMessage::PLayerMessages(
                                    PlayerMessages::DieSlowly
                                )
                            )
                        }
                    )
                }
                RemoteMessage::SpawHoleGunMissActor(
                    position,
                    shoooted_from,
                    radius,
                    color,
                    charging_volume_area
                ) => {
                    engine_handle.send_direct_message(
                        actor_id,
                        Message {
                            from: 0u128,
                            message: MessageType::SpecificActorMessage(
                                SpecificActorMessage::PlayersDollMessages(
                                    PlayersDollMessages::SpawHoleGunMissActor(
                                        Vec4::from_array(position),
                                        radius,
                                        Vec3::from_array(color),
                                        charging_volume_area
                                    )
                                )
                            )
                        }
                    )
                },
                RemoteMessage::SpawnHoleGunShotActor(
                    position,
                    shoooted_from,
                    radius,
                    color,
                    charging_volume_area
                ) => {
                    engine_handle.send_direct_message(
                        actor_id,
                        Message {
                            from: 0u128,
                            message: MessageType::SpecificActorMessage(
                                SpecificActorMessage::PlayersDollMessages(
                                    PlayersDollMessages::SpawnHoleGunShotActor(
                                        Vec4::from_array(position),
                                        radius,
                                        Vec3::from_array(color),
                                        charging_volume_area
                                    )
                                )
                            )
                        }
                    )
                },
                RemoteMessage::SetTransform(tr) => {
                    let transform = Transform::from_serializable_transform(tr);

                    engine_handle.send_direct_message(
                        actor_id,
                        Message {
                            from: 0u128,
                            message: MessageType::CommonActorsMessages(
                                CommonActorsMessages::SetTransform(transform)
                            )
                        }
                    )
                },
                RemoteMessage::DealDamageAndAddForce(damage, force, impact_pos) => {
                    engine_handle.send_direct_message(
                        actor_id,
                        Message {
                            from: 0u128,
                            message: MessageType::SpecificActorMessage(
                                SpecificActorMessage::PLayerMessages(
                                    PlayerMessages::DealDamageAndAddForce(
                                        damage,
                                        Vec4::from_array(force),
                                        Vec4::from_array(impact_pos),
                                    )
                                )
                            )
                        }
                    )
                },
                RemoteMessage::Enable(enable_state) => {
                    engine_handle.send_direct_message(
                        actor_id,
                        Message {
                            from: 0u128,
                            message: MessageType::CommonActorsMessages(
                                CommonActorsMessages::Enable(enable_state)
                            )
                        }
                    )
                },
            }
        },

        NetMessage::RemoteBoardCastMessage(message) => {
            match message {
                RemoteMessage::SpawnMachineGunShot(pos, is_miss) => {
                    engine_handle.send_boardcast_message(
                        Message {
                            from: 0u128,
                            message: MessageType::SpecificActorMessage(
                                SpecificActorMessage::PlayersDollMessages(
                                    PlayersDollMessages::SpawnMachineGunShot(
                                        Vec4::from_array(pos),
                                        is_miss
                                    )
                                )
                            )
                        }
                    );
                }
                RemoteMessage::PlayerRespawn(position) => {
                    engine_handle.send_boardcast_message(
                        Message {
                            from: 0u128,
                            message: MessageType::SpecificActorMessage(
                                SpecificActorMessage::PlayersDollMessages(
                                    PlayersDollMessages::Respawn(
                                        Vec4::from_array(position)
                                    )
                                )
                            )
                        }
                    )
                }
                RemoteMessage::DieImmediately => {
                    engine_handle.send_boardcast_message(
                        Message {
                            from: 0u128,
                            message: MessageType::SpecificActorMessage(
                                SpecificActorMessage::PLayerMessages(
                                    PlayerMessages::DieImmediately
                                )
                            )
                        }
                    )
                },
                RemoteMessage::DieSlowly => {
                    engine_handle.send_boardcast_message(
                        Message {
                            from: 0u128,
                            message: MessageType::SpecificActorMessage(
                                SpecificActorMessage::PLayerMessages(
                                    PlayerMessages::DieSlowly
                                )
                            )
                        }
                    )
                }
                RemoteMessage::HoleGunStartCharging => {
                    engine_handle.send_boardcast_message(
                        Message {
                            from: 0u128,
                            message: MessageType::SpecificActorMessage(
                                SpecificActorMessage::PlayersDollMessages(
                                    PlayersDollMessages::HoleGunStartCharging
                                )
                            )
                        }
                    )
                }
                RemoteMessage::SpawHoleGunMissActor(
                    position,
                    shoooted_from,
                    radius,
                    color,
                    charging_volume_area
                ) => {
                    engine_handle.send_boardcast_message(
                        Message {
                            from: 0u128,
                            message: MessageType::SpecificActorMessage(
                                SpecificActorMessage::PlayersDollMessages(
                                    PlayersDollMessages::SpawHoleGunMissActor(
                                        Vec4::from_array(position),
                                        radius,
                                        Vec3::from_array(color),
                                        charging_volume_area
                                    )
                                )
                            )
                        }
                    )
                },
                RemoteMessage::SpawnHoleGunShotActor(
                    position,
                    shoooted_from,
                    radius,
                    color,
                    charging_volume_area
                ) => {
                    engine_handle.send_boardcast_message(
                        Message {
                            from: 0u128,
                            message: MessageType::SpecificActorMessage(
                                SpecificActorMessage::PlayersDollMessages(
                                    PlayersDollMessages::SpawnHoleGunShotActor(
                                        Vec4::from_array(position),
                                        radius,
                                        Vec3::from_array(color),
                                        charging_volume_area
                                    )
                                )
                            )
                        }
                    )
                },
                RemoteMessage::SetTransform(tr) => {
                    let transform = Transform::from_serializable_transform(tr);

                    engine_handle.send_boardcast_message(
                        Message {
                            from: 0u128,
                            message: MessageType::CommonActorsMessages(
                                CommonActorsMessages::SetTransform(transform)
                            )
                        }
                    )
                },
                RemoteMessage::DealDamageAndAddForce(damage, force, impact_pos) => {
                    engine_handle.send_boardcast_message(
                        Message {
                            from: 0u128,
                            message: MessageType::SpecificActorMessage(
                                SpecificActorMessage::PLayerMessages(
                                    PlayerMessages::DealDamageAndAddForce(
                                        damage,
                                        Vec4::from_array(force),
                                        Vec4::from_array(impact_pos),
                                    )
                                )
                            )
                        }
                    )
                },
                RemoteMessage::Enable(enable_state) => {
                    engine_handle.send_boardcast_message(
                        Message {
                            from: 0u128,
                            message: MessageType::CommonActorsMessages(
                                CommonActorsMessages::Enable(enable_state)
                            )
                        }
                    )
                },
            }
        }
    }
}
