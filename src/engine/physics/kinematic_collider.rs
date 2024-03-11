use crate::{
    transform::Transform,
    actor::{
        Component,
        ActorID,
    },
    engine::{
        engine_handle::EngineHandle,
        physics::{
            physics_system_data::StaticCollidersData,
            common_physical_functions::{
                get_dist,
                get_normal,
                get_big_normal,
                get_bounce_and_friction,
            },
        },
    },
};

use glam::Vec4;



pub enum KinematicColliderMessages {
    
}

const MIN_STEP: f32 = 0.009;
const MAX_COLLIDING_ITERATIONS: u32 = 50;
const MAX_SMALL_STEPS_COLLIDING_ITERATIONS: u32 = 150;

pub struct KinematicCollider {
    pub is_enable: bool,
    collider_radius: f32,
    max_speed: f32,
    max_accel: f32,
    friction_on_air: f32,
    // friction_on_ground: f32,
    wish_direction: Vec4,
    movment_mult: f32,
    current_velocity: Vec4,
    forces: Vec<Vec4>,
    pub is_on_ground: bool,
    actors_id: Option<ActorID>,
}

impl Component for KinematicCollider {
    fn init(&mut self, id: ActorID) {
        self.actors_id = Some(id);
    }

    fn get_id(&self) -> Option<ActorID> {
        let id = self.actors_id.expect("Component was not initialised");

        Some(id)
    }
}

static mut DEBUG_ITERATION_COUTER: u32 = 0u32;
static mut DEBUG_MAX_ITERATIONS: u32 = 0u32;

impl KinematicCollider {
    pub fn new(
        max_speed: f32,
        max_accel: f32,
        collider_radius: f32,
        friction_on_air: f32,
        // friction_on_ground: f32,
    ) -> Self {
        KinematicCollider {
            is_enable: true,
            collider_radius,
            max_speed,
            max_accel,
            friction_on_air,
            // friction_on_ground,
            wish_direction: Vec4::ZERO,
            movment_mult: 1.0,
            current_velocity: Vec4::ZERO,
            forces: Vec::with_capacity(10),
            is_on_ground: false,
            actors_id: None,
        }
    }

    pub fn get_collider_radius(&self) -> f32 {
        self.collider_radius
    }
    
    pub fn set_wish_direction(&mut self, wish_direction: Vec4, movement_mult: f32) {
        self.wish_direction = wish_direction;
        self.movment_mult = movement_mult;
    }

    pub fn add_force(&mut self, force: Vec4) {
        self.forces.push(force);
    }

