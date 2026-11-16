// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2026  Timofei Molokov



use crate::{
    engine::transform::Transform,
    actor::{
        ActorID, Message, MessageType, PhysicsMessages
    },
    engine::{
        world::messenger::Messenger,
        world::physics::{
            physics_system_data::ShapeType,
            kinematic_collider::KinematicCollider,
            common_physical_functions::{
                sd_box,
                sd_inf_box,
                sd_sph_box,
                sd_sphere
            },
        },
    },
};

use glam::{Vec4, Vec4Swizzles};


   
#[derive(Clone)]
pub enum AreaMessage {
    ActorIsContainedInsideArea(ActorID),
    #[allow(dead_code)] // payload kept for message consumers
    ActorEscapeArea(ActorID),
    ActorEnterArea(ActorID),
}

pub struct Area {
    frame_position: Vec4,
    frame_size: Vec4,
    
    pub size: Vec4,
    pub translation: Vec4,
    shape_type: ShapeType,
    actor_id: Option<ActorID>,
    intersected_actor_ids: Vec<ActorID>,
}

impl Area {

    pub fn set_id(&mut self, id: ActorID)
    {
        self.actor_id = Some(id);
    }

    pub fn clear_containing_colliders_list(&mut self)
    {
        self.intersected_actor_ids.clear();
    } 

    pub fn set_frame_position(&mut self, frame_position: Vec4) {
        self.frame_position = frame_position;
    }

    pub fn set_frame_size(&mut self, frame_size: Vec4) {
        self.frame_size = frame_size;
    }

    pub fn new(translation: Vec4, shape_type: ShapeType, size: Vec4) -> Self {
        Area {
            frame_position: Vec4::ZERO,
            frame_size: Vec4::ZERO,

            translation,
            shape_type,
            size,
            actor_id: None,
            intersected_actor_ids: Vec::with_capacity(10),
        }
    }

    pub fn physics_tick(
        &mut self,
        kinematic_colliders: &Vec<(&mut Transform, &mut KinematicCollider)>,
        messenger: &mut Messenger,
    ) {
        for (kinematic_collider_transform, kinematic_collider) in kinematic_colliders {

            let collider_id = kinematic_collider.get_id().expect(
                "the kinematic collider has no actor id"
            );

            let is_intersecting = self
                .kinematic_collider_intersects_area(&kinematic_collider_transform, &kinematic_collider);
            
            let was_intersected = self
                .kinematic_collider_was_intersected_with_area(collider_id);

            if was_intersected {
                if !is_intersecting {

                    // Kinematic collider escaped the area

                    let message_content = MessageType::PhysicsMessages(
                        PhysicsMessages::AreaMessage(
                            AreaMessage::ActorEscapeArea(collider_id)
                        )
                    );

                    let message = Message {
                        from: self.actor_id.expect("the area was not initialized"),
                        remote_sender: false,
                        message: message_content,
                    };

                    messenger.send_direct_message(
                        self.actor_id.expect("the area was not initialized"),
                        message
                    )
                }
            }

            if !was_intersected {
                if is_intersecting {
                    // Kinematic collider entered the area

                    let message_content = MessageType::PhysicsMessages(
                        PhysicsMessages::AreaMessage(
                            AreaMessage::ActorEnterArea(collider_id)
                        )
                    );

                    let message = Message {
                        from: self.actor_id.expect("the area was not initialized"),
                        remote_sender: false,
                        message: message_content,
                    };

                    messenger.send_direct_message(
                        self.actor_id.expect("the area was not initialized"),
                        message
                    )
                }
            }

            self.update_intersected_actor_ids(collider_id, was_intersected, is_intersecting);
        }

        for intersected_actor_id in self.intersected_actor_ids.iter() {
            
            let message_content = MessageType::PhysicsMessages(
                PhysicsMessages::AreaMessage(
                    AreaMessage::ActorIsContainedInsideArea(*intersected_actor_id)
                )
            );

            let message = Message {
                from: self.actor_id.expect("the area was not initialized"),
                remote_sender: false,
                message: message_content,
            };

            messenger.send_direct_message(
                self.actor_id.expect("the area was not initialized"),
                message
            )
        }
    }

    fn kinematic_collider_intersects_area(
        &self,
        kinematic_collider_transform: &Transform,
        kinematic_collider: &KinematicCollider
    ) -> bool {
        let kinematic_collider_position = kinematic_collider_transform.get_position();

        let distance_from_collider_center = match self.shape_type {
            ShapeType::Cube => {
                sd_box(
                    kinematic_collider_position - self.frame_position,
                    self.frame_size
                )
            },
            ShapeType::Sphere => {
                sd_sphere(
                    kinematic_collider_position - self.frame_position,
                    self.frame_size.x
                )
            },
            ShapeType::SphCube => {
                sd_sph_box(
                    kinematic_collider_position - self.frame_position,
                    self.frame_size
                )
            },
            ShapeType::CubeInfW => {
                sd_inf_box(
                    kinematic_collider_position - self.frame_position,
                    self.frame_size.xyz()
                )
            }
        };

        if distance_from_collider_center <= kinematic_collider.get_collider_radius() {
            
            return true
            
        } else {

            return false
        }
    }


    fn kinematic_collider_was_intersected_with_area(
        &self,
        collider_id: ActorID,
    ) -> bool {
        
        let mut contain = false;

        for intersected_actor_id in self.intersected_actor_ids.iter() {
            if *intersected_actor_id == collider_id {
                contain = true
            }
        }
    
        contain
    }


    fn find_kinematic_collider_index(
        &self,
        collider_id: ActorID,
    ) -> Option<usize> {
        
        let mut counter: usize = 0;
                
        for intersected_actor_id in self.intersected_actor_ids.iter() {
            if *intersected_actor_id == collider_id {
                return Some(counter);
            }

            counter += 1;
        }
    
        None
    }


    fn update_intersected_actor_ids(&mut self, id: ActorID, was_intersected: bool, is_intersecting: bool) {
        
        if was_intersected && is_intersecting {
            return;
        }

        if !was_intersected && !is_intersecting {
            return;
        }

        if is_intersecting {
            self.intersected_actor_ids.push(id);
        }

        if was_intersected {
            let collider_index = {
                self.find_kinematic_collider_index(id)
                    .expect("area physics tick: can't find the actor's id among the intersected actors")
            };

            self.intersected_actor_ids.remove(collider_index);
        }
    }
}