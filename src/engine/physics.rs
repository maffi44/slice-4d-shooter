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
    dynamic_collider::PlayersDollCollider,
    physics_system_data::{
        Hit,
        FrameCollidersBuffers,
        PhysicsState
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
    physics_state: PhysicsState,
    frame_colliders_buffers: FrameCollidersBuffers,
}


impl PhysicsSystem {
    pub fn new(world: &World) -> Self {
        
        let physics_state = PhysicsState::new(world);

        log::info!("physics system: static_colliders_data init");

        let frame_colliders_buffers = FrameCollidersBuffers::new();

        log::info!("physics system: frame_colliders_buffers init");

        PhysicsSystem {
            physics_state,
            frame_colliders_buffers
        }
    }

    pub fn process_physics(
        &mut self,
        world: &mut World,
        delta: f32,
        engine_handle: &mut EngineHandle,
    ) {

        self.physics_state.clear_temporal_colliders();

        self.frame_colliders_buffers.kinematic_colliders.clear();
        self.frame_colliders_buffers.dynamic_colliders.clear();
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
        let mut dynamic_colliders: Vec<&mut PlayersDollCollider> = unsafe {
            std::mem::transmute_copy(&self.frame_colliders_buffers.dynamic_colliders)
        };


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
                        
                        self.physics_state.add_temporal_static_collider(temporal_static_collider);
                    }
                }

                if let Some(colliders) = physical_element.static_objects {
                    for static_object in colliders {

                        let mut temporal_static_collider = static_object.collider.clone();

                        temporal_static_collider.position += transform.get_position();
                        
                        temporal_static_collider.size *= transform.get_scale();
                        
                        self.physics_state.add_temporal_static_collider(temporal_static_collider);
                    }
                }

                if let Some(colliders) = physical_element.dynamic_colliders {
                    for dynamic_collider in colliders {
                        
                        // temporary solution to immitate kinematic physic
                        let mut temporal_dynamoc_collider = dynamic_collider.clone();

                        temporal_dynamoc_collider.position += transform.get_position();

                        self.physics_state.add_temporal_dynamic_collider(temporal_dynamoc_collider);
                        
                        // dynamic_colliders.push(dynamic_collider);
                    }
                }
        
                if let Some(kinematic_collider) = physical_element.kinematic_collider {
                    kinematic_colliders.push((transform, kinematic_collider));
                }

            }
        }
        
        // Here need to be some code to find potential colliding kinematic and dynamic
        // colliders to combine them into groups and calculate physics in these groups

        // temp
        for (transform, kinematic_collider) in kinematic_colliders.iter_mut() {
            kinematic_collider.physics_tick(
                delta,
                &self.physics_state,
                transform,
                engine_handle,
            )
        }

        for area in areas.iter_mut() {
            area.physics_tick(&kinematic_colliders, engine_handle);
        }

        std::mem::forget(kinematic_colliders);
        std::mem::forget(areas);
        std::mem::forget(dynamic_colliders);

        self.frame_colliders_buffers.kinematic_colliders.clear();
        self.frame_colliders_buffers.areas.clear();
        self.frame_colliders_buffers.dynamic_colliders.clear();
    }

    pub fn ray_cast(&self, from: Vec4, direction: Vec4, distance: f32) -> Option<Hit> {
        
        let mut i = 0_usize;

        let mut pos: Vec4 = from;

        let dir = direction.try_normalize().expect(
            "Direction vector in ray_cast function cannot be normalized"
        );

        let mut total_dist = 0.0;

        while i < MAX_RAY_MARCHING_STEPS {

            if total_dist > distance {
                break;
            }

            let dist = get_dist(pos, &self.physics_state);

            if dist < THRESHOLD {
                return Some(
                        Hit {
                        hit_point: pos,
                        hit_normal: get_normal(pos, &self.physics_state),
                        hited_actors_id: None,
                    }
                );
            }

            total_dist += dist;

            pos += dir * dist;

            i += 1;
        }

        None
    }


    pub fn sphere_cast(sphere_pos: Vec4, sphere_radius: f32) {

    }
}