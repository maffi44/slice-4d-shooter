use glam::{
    FloatExt, Vec4, Vec4Swizzles
};

use super::sdf_functions::{
    sd_box,
    sd_inf_box,
    sd_sph_box,
    sd_sphere
};

use crate::systems::actor::{
    Component,
    ActorID,
};
use crate::systems::engine_handle::{self, EngineHandle};
use crate::systems::transform::Position;

use super::super::transform::Transform;
use super::physics_system_data::StaticCollidersData;
// use crate::systems::static_obj::{StaticObject, self};


pub enum KinematicColliderMessages {
    
}

const THRESHOLD: f32 = 0.005;
const HALF_THRESHOLD: f32 = 0.00025;
const MAX_COLLIDING_ITERATIONS: u32 = 50;

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
            if distance_from_edge < THRESHOLD || is_pushed {
    
                // log::info!("BOUND");
    
                is_collide = true;
    
                let normal = get_normal(position, static_objects, stickiness);
                
                // log::info!("normal is {}", normal);
    
                // log::info!("translation len before reject is {}", translation.length());
    
                // log::info!("direction is {}", translation.normalize());
    
                if normal.dot(translation) < 0.0 {
    
                    let probable_transltaion_dir = translation.reject_from_normalized(normal).normalize();
    
                    let next_normal = get_normal(
                        position + probable_transltaion_dir * THRESHOLD,
                        static_objects,
                        stickiness
                    );
    
                    // log::info!("next normal is {}", normal);
    
                    let curvature_coefficient = next_normal.dot(probable_transltaion_dir);
    
                    // log::info!("curvature_coefficient is {}", curvature_coefficient);
    
                    if curvature_coefficient < 0.0 {
    
                        let prev_normal = get_normal(
                            position - probable_transltaion_dir * THRESHOLD,
                            static_objects,
                            stickiness
                        );
    
                        // log::info!("prev normal is {}", prev_normal);
    
                        if next_normal.dot(translation) < 0.0 {
                            let (bounce, new_friction) = get_bounce_and_friction(
                                position + probable_transltaion_dir * THRESHOLD,
                                static_objects,
                                stickiness
                            );

                            friction = friction.max(new_friction);

                            let current_velocity = self.current_velocity;

                            let coef = translation.length() / current_velocity.length();

                            let mut new_velocity = current_velocity.reject_from_normalized(next_normal);

                            let absorbed_velocity = new_velocity - current_velocity;

                            new_velocity += absorbed_velocity * bounce;

                            let diff = new_velocity - current_velocity;


                            self.current_velocity = new_velocity;


                            // let mut new_translation = translation.reject_from_normalized(next_normal);

                            // let absorbed_transaltion = new_translation - translation;

                            // new_translation += absorbed_transaltion * next_pos_bounce_coefficient;

                            translation += diff * coef;

                            // log::info!("direction after first bound is {}", translation.normalize());
    
                            // log::info!("translation len after first reject is {}", translation.length());
                        }
    
                        if prev_normal.dot(translation) < 0.0 {

                            let (bounce, new_friction) = get_bounce_and_friction(
                                position + probable_transltaion_dir * THRESHOLD,
                                static_objects,
                                stickiness
                            );

                            friction = friction.max(new_friction);

                            let current_velocity = self.current_velocity;
    
                            let coef = translation.length() / current_velocity.length();

                            let mut new_velocity = current_velocity.reject_from_normalized(prev_normal);

                            let absorbed_velocity = new_velocity - current_velocity;

                            new_velocity += absorbed_velocity * bounce;

                            let diff = new_velocity - current_velocity;

                            self.current_velocity = new_velocity;


                            // let mut new_translation = translation.reject_from_normalized(prev_normal);

                            // let absorbed_transaltion = new_translation - translation;

                            // new_translation += absorbed_transaltion * prev_pos_bounce_coefficient;

                            translation += diff * coef;

                            // log::info!("direction after second bound is {}", translation.normalize());
    
                            // log::info!("translation len after second reject is {}", translation.length());    
                        }
    
                    } else {

                        let (bounce, new_friction) = get_bounce_and_friction(
                            position + probable_transltaion_dir * THRESHOLD,
                            static_objects,
                            stickiness
                        );

                        friction = friction.max(new_friction);

                        let current_velocity = self.current_velocity;

                        let coef = translation.length() / current_velocity.length();
    
                        let mut new_velocity = current_velocity.reject_from_normalized(normal);

                        let absorbed_velocity = new_velocity - current_velocity;

                        new_velocity += absorbed_velocity * bounce;

                        let diff = new_velocity - current_velocity;

                        self.current_velocity = new_velocity;


                        // let mut new_translation = translation.reject_from_normalized(normal);

                        // let absorbed_transaltion = new_translation - translation;

                        translation += diff * coef;

                        // log::info!("direction after bound is {}", translation.normalize());
    
                        // log::info!("translation len after reject is {}", translation.length());
                    }
                }
    
            }
    
            let dist_on_try_move = get_dist(
                position + translation.clamp_length_max(collider_radius - THRESHOLD),
                static_objects,
                stickiness
            );
    
            if dist_on_try_move - collider_radius > 0.0 {
    
                position += translation.clamp_length_max(collider_radius - THRESHOLD);
    
                if translation.length() < collider_radius - THRESHOLD {

                    if is_collide {
                        self.current_velocity *= 1.0 - delta * friction.max(self.friction_on_air);
                    } else {
                        self.current_velocity *= 1.0 - delta*self.friction_on_air;
                    }
    
                    return position - start_position;
    
                } else {
                    translation = translation.clamp_length_max(
                        (translation.length() - (collider_radius - THRESHOLD)).max(0.0)
                    );
    
                    counter += 1;
                    continue;
                }
            }
    
            let mut translation_length = translation.length();
    
            let translation_dir = translation.normalize();
    
            let small_steps_counter = 0u32;
    
            while translation_length > 0.0 {
    
                // log::info!("SMALL STEPS");
    
                if small_steps_counter > MAX_COLLIDING_ITERATIONS {
                    panic!("More then max colliding small steps iterations");
                }
    
                let current_translation_len = translation_length.min(distance_from_edge.max(THRESHOLD));
    
                position += translation_dir * current_translation_len;
    
                translation_length -= current_translation_len;
    
                distance_from_edge = get_dist(position, static_objects, stickiness) - collider_radius;
    
                translation = translation_dir * translation_length;
                
                if distance_from_edge < 0.0 {
    
                    break;
                }
    
            }
    
            counter += 1;
        }

        if is_collide {
            self.current_velocity *= 1.0 - delta * friction.max(self.friction_on_air);
        } else {
            self.current_velocity *= 1.0 - delta*self.friction_on_air;
        }
    
        position - start_position
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
        pos -= normal * (distance_from_center - THRESHOLD);
        
        distance_from_center = get_dist(pos, static_objects, stickiness);

        counter += 1;
    }

    let mut distance_from_edge = distance_from_center - collider_radius;

    let mut counter = 0u32;

    while distance_from_edge < 0.0 {

        // if gets a many iterationsrunning maybe collider is stack between two walls
        // try to push it with big normals 
        if counter > MAX_COLLIDING_ITERATIONS / 2 {

            let pushing_counter = 0u32;

            while distance_from_edge < 0.0 {

                if pushing_counter > MAX_COLLIDING_ITERATIONS {
                    panic!("'move_collider_outside' More the max colliding iterations when overlapping (pushing) the object");
                }
            
                let normal = get_big_normal(pos, collider_radius, static_objects, stickiness);

                pos += normal * distance_from_edge.abs().max(THRESHOLD * 0.25);

                distance_from_edge = get_dist(pos, static_objects, stickiness) - collider_radius;
        
            }
            
            break;
        }
        is_collided = true;

        let normal = get_normal(pos, static_objects, stickiness);

        pos += normal * distance_from_edge.abs().max(THRESHOLD * 0.25);

        distance_from_edge = get_dist(pos, static_objects, stickiness) - collider_radius;
        
        // log::info!("'move_collider_outside' disatnce from th edge is {}", distance_from_edge);

        counter += 1;
    }

    (pos, is_collided)
}