    pub fn physics_tick(
        &mut self,
        delta: f32,
        static_objects: &StaticCollidersData,
        shapes_stickiness: f32,
        transform: &mut Transform,
        engine_handle: &mut EngineHandle,
    ) {
        self.is_on_ground = false;

        while let Some(force) = self.forces.pop() {
            self.current_velocity += force;
        }

        if self.is_enable {

            if self.wish_direction.length().is_normal() {
                // self.wish_direction = self.wish_direction.normalize();
    
                let current_speed_in_wishdir = self.current_velocity.dot(self.wish_direction);
    
                let speed = self.max_speed - current_speed_in_wishdir;
    
                let add_speed = 0.0_f32.max(speed.min(self.max_accel * delta));
    
                self.current_velocity += self.wish_direction * (add_speed * self.movment_mult);
    
            }

            let position_increment = self.move_collider(
                delta,
                static_objects,
                transform.get_position(),
                shapes_stickiness
            );

            transform.increment_position(position_increment);

            // if position_increment.length().is_normal() {
            //     self.current_velocity = self.current_velocity.project_onto_normalized(position_increment.normalize());
            // }

            //check if collider staying on the ground
            let bottom_position = transform.get_position() - ((self.collider_radius * 0.1) * Vec4::Y);

            if get_dist(bottom_position, static_objects, shapes_stickiness) < self.collider_radius * 0.95 {
                self.is_on_ground = true;
            }

        } else {

            self.current_velocity = Vec4::ZERO;
        }

        self.wish_direction = Vec4::ZERO;
    }


    
    fn move_collider(
        &mut self,
        delta: f32,
        static_objects: &StaticCollidersData,
        start_position: Vec4,
        stickiness: f32,
    ) -> Vec4 {

        self.fix_currnet_velocity();

        #[cfg(debug_assertions)]
        unsafe {
            log::warn!("MAX ITERATIONS COUNTER IS {}", DEBUG_MAX_ITERATIONS);

            DEBUG_MAX_ITERATIONS = DEBUG_MAX_ITERATIONS.max(DEBUG_ITERATION_COUTER);

            DEBUG_ITERATION_COUTER = 0;
        }

        let mut position = start_position;

        let mut translation = self.current_velocity * delta;

        let start_translation = translation;

        let collider_radius = self.collider_radius;
    
        let mut is_collide = false;
    
        let mut friction: f32 = 0.0;
    
        // pushing out the kinematic collider if it is stuck inside the object
        // at beginning of physics frame.
        // It is only possible if some static object moved in the previus frame
        // (unless the kinematic collider was disabled in previous frame),
        // and if this happaned, we will add the collision force to the kinematic collider 
        let (new_pos, is_pushed) = move_collider_outside(
            position,
            collider_radius,
            static_objects,
            stickiness
        );

        if is_pushed {
            is_collide = is_pushed;

            let collide_translation =  new_pos - position;

            self.current_velocity += collide_translation * 1.0/delta;

            position = new_pos;
        }
    
    
        // log::info!("start position is {}", start_position);
        
        let mut counter = 0u32;
    
        while translation.length().is_normal() {
    
            // log::info!("ITERATION number {}", counter);
    
            if counter > MAX_COLLIDING_ITERATIONS {
                panic!("More then max colliding iterations");
            }
    
            // if collider stuck inside object let's push it out
            let (new_pos, is_pushed) = move_collider_outside(position, collider_radius, static_objects, stickiness);
    
            is_collide = is_pushed;
    
            position = new_pos;
    
            // get distance from edge of the object to the nearest object
            let mut distance_from_edge = get_dist(position, static_objects, stickiness) - collider_radius;
    
            // log::info!("distance from the edge is {}", distance_from_edge);
    
            // bound if collide
            if distance_from_edge < MIN_STEP {
    
                // log::info!("BOUND");
    
                is_collide = true;
    
                let normal = get_normal(position, static_objects, stickiness);
                
                // log::info!("normal is {}", normal);
    
                // log::info!("translation len before reject is {}", translation.length());
    
                // log::info!("direction is {}", translation.normalize());
    
                if normal.dot(translation) < 0.0 {
    
                    let probable_transltaion_dir = translation.reject_from_normalized(normal).normalize();
    
                    let next_normal = get_normal(
                        position + probable_transltaion_dir * MIN_STEP,
                        static_objects,
                        stickiness
                    );
    
                    // log::info!("next normal is {}", normal);
    
                    let curvature_coefficient = next_normal.dot(probable_transltaion_dir);
    
                    // log::info!("curvature_coefficient is {}", curvature_coefficient);
    
                    if curvature_coefficient < 0.0 {
    
                        let prev_normal = get_normal(
                            position - probable_transltaion_dir * MIN_STEP,
                            static_objects,
                            stickiness
                        );
    
                        // log::info!("prev normal is {}", prev_normal);
    
                        if next_normal.dot(translation) < 0.0 {
                            let (bounce, new_friction) = get_bounce_and_friction(
                                position + probable_transltaion_dir * MIN_STEP,
                                static_objects,
                                stickiness
                            );

                            friction = friction.max(new_friction);

                            let current_velocity = self.current_velocity;

                            let current_velocity_length = current_velocity.length();

                            if current_velocity_length.is_normal() {

                                let coef = translation.length() / current_velocity_length;
    
                                let mut new_velocity = current_velocity.reject_from_normalized(next_normal);
    
                                let absorbed_velocity = new_velocity - current_velocity;
    
                                new_velocity += absorbed_velocity * bounce;
    
                                let diff = new_velocity - current_velocity;
    
                                self.current_velocity = new_velocity;
                                
                                translation += diff * coef;
                            }



                            // let mut new_translation = translation.reject_from_normalized(next_normal);

                            // let absorbed_transaltion = new_translation - translation;

                            // new_translation += absorbed_transaltion * next_pos_bounce_coefficient;


                            // log::info!("direction after first bound is {}", translation.normalize());
    
                            // log::info!("translation len after first reject is {}", translation.length());
                        }
    
                        if prev_normal.dot(translation) < 0.0 {

                            let (bounce, new_friction) = get_bounce_and_friction(
                                position - probable_transltaion_dir * MIN_STEP,
                                static_objects,
                                stickiness
                            );

                            friction = friction.max(new_friction);

                            let current_velocity = self.current_velocity;
    
                            let current_velocity_length = current_velocity.length();

                            if current_velocity_length.is_normal() {

                                let coef = translation.length() / current_velocity_length;
    
                                let mut new_velocity = current_velocity.reject_from_normalized(prev_normal);
    
                                let absorbed_velocity = new_velocity - current_velocity;
    
                                new_velocity += absorbed_velocity * bounce;
    
                                let diff = new_velocity - current_velocity;
    
                                self.current_velocity = new_velocity;
                                
                                translation += diff * coef;
                            }

                            // log::info!("direction after second bound is {}", translation.normalize());
    
                            // log::info!("translation len after second reject is {}", translation.length());    
                        }
    
                    } else {

                        let (bounce, new_friction) = get_bounce_and_friction(
                            position,
                            static_objects,
                            stickiness
                        );

                        friction = friction.max(new_friction);

                        let current_velocity = self.current_velocity;

                        let current_velocity_length = current_velocity.length();

                        if current_velocity_length.is_normal() {

                            let coef = translation.length() / current_velocity_length;

                            let mut new_velocity = current_velocity.reject_from_normalized(normal);

                            let absorbed_velocity = new_velocity - current_velocity;

                            new_velocity += absorbed_velocity * bounce;

                            let diff = new_velocity - current_velocity;

                            self.current_velocity = new_velocity;
                            
                            translation += diff * coef;
                        }

                        // log::info!("direction after bound is {}", translation.normalize());
    
                        // log::info!("translation len after reject is {}", translation.length());
                    }
                }
    
            }
    
            let dist_on_try_move = get_dist(
                position + translation.clamp_length_max(collider_radius - MIN_STEP),
                static_objects,
                stickiness
            );
    
            if dist_on_try_move - collider_radius > 0.0 {
    
                position += translation.clamp_length_max(collider_radius - MIN_STEP);
    
                if translation.length() < collider_radius - MIN_STEP {

                    if is_collide {
                        self.current_velocity *= 1.0 - delta * friction.max(self.friction_on_air);
                    } else {
                        self.current_velocity *= 1.0 - delta*self.friction_on_air;
                    }

                    self.fix_currnet_velocity();

                    return position - start_position;
    
                } else {
                    translation = translation.clamp_length_max(
                        (translation.length() - (collider_radius - MIN_STEP)).max(0.0)
                    );
    
                    counter += 1;
                    continue;
                }
            }
    
            let mut translation_length = translation.length();
    
            let translation_dir = translation.normalize();
    
            let mut small_steps_counter = 0u32;
    
            while translation_length > 0.0 {
    
                // log::info!("SMALL STEPS");
    
                if small_steps_counter > MAX_SMALL_STEPS_COLLIDING_ITERATIONS {

                    #[cfg(debug_assertions)]
                    unsafe {
                        log::warn!("ALL ITERATIONS COUNTER IS {}", DEBUG_ITERATION_COUTER);
                    }

                    panic!("More then max colliding small steps iterations");
                }
    
                let current_translation_len = translation_length.min(distance_from_edge.max(MIN_STEP));
    
                position += translation_dir * current_translation_len;
    
                translation_length -= current_translation_len;
    
                distance_from_edge = get_dist(position, static_objects, stickiness) - collider_radius;
    
                translation = translation_dir * translation_length;
                
                if distance_from_edge < 0.0 {
    
                    break;
                }

                small_steps_counter += 1;

                #[cfg(debug_assertions)]
                unsafe {
                    DEBUG_ITERATION_COUTER += 1;
                }
    
            }
    
            counter += 1;

            #[cfg(debug_assertions)]
            unsafe {
                DEBUG_ITERATION_COUTER += 1;
            }
        }

        if is_collide {
            self.current_velocity *= 1.0 - delta * friction.max(self.friction_on_air);
        } else {
            self.current_velocity *= 1.0 - delta*self.friction_on_air;
        }

        self.fix_currnet_velocity();
    
        position - start_position
    }



