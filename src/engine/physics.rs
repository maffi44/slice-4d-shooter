pub mod physics_system_data;
pub mod colliders_container;
pub mod kinematic_collider;
pub mod static_collider;
pub mod dynamic_collider;
pub mod area;
pub mod common_physical_functions;

use crate::{
    actor::Actor,
    transform::Transform,
    engine::{
        engine_handle::EngineHandle,
        world::World,
    }
};

use self::{
    area::Area,
    kinematic_collider::KinematicCollider,
    physics_system_data::{
        Hit,
        FrameCollidersBuffers,
        StaticCollidersData
    },
    common_physical_functions::{
        get_dist,
        get_normal,
        THRESHOLD,
    }
};

use glam::Vec4;




const MAX_RAY_MARCHING_STEPS: usize = 150;

pub struct PhysicsSystem {
    static_colliders_data: StaticCollidersData,
    frame_colliders_buffers: FrameCollidersBuffers,
}


impl PhysicsSystem {
    pub fn new(world: &World) -> Self {
        
        let static_colliders_data = StaticCollidersData::new(world);

        log::info!("physics system: static_colliders_data init");

        let frame_colliders_buffers = FrameCollidersBuffers::new();

        log::info!("physics system: frame_colliders_buffers init");

        PhysicsSystem {
            static_colliders_data,
            frame_colliders_buffers
        }
    }

    pub fn process_physics(
        &mut self,
        world: &mut World,
        delta: f32,
        engine_handle: &mut EngineHandle,
    ) {

        self.static_colliders_data.clear_temporal_static_colliders();

        self.frame_colliders_buffers.kinematic_colliders.clear();
        // self.frame_colliders_buffers.dynamic_colliders.clear();
        self.frame_colliders_buffers.areas.clear();

        // I use frame_colliders_buffers as a memory buffer in order
        // not to allocate memory dynamically each call process_physics().
        //
        // TODO: Change this unsafe functionality to use a regular Vec<&mut 'SomeCollider'>::new()
        //  with a custom allocator 

        let mut kinematic_colliders: Vec<(&mut Transform, &mut KinematicCollider)> = unsafe {
            std::mem::transmute_copy(&self.frame_colliders_buffers.kinematic_colliders)
        };
        let mut areas: Vec<&mut Area> = unsafe {
            std::mem::transmute_copy(&self.frame_colliders_buffers.areas)
        };
        // let mut dynamic_colliders: Vec<&mut DynamicCollider> = unsafe {
        //     std::mem::transmute_copy(&self.frame_colliders_buffers.dynamic_colliders)
        // };


        for (_, actor) in world.actors.iter_mut() {

            if let Some(physical_element) = actor.get_physical_element() {

                let transform = physical_element.transform;
                
                if let Some(area) = physical_element.area {

                    area.set_frame_position(transform.get_position() + area.translation);
                    area.set_frame_size(transform.get_scale() * area.size);

                    areas.push(area);
                }

                if let Some(colliders) = physical_element.static_colliders {
                    for static_collider in colliders {

                        let mut temporal_static_collider = static_collider.clone();

                        temporal_static_collider.position += transform.get_position();
                        
                        temporal_static_collider.size *= transform.get_scale();
                        
                        self.static_colliders_data.add_temporal_static_collider(temporal_static_collider);
                    }
                }

                if let Some(colliders) = physical_element.static_objects {
                    for static_object in colliders {

                        let mut temporal_static_collider = static_object.collider.clone();

                        temporal_static_collider.position += transform.get_position();
                        
                        temporal_static_collider.size *= transform.get_scale();
                        
                        self.static_colliders_data.add_temporal_static_collider(temporal_static_collider);
                    }
                }
                
                if let Some(kinematic_collider) = physical_element.kinematic_collider {
                    kinematic_colliders.push((transform, kinematic_collider));
                }


                // if let Some(colliders) = colliders_container.dynamic_colliders {
                //     for dynamic_collider in colliders {
                        
                //         dynamic_colliders.push(dynamic_collider);
                //     }
                // }

            }
        }
        
        // Here need to be some code to find potential colliding kinematic and dynamic
        // colliders to combine them into groups and calculate physics in these groups

        // temp
        for (transform, kinematic_collider) in kinematic_colliders.iter_mut() {
            kinematic_collider.physics_tick(
                delta,
                &self.static_colliders_data,
                transform,
                engine_handle,
            )
        }

        for area in areas.iter_mut() {
            area.physics_tick(&kinematic_colliders, engine_handle);
        }

        std::mem::forget(kinematic_colliders);
        std::mem::forget(areas);
        // std::mem::forget(dynamic_colliders);

        self.frame_colliders_buffers.kinematic_colliders.clear();
        self.frame_colliders_buffers.areas.clear();
        // self.frame_colliders_buffers.dynamic_colliders.clear();
    }

    pub fn ray_cast(&self, from: Vec4, direction: Vec4, distance: f32) -> Option<Hit> {
        
        let mut i = 0_usize;

        let mut p = from;

        let dir = direction.try_normalize().expect(
            "Direction vector in ray_cast function cannot be normalized"
        );

        while i < MAX_RAY_MARCHING_STEPS {
            let dist = get_dist(p, &self.static_colliders_data);

            if dist < THRESHOLD {
                return Some(
                        Hit {
                        hit_point: p,
                        hit_normal: get_normal(p, &self.static_colliders_data),
                        hited_players_id: None,
                    }
                );
            }

            p += dir * dist;

            i += 1;
        }

        None
    }
}