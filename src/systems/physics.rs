pub mod physics_system_data;
pub mod colliders_container;
pub mod kinematic_collider;
pub mod static_collider;
pub mod dynamic_collider;
pub mod area;

use self::{
    kinematic_collider::KinematicCollider,
    dynamic_collider::DynamicCollider,
    static_collider::StaticCollider,
    area::Area,
    physics_system_data::{
        FrameCollidersBuffers,
        StaticCollidersData
    }
};

use super::{
    actor::Actor, world::World
};

pub struct PhysicsSystem {
    static_colliders_data: StaticCollidersData,
    frame_colliders_buffers: FrameCollidersBuffers,
}


impl PhysicsSystem {
    pub fn new(world: &World) -> Self {
        
        let static_colliders_data = StaticCollidersData::new(world);
        let frame_colliders_buffers = FrameCollidersBuffers::new();

        PhysicsSystem {
            static_colliders_data,
            frame_colliders_buffers
        }
    }

    pub fn process_physics(&mut self, world: &mut World, delta: f32) {

        self.frame_colliders_buffers.kinematic_colliders.clear();
        self.frame_colliders_buffers.dynamic_colliders.clear();
        self.frame_colliders_buffers.areas.clear();

        // I use frame_colliders_buffers as a memory buffer in order
        // not to allocate memory dynamically each call process_physics().
        //
        // TODO: Change this unsafe functionality to use a regular Vec<&mut 'SomeCollider'>::new()
        //  with a custom allocator 

        let mut kinematic_colliders: Vec<&mut KinematicCollider> = unsafe {
            std::mem::transmute_copy(&self.frame_colliders_buffers.kinematic_colliders)
        };
        let mut dynamic_colliders: Vec<&mut DynamicCollider> = unsafe {
            std::mem::transmute_copy(&self.frame_colliders_buffers.dynamic_colliders)
        };
        let mut areas: Vec<&mut Area> = unsafe {
            std::mem::transmute_copy(&self.frame_colliders_buffers.areas)
        };

        for (_, actor) in world.actors.iter_mut() {

            if let Some(colliders_container) = actor.get_colliders_container() {

                if let Some(kinematic_collider) = colliders_container.kinematic_collider {
                    kinematic_colliders.push(kinematic_collider);
                }
                
                if let Some(area) = colliders_container.area {
                    areas.push(area);
                }

                if let Some(colliders) = colliders_container.dynamic_colliders {
                    for dynamic_collider in colliders {
                        
                        dynamic_colliders.push(dynamic_collider);
                    }
                }

                if let Some(colliders) = colliders_container.static_colliders {
                    for static_collider in colliders {
                        
                        self.static_colliders_data.add_temporal_static_collider(static_collider.clone());
                    }
                }
            }
        }
        
        // Here need to be some code to find potential colliding kinematic and dynamic
        // colliders to combine them into groups and calculate physics in these groups

        // temp
        for kinematic_collider in kinematic_colliders {
            kinematic_collider.physics_tick(delta, &self.static_colliders_data)
        }

        for area in areas {
            area.physic_tick(kinematic_colliders);
        }

        self.static_colliders_data.clear_temporal_static_colliders();

        std::mem::forget(kinematic_colliders);
        std::mem::forget(dynamic_colliders);
        std::mem::forget(areas);

        self.frame_colliders_buffers.kinematic_colliders.clear();
        self.frame_colliders_buffers.dynamic_colliders.clear();
        self.frame_colliders_buffers.areas.clear();
    }
}