    #[inline]
    fn fix_currnet_velocity(&mut self) {
        if !self.current_velocity.is_finite() {
            if !self.current_velocity.x.is_normal() {
                self.current_velocity.x = 0.0;  
            };
            if !self.current_velocity.y.is_normal() {
                self.current_velocity.y = 0.0;  
            };
            if !self.current_velocity.z.is_normal() {
                self.current_velocity.z = 0.0;  
            };
            if !self.current_velocity.w.is_normal() {
                self.current_velocity.w = 0.0;  
            };
        }
    }
}



#[inline]
fn move_collider_outside(
    position: Vec4,
    collider_radius: f32,
    static_objects: &StaticCollidersData,
    stickiness: f32
) -> (Vec4, bool) {

    let mut pos = position;

    let mut is_collided = false;
    
    let mut distance_from_center = get_dist(pos, static_objects, stickiness);

    let mut counter = 0u32;

    while distance_from_center < 0.0 {
        if counter > MAX_COLLIDING_ITERATIONS {
            panic!("'move_collider_outside' More the max colliding iterations inside the object")
        }
        is_collided = true;

        let normal = get_normal(pos, static_objects, stickiness);
        pos -= normal * (distance_from_center - MIN_STEP);
        
        distance_from_center = get_dist(pos, static_objects, stickiness);

        #[cfg(debug_assertions)]
        unsafe {
            DEBUG_ITERATION_COUTER += 1;
        }

        counter += 1;
    }

    let mut distance_from_edge = distance_from_center - collider_radius;

    let mut counter = 0u32;

    while distance_from_edge < 0.0 {

        // if gets a many iterationsrunning maybe collider is stack between two walls
        // try to push it with big normals 
        if counter > MAX_COLLIDING_ITERATIONS / 2 {

            let mut pushing_counter = 0u32;

            while distance_from_edge < 0.0 {

                if pushing_counter > MAX_COLLIDING_ITERATIONS {
                    panic!("'move_collider_outside' More the max colliding iterations when overlapping (pushing) the object");
                }
            
                let normal = get_big_normal(pos, collider_radius, static_objects, stickiness);

                pos += normal * distance_from_edge.abs().max(MIN_STEP * 0.25);

                distance_from_edge = get_dist(pos, static_objects, stickiness) - collider_radius;
                
                pushing_counter += 1;
            }
            
            break;
        }
        is_collided = true;

        let normal = get_normal(pos, static_objects, stickiness);

        pos += normal * distance_from_edge.abs().max(MIN_STEP * 0.25);

        distance_from_edge = get_dist(pos, static_objects, stickiness) - collider_radius;
        
        // log::info!("'move_collider_outside' disatnce from th edge is {}", distance_from_edge);

        #[cfg(debug_assertions)]
        unsafe {
            DEBUG_ITERATION_COUTER += 1;
        }

        counter += 1;
    }

    (pos, is_collided)
}




