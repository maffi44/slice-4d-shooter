// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2026  Timofei Molokov



pub mod physics_system_data;
pub mod colliders_container;
pub mod kinematic_collider;
pub mod static_collider;
pub mod dynamic_collider;
pub mod area;
pub mod common_physical_functions;

use crate::{
    actor::{
        Actor,
        ActorID,
    },
    engine::world::{
        actors::Actors,
        messenger::Messenger,
        world_descriptor::WorldDescriptor,
    },
    engine::transform::Transform
};

use self::{
    area::Area,
    common_physical_functions::{
        get_dist,
        get_id_and_team,
        get_normal,
        THRESHOLD
    },
    dynamic_collider::PlayersDollCollider,
    kinematic_collider::KinematicCollider,
    physics_system_data::{
        Hit,
        PhysicsState
    }
};

use glam::Vec4;




const MAX_RAY_MARCHING_STEPS: usize = 500;

pub struct PhysicsSystem {
    physics_state: PhysicsState,
}


impl PhysicsSystem {
    pub fn new(descriptor: &WorldDescriptor) -> Self {
        
        let mut physics_state = PhysicsState::new();

        physics_state.update_level_static_info(descriptor);

        log::info!("physics system: static colliders data init");

        PhysicsSystem {
            physics_state,
        }
    }


    pub fn process_physics(
        &mut self,
        actors: &mut Actors,
        delta: f32,
        messenger: &mut Messenger,
    ) {

        self.physics_state.clear_temporal_colliders();


        let mut kinematic_colliders: Vec<(&mut Transform, &mut KinematicCollider)> = Vec::new();
        let mut areas: Vec<&mut Area> = Vec::new();
        let _dynamic_colliders: Vec<&mut PlayersDollCollider> = Vec::new();

        for (_, actor) in actors.iter_mut() {

            if let Some(physical_element) = actor.get_physical_element() {

                let id = physical_element.id;

                let transform = physical_element.transform;
                
                if let Some(area) = physical_element.area {

                    area.set_id(id);

                    area.set_frame_position(transform.get_position() + area.translation);
                    area.set_frame_size(transform.get_scale() * area.size);

                    areas.push(area);
                }

                if let Some(colliders) = physical_element.static_colliders {
                    for static_collider in colliders {

                        static_collider.set_id(id);

                        let mut temporal_static_collider = static_collider.clone();

                        temporal_static_collider.position += transform.get_position();
                        
                        temporal_static_collider.size *= transform.get_scale();
                        
                        self.physics_state.add_temporal_static_collider(temporal_static_collider);
                    }
                }

                if let Some(colliders) = physical_element.static_objects {
                    for static_object in colliders {

                        let mut temporal_static_collider = static_object.collider.clone();

                        temporal_static_collider.set_id(id);

                        temporal_static_collider.position += transform.get_position();
                        
                        temporal_static_collider.size *= transform.get_scale();
                        
                        self.physics_state.add_temporal_static_collider(temporal_static_collider);
                    }
                }

                if let Some((colliders, team)) = physical_element.dynamic_colliders {
                    for dynamic_collider in colliders {

                        dynamic_collider.set_id(id);
                        dynamic_collider.actors_team = team;
                        
                        // temporary solution to imitate kinematic physics
                        let mut temporal_dynamic_collider = dynamic_collider.clone();

                        temporal_dynamic_collider.position += transform.get_position();

                        self.physics_state.add_temporal_dynamic_collider(temporal_dynamic_collider);
                        
                        // dynamic_colliders.push(dynamic_collider);
                    }
                }
        
                if let Some((kinematic_collider, specific_transform)) = physical_element.kinematic_collider {

                    kinematic_collider.set_id(id);
                    
                    if specific_transform.is_some() {
                        kinematic_colliders.push((specific_transform.unwrap(), kinematic_collider));
                    } else {
                        kinematic_colliders.push((transform, kinematic_collider));
                    }
                }

            }
        }
        
        // Here should be code that finds potentially colliding kinematic and dynamic
        // colliders, combines them into groups and calculates physics within these groups

        // temp
        for (transform, kinematic_collider) in kinematic_colliders.iter_mut() {
            kinematic_collider.physics_tick(
                delta,
                &self.physics_state,
                transform,
                messenger,
            )
        }

        for area in areas.iter_mut() {
            area.physics_tick(&kinematic_colliders, messenger);
        }
    }


    
    pub fn ray_cast(&self, from: Vec4, direction: Vec4, distance: f32, excluded_id: Option<ActorID>) -> Option<Hit> {
        
        let mut i = 0_usize;

        let mut pos: Vec4 = from;

        let dir = direction.try_normalize().expect(
            "the direction vector in ray_cast can't be normalized"
        );

        let mut total_dist = 0.0;

        while i < MAX_RAY_MARCHING_STEPS {

            if total_dist > distance {
                break;
            }

            let dist = get_dist(pos, &self.physics_state, excluded_id);

            if dist < THRESHOLD {

                if let Some((hit_actors_id, hit_actors_team)) = get_id_and_team(pos, &self.physics_state)
                {
                    return Some(
                        Hit {
                            hit_point: pos,
                            hit_normal: get_normal(pos, &self.physics_state, excluded_id),
                            hit_actors_id,
                            hit_actors_team,
                        }
                    );
                }
                else
                {
                    return Some(
                            Hit {
                            hit_point: pos,
                            hit_normal: get_normal(pos, &self.physics_state, excluded_id),
                            hit_actors_id: None,
                            hit_actors_team: None,
                        }
                    );
                }
            }

            total_dist += dist;

            pos += dir * dist;

            i += 1;
        }

        None
    }


    /// The dynamic colliders (players) intersecting the cast sphere, except `excluded_id`.
    /// Returns a lazy iterator: no allocation per cast.
    pub fn sphere_cast_on_dynamic_colliders(
        &self,
        cast_sphere_pos: Vec4,
        cast_sphere_radius: f32,
        excluded_id: Option<ActorID>
    ) -> impl Iterator<Item = Hit> + '_ {

        self.physics_state.player_forms.iter().filter_map(move |dyn_sphere| {

            let vec_between_centers = cast_sphere_pos - dyn_sphere.position;

            if vec_between_centers.length() - (dyn_sphere.radius + cast_sphere_radius) >= 0.0 {
                return None;
            }

            if let (Some(excluded_id), Some(hited_actor_id)) = (excluded_id, dyn_sphere.get_id()) {
                if excluded_id == hited_actor_id {
                    return None;
                }
            }

            let hit_normal = vec_between_centers.normalize();

            let hit_point = dyn_sphere.position + (hit_normal * dyn_sphere.radius);

            Some(Hit {
                hit_point,
                hit_normal,
                hit_actors_id: dyn_sphere.get_id(),
                hit_actors_team: Some(dyn_sphere.actors_team),
            })
        })
    }
}
