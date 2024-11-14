pub mod net_protocols;

use fyrox_core::futures::{SinkExt, StreamExt};
use glam::{Vec3, Vec4};
use net_protocols::{ClientMessage, ServerMessage};
#[cfg(not(target_arch = "wasm32"))]
use tokio::runtime::Runtime;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;

use tokio_tungstenite::{connect_async, tungstenite};

use matchbox_socket::{
    MultipleChannels, PeerId, PeerState, RtcIceServerConfig, WebRtcSocket
};

use crate::{
    actor::{
        player::{player_settings::PlayerSettings, PlayerMessages},
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
    }, matchmaking_server_protocol::{ClientMatchmakingServerProtocol, MatchmakingServerMessage}, transform::{
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
#[derive(Clone)]
pub enum NetMessage {
    RemoteCommand(RemoteCommand),
    RemoteDirectMessage(ActorID, RemoteMessage),
    RemoteBoardCastMessage(RemoteMessage),
}

#[repr(C)]
#[alkahest(Formula, Serialize, Deserialize)]
#[derive(Clone)]
pub enum RemoteCommand {
    // transform, radius, is_alive status
    SpawnPlayersDollActor(SerializableTransform, f32, bool),
    SpawnPlayerDeathExplode([f32;4]),
    RemoveActor(ActorID),
}

#[repr(C)]
#[alkahest(Formula, Serialize, Deserialize)]
#[derive(Clone)]
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
    PeerConnected(u128),
    PeerDisconnected(u128),

    SendDirectNetMessageReliable(NetMessage, u128),
    SendDirectNetMessageUnreliable(NetMessage, u128),
    SendBoardcastNetMessageReliable(NetMessage),
    SendBoardcastNetMessageUnreliable(NetMessage),
}
    
pub struct NetSystem {
    socket: WebRtcSocket<MultipleChannels>,
    connected: bool,
    other_players_ids: Vec<u128>,
    game_server_peer_id: Option<PeerId>,
}

impl NetSystem {
    pub async fn new(
        settings: &PlayerSettings,
        #[cfg(not(target_arch = "wasm32"))]
        async_runtime: &mut Runtime
    ) -> Self {

        let (socket, socket_future) = matchbox_socket::WebRtcSocketBuilder::new(settings.room_url.clone())
            .ice_server(RtcIceServerConfig {
                urls: settings.bash_and_turn_servers.clone(),
                username: Some(settings.turn_server_username.clone()),
                credential: Some(settings.turn_server_credential.clone()),
            })
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
            other_players_ids: Vec::new(),
            game_server_peer_id: None,
        }
    }

    async fn get_game_server_url(matchmaking_server_url: String) -> String {
        
        let (mut ws_stream, _) =
            connect_async(matchmaking_server_url)
            .await
            .expect("Failed to connect to matchmaking server");

        let message = ClientMatchmakingServerProtocol::ClientMessage(
            crate::matchmaking_server_protocol::ClientMessage::RequestToConnectToGameServer(
                (0,0,0)
            )
        ).to_packet();

        while let Err(_) = ws_stream
            .send(tokio_tungstenite::tungstenite::Message::binary(message.clone()))
            .await
        {}

        if let Some(Ok(message)) = ws_stream.next().await {
            if let Ok(message) =
                alkahest::deserialize::<ClientMatchmakingServerProtocol, ClientMatchmakingServerProtocol>(&message.into_data())
            {
                match message {
                    ClientMatchmakingServerProtocol::MatchmakingServerMessage(
                        message
                    ) => {
                        match message {
                            MatchmakingServerMessage::GameServerAddress(url) => {

                            }
                            MatchmakingServerMessage::NoFreeServers => {

                            }
                            MatchmakingServerMessage::WrongGameVersionCorrectIs(correct_game_version) => {
                                
                            }
                        }
                    }
                    _ => {}
                }
            }
        }


        String::new()
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

        if let Ok(vec) = self.socket.try_update_peers() {
            for (peer_id, peer_state) in vec {
                match peer_state {
                    PeerState::Connected => {
                        self.connected = true;
                        self.game_server_peer_id = Some(peer_id);

                        engine_handle.send_command(Command {
                            sender: 0_u128,
                            command_type: CommandType::NetCommand(
                                NetCommand::NetSystemIsConnectedAndGetNewPeerID(
                                    self
                                        .socket
                                        .id()
                                        .expect("ERROR: A registrated peer (game server) connection, but the game client still does not have id in the p2p network")
                                        .0
                                        .as_u128()
                                )
                            ),
                        });
                    }
                    PeerState::Disconnected => {
                        self.connected = false;
                        self.game_server_peer_id = None;

                        for player_id in self.other_players_ids.iter() {
                            engine_handle.send_command(Command {
                                sender: 0_u128,
                                command_type: CommandType::NetCommand(
                                    NetCommand::PeerDisconnected(*player_id)
                                ),
                            });
                        }
                        self.other_players_ids.clear();
                    }
                }   
            }
        }

        

        for (_, packet) in self.socket.channel_mut(0).receive() {

            if let Some(message) = ServerMessage::from_packet(packet) {
                match message {

                    ServerMessage::PlayerConnected(player_id) => {
                        engine_handle.send_command(Command {
                            sender: 0_u128,
                            command_type: CommandType::NetCommand(
                                NetCommand::PeerConnected(player_id)
                            ),
                        });
                        self.other_players_ids.push(player_id);
                    }

                    ServerMessage::PlayerDisconnected(player_id) => {
                        let mut index = 0usize;
                        let mut finded = false;
                        for stored_peer in self.other_players_ids.iter() {
                            if *stored_peer == player_id {
                                finded = true;
                                break;
                            }
                            index += 1;
                        }

                        if finded {
                            self.other_players_ids.remove(index);

                            engine_handle.send_command(Command {
                                sender: 0_u128,
                                command_type: CommandType::NetCommand(
                                    NetCommand::PeerDisconnected(player_id)
                                ),
                            });
                        }
                    }
                    
                    ServerMessage::NetMessage(from_player, message) => {
                        process_message(from_player, message, engine_handle, audio_system);
                    }
                }
            }
        }

        for (_, packet) in self.socket.channel_mut(1).receive() {
            
            if let Some(message) = ServerMessage::from_packet(packet) {
                match message {

                    ServerMessage::PlayerConnected(player_id) => {
                        engine_handle.send_command(Command {
                            sender: 0_u128,
                            command_type: CommandType::NetCommand(
                                NetCommand::PeerConnected(player_id)
                            ),
                        });
                        self.other_players_ids.push(player_id);
                    }

                    ServerMessage::PlayerDisconnected(player_id) => {
                        let mut index = 0usize;
                        let mut finded = false;
                        for stored_peer in self.other_players_ids.iter() {
                            if *stored_peer == player_id {
                                finded = true;
                                break;
                            }
                            index += 1;
                        }

                        if finded {
                            self.other_players_ids.remove(index);

                            engine_handle.send_command(Command {
                                sender: 0_u128,
                                command_type: CommandType::NetCommand(
                                    NetCommand::PeerDisconnected(player_id)
                                ),
                            });
                        }
                    }
                    
                    ServerMessage::NetMessage(from_player, message) => {
                        process_message(from_player, message, engine_handle, audio_system);
                    }
                }
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

        if let Some(game_server_id) = self.game_server_peer_id {

            let packet = ClientMessage::BoardcastMessage(message).to_packet();
    
            for peer in self.other_players_ids.iter() {
                self.socket
                    .channel_mut(0)
                    .send(
                        packet.clone(),
                        game_server_id
                    );
            }
        }

    }

    pub fn send_boardcast_message_unreliable(&mut self, message: NetMessage) {

        if let Some(game_server_id) = self.game_server_peer_id {

            let packet = ClientMessage::BoardcastMessage(message).to_packet();
    
            for peer in self.other_players_ids.iter() {
                self.socket
                    .channel_mut(1)
                    .send(
                        packet.clone(),
                        game_server_id
                    );
            }
        }
    }
    
    pub fn send_direct_message_reliable(&mut self, message: NetMessage, peer: u128) {

        if let Some(game_server_id) = self.game_server_peer_id {

            let packet = ClientMessage::DirectMessage(peer, message).to_packet();
    
            for peer in self.other_players_ids.iter() {
                self.socket
                    .channel_mut(0)
                    .send(
                        packet.clone(),
                        game_server_id
                    );
            }
        }
    }

    pub fn send_direct_message_unreliable(&mut self, message: NetMessage, peer: u128) {
        
        if let Some(game_server_id) = self.game_server_peer_id {

            let packet = ClientMessage::DirectMessage(peer, message).to_packet();
    
            for peer in self.other_players_ids.iter() {
                self.socket
                    .channel_mut(1)
                    .send(
                        packet.clone(),
                        game_server_id
                    );
            }
        }
    }
}

fn process_message(peer_id: u128, message: NetMessage, engine_handle: &mut EngineHandle, audio_system: &mut AudioSystem) {
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