#[inline]
fn smin(a: f32, b: f32, k: f32) -> f32
{
    let x = (b-a)/k;
    let g = 0.5*(x-(x*x+0.25).sqrt());
    return a + k * g;
}

const MAX_DIST: f32 = 700_f32;

#[inline]
fn get_dist(p: Vec4, static_objects: &StaticCollidersData, stickiness: f32) -> f32 {
    let mut d = MAX_DIST;

    for collider in static_objects.cubes.iter_stickiness() {
         d = smin(
            d,
            sd_box(p - collider.position.clone(), collider.size.clone()) - collider.roundness,
            stickiness
        );
    }
    for collider in static_objects.inf_w_cubes.iter_stickiness() {
        d = smin(
            d,
            sd_inf_box(p - collider.position.clone(), collider.size.xyz()) - collider.roundness,
            stickiness
        );
    }
    for collider in static_objects.spheres.iter_stickiness() {
        d = smin(
            d,
            sd_sphere(p - collider.position.clone(), collider.size.x) - collider.roundness,
            stickiness
        );
    }
    for collider in static_objects.sph_cubes.iter_stickiness() {
        d = smin(
            d,
            sd_sph_box(p - collider.position.clone(), collider.size.clone()) - collider.roundness,
            stickiness
        );
    }
    

    for collider in static_objects.cubes.iter_normal() {
         d = d.min(sd_box(p - collider.position.clone(), collider.size.clone()) - collider.roundness);
    }
    for collider in static_objects.inf_w_cubes.iter_normal() {
        d = d.min(sd_inf_box(p - collider.position.clone(), collider.size.xyz()) - collider.roundness);
    }
    for collider in static_objects.spheres.iter_normal() {
        d = d.min(sd_sphere(p - collider.position.clone(), collider.size.x) - collider.roundness);
    }
    for collider in static_objects.sph_cubes.iter_normal() {
        d = d.min(sd_sph_box(p - collider.position.clone(), collider.size.clone()) - collider.roundness);
    }


    let mut dd = MAX_DIST;

    for collider in static_objects.cubes.iter_neg_stickiness() {
        dd = smin(
            dd,
            sd_box(p - collider.position.clone(), collider.size.clone()) - collider.roundness,
            stickiness
        );
    }
    for collider in static_objects.inf_w_cubes.iter_neg_stickiness() {
            dd = smin(
            dd,
            sd_inf_box(p - collider.position.clone(), collider.size.xyz()) - collider.roundness,
            stickiness
        );
    }
    for collider in static_objects.spheres.iter_neg_stickiness() {
            dd = smin(
            dd,
            sd_sphere(p - collider.position.clone(), collider.size.x) - collider.roundness,
            stickiness
        );
    }
    for collider in static_objects.sph_cubes.iter_neg_stickiness() {
            dd = smin(
            dd,
            sd_sph_box(p - collider.position.clone(), collider.size.clone()) - collider.roundness,
            stickiness
        );
    }

    d = d.max(-dd);
    

    for collider in static_objects.cubes.iter_negative() {
        d = d.max(-(sd_box(p - collider.position.clone(), collider.size.clone()) - collider.roundness));
    }
    for collider in static_objects.inf_w_cubes.iter_negative() {
        d = d.max(-(sd_inf_box(p - collider.position.clone(), collider.size.xyz()) - collider.roundness));
    }
    for collider in static_objects.spheres.iter_negative() {
        d = d.max(-(sd_sphere(p - collider.position.clone(),collider.size.x) - collider.roundness));
    }
    for collider in static_objects.sph_cubes.iter_negative() {
        d = d.max(-(sd_sph_box(p - collider.position.clone(), collider.size.clone()) - collider.roundness));
    }

    return d;
}


