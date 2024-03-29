use glam::{Vec3, Vec4};

use crate::{engine::{engine_handle::EngineHandle, physics::{colliders_container::PhysicalElement, dynamic_collider::DynamicCollider, physics_system_data::ShapeType, static_collider::StaticCollider, PhysicsSystem}, render::VisualElement, world::static_object::{self, ObjectMatrial, StaticObject}}, transform::Transform};

use super::{player::PlayerMessages, Actor, ActorID, CommonActorsMessages, Component, Message, MessageType, SpecificActorMessage};


const PLAYERS_DOLL_COLOR: Vec3 = Vec3::new(0.8, 0.8, 0.8);
pub struct PlayersDoll {
    id: Option<ActorID>,
    transform: Transform,

    static_objects: Vec<StaticObject>,
    is_enable: bool,
    hp: i32
}

impl PlayersDoll {
    pub fn new(id: ActorID, player_sphere_radius: f32) -> Self {

        let static_object = StaticObject {
            collider: StaticCollider {
                shape_type: ShapeType::Sphere,
                position: Vec4::ZERO,
                size: Vec4::new(player_sphere_radius, 0.0, 0.0, 0.0),
                is_positive: true,
                roundness: 0.0,
                stickiness: false,
                friction: 0.0,
                bounce_rate: 0.0,
                actors_id: Some(id),
            },
            material: ObjectMatrial::new(PLAYERS_DOLL_COLOR),
        };

        let mut static_objects = Vec::with_capacity(1);

        static_objects.push(static_object);

        PlayersDoll {
            id: Some(id),
            transform: Transform::new_zero(),
            is_enable: true,
            static_objects,
            hp: 0
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
                    CommonActorsMessages::EnableCollider(switch) => {},

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
                            PlayerMessages::DealDamage(damage) => {
                                self.hp -= *damage as i32;
                            }
                            PlayerMessages::SendCreatePlayersDollMessageToPeers => {}
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

        for object in self.static_objects.iter_mut() {
            object.collider.init(id);
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
                    static_objects: Some(&mut self.static_objects),
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
                    static_objects: Some(&self.static_objects),
                    coloring_areas: None,
                    volume_areas: None,
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