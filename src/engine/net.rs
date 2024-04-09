use std::vec;

use glam::{Vec3, Vec4};
use wasm_bindgen::JsValue;

use matchbox_socket::{
    MultipleChannels, PeerId, PeerState, RtcIceServerConfig, WebRtcSocket
};

use crate::{actor::{holegun_miss::HoleGunMiss, holegun_shot::HoleGunShot, player::PlayerMessages, players_doll::PlayersDoll, ActorID, ActorWrapper, CommonActorsMessages, Message, MessageType, SpecificActorMessage}, transform::{self, SerializableTransform, Transform}};

use super::{engine_handle::{
    Command, CommandType, EngineHandle
}, world::static_object::{SphericalVolumeArea, VolumeArea}};

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
    SpawnHoleGunShotActor([f32;4], [f32;4], f32, [f32;3], f32),
    SpawHoleGunMissActor([f32;4], [f32;4], f32, [f32;3], f32),
    SpawnPlayersDollActor(SerializableTransform, f32),
    RemoveActor(ActorID),
}

#[repr(C)]
#[alkahest(Formula, Serialize, Deserialize)]
pub enum RemoteMessage {
    DealDamage(u32),
    Enable(bool),
    SetTransform(SerializableTransform)
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
            connected: false,
            peers: Vec::new(),
        }
    }

    pub fn tick(&mut self, engine_handle: &mut EngineHandle) {

        if self.socket.any_closed() {

            log::warn!("Net system: connection to signaling server is lost");
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
                        engine_handle.send_command(Command {
                            sender: 0_u128,
                            command_type: CommandType::NetCommand(
                                NetCommand::PeerDisconnected(peer)
                            ),
                        });
                        
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
                        }
                    }
                }
            }
        }

        for (peer, packet) in self.socket.channel_mut(0).receive() {
            
            if let Some(message) = NetMessage::from_packet(packet) {
                process_message(peer, message, engine_handle);
            }
        }

        for (peer, packet) in self.socket.channel_mut(1).receive() {
            
            if let Some(message) = NetMessage::from_packet(packet) {
                process_message(peer, message, engine_handle);
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

fn process_message(peer_id: PeerId, message: NetMessage, engine_handle: &mut EngineHandle) {
    match message {
        NetMessage::RemoteCommand(command) => {
            match command {
                RemoteCommand::RemoveActor(actor_id) => {
                    engine_handle.send_command(Command {
                        sender: 0u128,
                        command_type: CommandType::RemoveActor(actor_id)
                    })
                },
                RemoteCommand::SpawHoleGunMissActor(
                    position,
                    shoooted_from,
                    radius,
                    color,
                    charging_volume_area
                ) => {
                    let charging_volume_area = VolumeArea::SphericalVolumeArea(
                        SphericalVolumeArea {
                            translation: Vec4::from_array(shoooted_from),
                            radius: charging_volume_area,
                            color: Vec3::from_array(color),
                        }
                    );

                    let holegun_miss = HoleGunMiss::new(
                        Vec4::from_array(position),
                        Vec4::from_array(shoooted_from),
                        radius,
                        Vec3::from_array(color),
                        charging_volume_area
                    );

                    let actor = ActorWrapper::HoleGunMiss(holegun_miss);

                    engine_handle.send_command(Command {
                        sender: 0u128,
                        command_type: CommandType::SpawnActor(actor)
                    })
                },
                RemoteCommand::SpawnHoleGunShotActor(
                    position,
                    shoooted_from,
                    radius,
                    color,
                    charging_volume_area
                ) => {
                    let charging_volume_area = VolumeArea::SphericalVolumeArea(
                        SphericalVolumeArea {
                            translation: Vec4::from_array(shoooted_from),
                            radius: charging_volume_area,
                            color: Vec3::from_array(color),
                        }
                    );

                    let holegun_shot = HoleGunShot::new(
                        Vec4::from_array(position),
                        Vec4::from_array(shoooted_from),
                        radius,
                        Vec3::from_array(color),
                        charging_volume_area
                    );

                    let actor = ActorWrapper::HoleGunShot(holegun_shot);

                    engine_handle.send_command(Command {
                        sender: 0u128,
                        command_type: CommandType::SpawnActor(actor)
                    })
                },
                RemoteCommand::SpawnPlayersDollActor(tr, player_sphere_radius) => {
                    let transform = Transform::from_serializable_transform(tr);

                    let players_doll = PlayersDoll::new(
                        peer_id,
                        peer_id.0.as_u128(),
                        player_sphere_radius,
                        transform
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
                RemoteMessage::DealDamage(damage) => {
                    engine_handle.send_direct_message(
                        actor_id,
                        Message {
                            from: 0u128,
                            message: MessageType::SpecificActorMessage(
                                SpecificActorMessage::PLayerMessages(
                                    PlayerMessages::DealDamage(damage)
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
                RemoteMessage::DealDamage(damage) => {
                    engine_handle.send_boardcast_message(
                        Message {
                            from: 0u128,
                            message: MessageType::SpecificActorMessage(
                                SpecificActorMessage::PLayerMessages(
                                    PlayerMessages::DealDamage(damage)
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