fn get_bounce_and_friction(
    p: Vec4,
    static_objects: &StaticCollidersData,
    stickiness: f32,
) -> (f32, f32) {
    let mut d = MAX_DIST;

    let mut bounce_coeficient = 0.0;
    let mut friction = 0.0;

    for collider in static_objects.cubes.iter_stickiness() {
        let new_d = sd_box(p - collider.position.clone(), collider.size.clone()) - collider.roundness;
        
        let dd = smin(d, new_d, stickiness);
        
        let coef = ((dd - d) / (new_d - d)).clamp(0.0, 1.0);
        bounce_coeficient = bounce_coeficient.lerp(
            collider.bounce_rate,
            coef
        );
        friction = friction.lerp(
            collider.friction,
            coef
        );

        d = dd;
    }
    for collider in static_objects.inf_w_cubes.iter_stickiness() {
        let new_d = sd_inf_box(p - collider.position.clone(), collider.size.xyz()) - collider.roundness;

        let dd = smin(d, new_d, stickiness);
        
        let coef = ((dd - d) / (new_d - d)).clamp(0.0, 1.0);
        bounce_coeficient = bounce_coeficient.lerp(
            collider.bounce_rate,
            coef
        );
        friction = friction.lerp(
            collider.friction,
            coef
        );

        d = dd;
    }
    for collider in static_objects.spheres.iter_stickiness() {
        let new_d = sd_sphere(p - collider.position.clone(), collider.size.x) - collider.roundness;

        let dd = smin(d, new_d, stickiness);
        
        let coef = ((dd - d) / (new_d - d)).clamp(0.0, 1.0);
        bounce_coeficient = bounce_coeficient.lerp(
            collider.bounce_rate,
            coef
        );
        friction = friction.lerp(
            collider.friction,
            coef
        );

        d = dd;
    }
    for collider in static_objects.sph_cubes.iter_stickiness() {
        let new_d = sd_sph_box(p - collider.position.clone(), collider.size.clone()) - collider.roundness;

        let dd = smin(d, new_d, stickiness);
        
        let coef = ((dd - d) / (new_d - d)).clamp(0.0, 1.0);
        bounce_coeficient = bounce_coeficient.lerp(
            collider.bounce_rate,
            coef
        );
        friction = friction.lerp(
            collider.friction,
            coef
        );

        d = dd;
    }
    

    for collider in static_objects.cubes.iter_normal() {
        let new_d = sd_box(p - collider.position.clone(), collider.size.clone()) - collider.roundness;

        if new_d < d {
            bounce_coeficient = collider.bounce_rate;
            friction = collider.friction;

            d = new_d;
        };
    }
    for collider in static_objects.inf_w_cubes.iter_normal() {
        let new_d = sd_inf_box(p - collider.position.clone(), collider.size.xyz()) - collider.roundness;

        if new_d < d {
            bounce_coeficient = collider.bounce_rate;
            friction = collider.friction;

            d = new_d;
        };
    }
    for collider in static_objects.spheres.iter_normal() {
        let new_d = sd_sphere(p - collider.position.clone(), collider.size.x) - collider.roundness;

        if new_d < d {
            bounce_coeficient = collider.bounce_rate;
            friction = collider.friction;

            d = new_d;
        };
    }
    for collider in static_objects.sph_cubes.iter_normal() {
        let new_d = sd_sph_box(p - collider.position.clone(), collider.size.clone()) - collider.roundness;

        if new_d < d {
            bounce_coeficient = collider.bounce_rate;
            friction = collider.friction;

            d = new_d;
        };
    }

    (bounce_coeficient, friction)
}

