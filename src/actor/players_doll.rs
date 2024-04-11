use glam::{Vec3, Vec4};
use matchbox_socket::PeerId;

use crate::{engine::{engine_handle::{Command, CommandType, EngineHandle}, net::{NetCommand, NetMessage, RemoteMessage}, physics::{colliders_container::PhysicalElement, dynamic_collider::PlayersDollCollider, physics_system_data::ShapeType, static_collider::StaticCollider, PhysicsSystem}, render::VisualElement, world::static_object::{self, ObjectMatrial, StaticObject}}, transform::Transform};

use super::{player::PlayerMessages, Actor, ActorID, CommonActorsMessages, Component, Message, MessageType, SpecificActorMessage};


const PLAYERS_DOLL_COLOR: Vec3 = Vec3::new(0.8, 0.8, 0.8);
pub struct PlayersDoll {
    id: Option<ActorID>,
    transform: Transform,
    masters_peer_id: PeerId,

    dynamic_colliders: Vec<PlayersDollCollider>,
    is_enable: bool,
}

impl PlayersDoll {
    pub fn new(masters_peer_id: PeerId, id: ActorID, player_sphere_radius: f32, transform: Transform) -> Self {

        let weapon_offset = {
            Vec4::new(
                1.0,
                0.26,
                0.0,
                0.0
            ).normalize() * (player_sphere_radius * 1.35)
        };

        let dynamic_collider = PlayersDollCollider {
            position: Vec4::ZERO,
            radius: player_sphere_radius,
            friction: 0.0,
            bounce_rate: 0.0,
            actors_id: Some(id),
            weapon_offset,
        };

        let mut dynamic_colliders = Vec::with_capacity(1);

        dynamic_colliders.push(dynamic_collider);

        PlayersDoll {
            masters_peer_id,
            id: Some(id),
            transform,
            is_enable: true,
            dynamic_colliders,
        }
    }
}

impl Actor for PlayersDoll {
    fn recieve_message(&mut self, message: &Message, engine_handle: &mut EngineHandle) {
        let from = message.from;

        let message = &message.message;
        
        match message {
            MessageType::CommonActorsMessages(message) => {
                match message {
                    &CommonActorsMessages::SetTransform(transform) => {
                        self.transform = transform.clone();
                    },
                    CommonActorsMessages::Enable(switch) => {},

                    CommonActorsMessages::IncrementPosition(increment) => {
                        self.transform.increment_position(increment.clone());
                    },
                    CommonActorsMessages::IWasChangedMyId(new_id) => {}
                }
            }
            MessageType::PhysicsMessages(message) => {
                match message {
                    _ => {}
                }
            },
            MessageType::SpecificActorMessage(message) => {
                match &message {
                    &SpecificActorMessage::PLayerMessages(message) => {
                        match message {
                            PlayerMessages::DealDamageAndAddForce(damage, force) => {
                                engine_handle.send_command(
                                    Command {
                                        sender: self.id.expect("Player's Doll have not Actor's ID"),
                                        command_type: CommandType::NetCommand(
                                            NetCommand::SendDirectNetMessageReliable(
                                                NetMessage::RemoteDirectMessage(
                                                    self.id.expect("Player's Doll have not Actor's ID"),
                                                    RemoteMessage::DealDamageAndAddForce(
                                                        *damage,
                                                        force.to_array(),
                                                    )
                                                ),
                                                self.masters_peer_id
                                            )
                                        )
                                    }
                                )
                            }
                            PlayerMessages::NewPeerConnected(_) => {}
                        }
                    },
                    // _ => {},
                }

            }  
        }
    }


    fn get_mut_transform(&mut self) -> &mut Transform {
        &mut self.transform
    }


    fn get_transform(&self) -> &Transform {
        &self.transform
    }


    fn init(&mut self, id: ActorID) {
        self.id = Some(id);

        for collider in self.dynamic_colliders.iter_mut() {
            collider.init(id);
        }
    }


    fn get_id(&self) -> Option<ActorID> {
        self.id
    }

    fn set_id(&mut self, id: ActorID, engine_handle: &mut EngineHandle) {
        
        if let Some(prev_id) = self.id {
            engine_handle.send_boardcast_message(Message {
                from: prev_id,
                message: MessageType::CommonActorsMessages(
                    CommonActorsMessages::IWasChangedMyId(
                        id
                    )
                )
            });
        }

        self.id = Some(id);
    }

    fn get_physical_element(&mut self) -> Option<PhysicalElement> {
        if self.is_enable {
            Some(
                PhysicalElement {
                    transform: &mut self.transform,
                    kinematic_collider: None,
                    static_colliders: None,
                    dynamic_colliders: Some(&mut self.dynamic_colliders),
                    static_objects: None,
                    area: None,
                }
            )
        } else {
            None
        }
    }

    fn get_visual_element(&self) -> Option<VisualElement> {
        if self.is_enable {
            Some(
                VisualElement {
                    transform: &self.transform,
                    static_objects: None,
                    coloring_areas: None,
                    volume_areas: None,
                    player: Some(&self.dynamic_colliders[0])
                }
            )
        } else {
            None
        }
    }

    fn tick(
        &mut self,
        physic_system: &PhysicsSystem,
        engine_handle: &mut EngineHandle,
        delta: f32
    ) {
        if self.is_enable {

        } else {

        }
    }
}