#[inline]
fn get_normal(p: Vec4, static_objects: &StaticCollidersData, stickiness: f32) -> Vec4 {
    let a = p + Vec4::new(THRESHOLD, 0.000, 0.000, 0.000);
    let b = p + Vec4::new(-THRESHOLD, 0.000, 0.000,0.000);
    let c = p + Vec4::new(0.000, THRESHOLD, 0.000, 0.000);
    let d = p + Vec4::new(0.000, -THRESHOLD, 0.000, 0.000);
    let e = p + Vec4::new(0.000, 0.000, THRESHOLD, 0.000);
    let f = p + Vec4::new(0.000, 0.000, -THRESHOLD,0.000);
    let g = p + Vec4::new(0.000, 0.000, 0.000, THRESHOLD);
    let h = p + Vec4::new(0.000, 0.000, 0.000, -THRESHOLD);

    let fa = get_dist(a, static_objects, stickiness);
    let fb = get_dist(b, static_objects, stickiness);
    let fc = get_dist(c, static_objects, stickiness);
    let fd = get_dist(d, static_objects, stickiness);
    let fe = get_dist(e, static_objects, stickiness);
    let ff = get_dist(f, static_objects, stickiness);
    let fg = get_dist(g, static_objects, stickiness);
    let fh = get_dist(h, static_objects, stickiness);

    let normal = 
        Vec4::new(1.000, 0.000, 0.000, 0.000) * fa +
        Vec4::new(-1.000, 0.000, 0.000,0.000) * fb +
        Vec4::new(0.000, 1.000, 0.000, 0.000) * fc +
        Vec4::new(0.000, -1.000, 0.000, 0.000) * fd +
        Vec4::new(0.000, 0.000, 1.000, 0.000) * fe +
        Vec4::new(0.000, 0.000, -1.000,0.000) * ff +
        Vec4::new(0.000, 0.000, 0.000, 1.000) * fg +
        Vec4::new(0.000, 0.000, 0.000, -1.000) * fh;

    // if the collider is stuck in object's surface normal will be zero length
    // let's make some random normal in this case 
    if let Some(normal) = normal.try_normalize() {
        return normal;
    } else {
        return random_vec().normalize();
    }
}

#[inline]
fn get_big_normal(p: Vec4, size: f32, static_objects: &StaticCollidersData, stickiness: f32) -> Vec4 {
    let a = p + Vec4::new(size, 0.000, 0.000, 0.000);
    let b = p + Vec4::new(-size, 0.000, 0.000,0.000);
    let c = p + Vec4::new(0.000, size, 0.000, 0.000);
    let d = p + Vec4::new(0.000, -size, 0.000, 0.000);
    let e = p + Vec4::new(0.000, 0.000, size, 0.000);
    let f = p + Vec4::new(0.000, 0.000, -size,0.000);
    let g = p + Vec4::new(0.000, 0.000, 0.000, size);
    let h = p + Vec4::new(0.000, 0.000, 0.000, -size);

    let fa = get_dist(a, static_objects, stickiness);
    let fb = get_dist(b, static_objects, stickiness);
    let fc = get_dist(c, static_objects, stickiness);
    let fd = get_dist(d, static_objects, stickiness);
    let fe = get_dist(e, static_objects, stickiness);
    let ff = get_dist(f, static_objects, stickiness);
    let fg = get_dist(g, static_objects, stickiness);
    let fh = get_dist(h, static_objects, stickiness);

    let normal = 
        Vec4::new(1.000, 0.000, 0.000, 0.000) * fa +
        Vec4::new(-1.000, 0.000, 0.000,0.000) * fb +
        Vec4::new(0.000, 1.000, 0.000, 0.000) * fc +
        Vec4::new(0.000, -1.000, 0.000, 0.000) * fd +
        Vec4::new(0.000, 0.000, 1.000, 0.000) * fe +
        Vec4::new(0.000, 0.000, -1.000,0.000) * ff +
        Vec4::new(0.000, 0.000, 0.000, 1.000) * fg +
        Vec4::new(0.000, 0.000, 0.000, -1.000) * fh;

    // if the collider is stuck in object's surface normal will be zero length
    // let's make some random normal in this case 
    if let Some(normal) = normal.try_normalize() {
        return normal;
    } else {
        return random_vec().normalize();
    }
}


fn random_vec() -> Vec4 {
    let mut bytes : [u8;4] = [0,0,0,0];
    let res = getrandom::getrandom(&mut bytes);
    
    if let Err(err) = res {
        panic!("Can't make random f32 in random_vec fnction");
    }
    let x: f32 = f32::from_be_bytes(bytes);


    let mut bytes : [u8;4] = [0,0,0,0];
    let res = getrandom::getrandom(&mut bytes);
    
    if let Err(err) = res {
        panic!("Can't make random f32 in random_vec fnction");
    }
    let y: f32 = f32::from_be_bytes(bytes);


    let mut bytes : [u8;4] = [0,0,0,0];
    let res = getrandom::getrandom(&mut bytes);
    
    if let Err(err) = res {
        panic!("Can't make random f32 in random_vec fnction");
    }
    let z: f32 = f32::from_be_bytes(bytes);


    let mut bytes : [u8;4] = [0,0,0,0];
    let res = getrandom::getrandom(&mut bytes);
    
    if let Err(err) = res {
        panic!("Can't make random f32 in random_vec fnction");
    }
    let w: f32 = f32::from_be_bytes(bytes);
    

    let mut new_vec = Vec4::new(x, y, z, w);

    if !new_vec.length().is_normal() {
        new_vec = random_vec();
    }

    new_vec
}
