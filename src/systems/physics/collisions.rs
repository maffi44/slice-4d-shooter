use glam::{
    Vec4, Vec3, Vec2, Vec4Swizzles
};
use web_sys::console::dir;

use super::{super::transform::Transform, StaticObjectsData};

use crate::systems::static_obj::{StaticObject, self};

pub enum Collision<'a> {
    Static(&'a mut StaticCollision),
    Dynamic(&'a mut DynamicCollision),
    StaticArea(&'a mut StaticArea),
    DynamicArea(&'a mut DynamicArea),
}

pub struct StaticCollision {}

pub struct DynamicCollision {
    pub is_enable: bool,
    pub transform: Transform,
    collider_radius: f32,
    max_speed: f32,
    max_accel: f32,
    wish_direction: Vec4,
    movment_mult: f32,
    current_velocity: Vec4,
    forces: Vec<Vec4>,
    pub is_on_ground: bool,
}

impl DynamicCollision {
    pub fn new(transform: Transform, max_speed: f32, max_accel: f32, collider_radius: f32) -> Self {
        DynamicCollision {
            is_enable: true,
            transform,
            collider_radius,
            max_speed,
            max_accel,
            wish_direction: Vec4::ZERO,
            movment_mult: 1.0,
            current_velocity: Vec4::ZERO,
            forces: Vec::with_capacity(10),
            is_on_ground: false
        }
    }

    pub fn set_wish_direction(&mut self, wish_direction: Vec4, movement_mult: f32) {
        self.wish_direction = wish_direction;
        self.movment_mult = movement_mult;
    }

    pub fn add_force(&mut self, force: Vec4) {
        self.forces.push(force);
    }

    pub fn physics_tick(&mut self, delta: f32, static_objects: &StaticObjectsData) {

        self.is_on_ground = false;

        if self.wish_direction.length() > 0.0 {
            // self.wish_direction = self.wish_direction.normalize();

            let current_speed_in_wishdir = self.current_velocity.dot(self.wish_direction);

            let speed = self.max_speed - current_speed_in_wishdir;

            let add_speed = 0.0_f32.max(speed.min(self.max_accel * delta));

            self.current_velocity += self.wish_direction * (add_speed * self.movment_mult);

        }

        while let Some(force) = self.forces.pop() {
            self.current_velocity += force;
        }

        if self.is_enable {

            let (position_increment, is_collided) = translate_collider(self.transform.get_position(), self.current_velocity * delta, self.collider_radius, static_objects);
            
            if is_collided {
                self.current_velocity *= 1.0 - delta*3.5;
            } else {
                self.current_velocity *= 1.0 - delta*1.5;
            }
            // self.current_velocity *= 1.0 - delta*3.4;

            self.transform.increment_position(position_increment);

            if position_increment.length().is_normal() {
                self.current_velocity = self.current_velocity.project_onto_normalized(position_increment.normalize());
            }

            //check if collider staying on the ground
            let bottom_position = self.transform.get_position() - self.collider_radius * Vec4::Y;

            if get_dist(bottom_position, static_objects) < 0.1 {
                self.is_on_ground = true;
            }

        } else {

            // maybe temporal

            // if collider is not enable we nned to add some friction for movement
            self.current_velocity *= 1.0 - delta*3.4;
            
            self.transform.increment_position(self.current_velocity * delta);
        }

       
        self.wish_direction = Vec4::ZERO;
    }
}

pub struct StaticArea {}

pub struct DynamicArea {
    transform: Transform,
}

impl DynamicArea {
    pub fn new(transform: Transform) -> Self {
        DynamicArea {
            transform,
        }
    }
}

const THRESHOLD: f32 = 0.005;
const HALF_THRESHOLD: f32 = 0.00025;
const MAX_COLLIDING_ITERATIONS: u32 = 50;

fn translate_collider(
    mut position: Vec4,
    mut translation: Vec4,
    collider_radius: f32,
    static_objects: &StaticObjectsData
) -> (Vec4, bool) {

    let mut is_collide = false;

    let start_position = position;

    log::warn!("start position is {}", start_position);
    
    let mut counter = 0u32;

    while translation.length().is_normal() {

        log::warn!("ITERATION number {}", counter);

        if counter > MAX_COLLIDING_ITERATIONS {
            panic!("More then max colliding iterations");
        }

        // if collider stuck inside object let's push it out
        let is_pushed = move_collider_outside(&mut position, collider_radius, static_objects);

        // get distance from edge of the object to the nearest object
        let mut distance_from_edge = get_dist(position, static_objects) - collider_radius;

        log::warn!("distance from the edge is {}", distance_from_edge);

        // bound if collide
        if distance_from_edge < THRESHOLD || is_pushed {

            log::warn!("BOUND");

            is_collide = true;

            let normal = get_normal(position, static_objects);
            
            log::warn!("normal is {}", normal);

            log::warn!("translation len before reject is {}", translation.length());

            log::warn!("direction is {}", translation.normalize());

            if normal.dot(translation) < 0.0 {

                let probable_transltaion_dir = translation.reject_from_normalized(normal).normalize();

                let next_normal = get_normal(
                    position + probable_transltaion_dir * THRESHOLD,
                    static_objects
                );

                log::warn!("next normal is {}", normal);

                let curvature_coefficient = next_normal.dot(probable_transltaion_dir);

                log::warn!("curvature_coefficient is {}", curvature_coefficient);

                if curvature_coefficient < 0.0 {

                    let prev_normal = get_normal(
                        position - probable_transltaion_dir * THRESHOLD,
                        static_objects
                    );

                    log::warn!("prev normal is {}", prev_normal);

                    if next_normal.dot(translation) < 0.0 {
                        
                        translation = translation.reject_from_normalized(next_normal);
                        
                        log::warn!("direction after first bound is {}", translation.normalize());

                        log::warn!("translation len after first reject is {}", translation.length());
                    }

                    if prev_normal.dot(translation) < 0.0 {

                        translation = translation.reject_from_normalized(prev_normal);

                        log::warn!("direction after second bound is {}", translation.normalize());

                        log::warn!("translation len after second reject is {}", translation.length());    
                    }

                } else {

                    translation = translation.reject_from_normalized(normal);

                    log::warn!("direction after bound is {}", translation.normalize());

                    log::warn!("translation len after reject is {}", translation.length());
                }
            }

        }

        let dist_on_try_move = get_dist(
            position + translation.clamp_length_max(collider_radius - THRESHOLD),
            static_objects
        );

        if dist_on_try_move - collider_radius > 0.0 {

            position += translation.clamp_length_max(collider_radius - THRESHOLD);

            if translation.length() < collider_radius - THRESHOLD {

                return (position - start_position, is_collide);

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

            log::warn!("SMALL STEPS");

            if small_steps_counter > MAX_COLLIDING_ITERATIONS {
                panic!("More then max colliding small steps iterations");
            }

            let current_translation_len = translation_length.min(distance_from_edge.max(THRESHOLD));

            position += translation_dir * current_translation_len;

            translation_length -= current_translation_len;

            distance_from_edge = get_dist(position, static_objects) - collider_radius;

            translation = translation_dir * translation_length;
            
            if distance_from_edge < 0.0 {

                break;
            }

        }

        counter += 1;
    }

    (position - start_position, is_collide)
}

#[inline]
fn move_collider_outside(
    position: &mut Vec4,
    collider_radius: f32,
    static_objects: &StaticObjectsData
) -> bool {

    let mut pos = *position;

    let mut is_collided = false;
    
    let mut distance_from_center = get_dist(pos, static_objects);

    let mut counter = 0u32;

    while distance_from_center < 0.0 {
        if counter > MAX_COLLIDING_ITERATIONS {
            panic!("'move_collider_outside' More the max colliding iterations inside the object")
        }
        is_collided = true;

        let normal = get_normal(pos, static_objects);
        pos -= normal * (distance_from_center + THRESHOLD);
        
        distance_from_center = get_dist(pos, static_objects);
        
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
            
                let normal = get_big_normal(pos, collider_radius, static_objects);

                pos += normal * distance_from_edge.abs().max(THRESHOLD * 0.25);

                distance_from_edge = get_dist(pos, static_objects) - collider_radius;
        
            }
            
            break;
        }
        is_collided = true;

        let normal = get_normal(pos, static_objects);

        pos += normal * distance_from_edge.abs().max(THRESHOLD * 0.25);

        distance_from_edge = get_dist(pos, static_objects) - collider_radius;
        
        log::warn!("'move_collider_outside' disatnce from th edge is {}", distance_from_edge);

        counter += 1;
    }

    *position = pos;

    is_collided
}

#[inline]
fn get_dist(p: Vec4, static_objects: &StaticObjectsData) -> f32 {
    let mut d = f32::MAX;

    for (position, size) in static_objects.cubes.iter() {
         d = d.min(sd_box(p - position.clone(), size.clone()));
    }
    for (position, size) in static_objects.inf_w_cubes.iter() {
        d = d.min(sd_inf_box(p - position.clone(), size.xyz()));
    }
    for (position, size) in static_objects.spheres.iter() {
        d = d.min(sd_sphere(p - position.clone(), size.x));
    }
    for (position, size) in static_objects.shpcubes.iter() {
        d = d.min(sd_sph_box(p - position.clone(), size.clone()));
    }

    for (position, size) in static_objects.neg_cubes.iter() {
        d = d.max(-sd_box(p - position.clone(), size.clone()));
    }
    for (position, size) in static_objects.neg_inf_w_cubes.iter() {
        d = d.max(-sd_inf_box(p - position.clone(), size.xyz()));
    }
    for (position, size) in static_objects.neg_spheres.iter() {
        d = d.max(-sd_sphere(p - position.clone(),size.x));
    }
    for (position, size) in static_objects.neg_shpcubes.iter() {
        d = d.max(-sd_sph_box(p - position.clone(), size.clone()));
    }

    return d;
}

#[inline]
fn sd_inf_box(p: Vec4, b: Vec3) -> f32 {
    let d = Vec3::new(p.x, p.y, p.z).abs() - b;
    return f32::min(f32::max(d.x, f32::max(d.y, d.z)),0.0) + (d.max(Vec3::ZERO).length());
}

#[inline]
fn sd_sphere(p: Vec4, r: f32) -> f32 {
    p.length() - r
}

#[inline]
fn sd_sph_box(p: Vec4, b: Vec4) -> f32 {
    let d1: f32 = p.xy().length() - b.x;
    let d2: f32 = p.xz().length() - b.y;
    let d3: f32 = p.yz().length() - b.z;
    let d4: f32 = p.wx().length() - b.w;
    let d5: f32 = p.wy().length() - b.w;
    let d6: f32 = p.wz().length() - b.w;
    return d6.max(d5.max(d4.max(d1.max(d2.max(d3)))));
}

#[inline]
fn sd_sph_inf_box(p: Vec4, b: Vec4) -> f32 {
    let d1 = Vec2::new(p.w, p.x).length() - b.x;
    let d2 = Vec2::new(p.w, p.y).length() - b.y;
    let d = Vec2::new(p.x, p.y).abs() - Vec2::new(b.x,b.y);
    return f32::max(d1,f32::max(d2,f32::min(f32::max(d.x,d.y),0.0) + (d.max(Vec2::ZERO)).length()));
}

#[inline]
fn sd_box(p: Vec4, b: Vec4) -> f32 {
    let d = p.abs() - b;
    return f32::min(f32::max(d.x,f32::max(d.y,f32::max(d.z, d.w))),0.0) + d.max(Vec4::ZERO).length();
}

#[inline]
fn get_normal(p: Vec4, static_objects: &StaticObjectsData) -> Vec4 {
    let a = p + Vec4::new(HALF_THRESHOLD, 0.000, 0.000, 0.000);
    let b = p + Vec4::new(-HALF_THRESHOLD, 0.000, 0.000,0.000);
    let c = p + Vec4::new(0.000, HALF_THRESHOLD, 0.000, 0.000);
    let d = p + Vec4::new(0.000, -HALF_THRESHOLD, 0.000, 0.000);
    let e = p + Vec4::new(0.000, 0.000, HALF_THRESHOLD, 0.000);
    let f = p + Vec4::new(0.000, 0.000, -HALF_THRESHOLD,0.000);
    let g = p + Vec4::new(0.000, 0.000, 0.000, HALF_THRESHOLD);
    let h = p + Vec4::new(0.000, 0.000, 0.000, -HALF_THRESHOLD);

    let fa = get_dist(a, static_objects);
    let fb = get_dist(b, static_objects);
    let fc = get_dist(c, static_objects);
    let fd = get_dist(d, static_objects);
    let fe = get_dist(e, static_objects);
    let ff = get_dist(f, static_objects);
    let fg = get_dist(g, static_objects);
    let fh = get_dist(h, static_objects);

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

fn get_big_normal(p: Vec4, size: f32, static_objects: &StaticObjectsData) -> Vec4 {
    let a = p + Vec4::new(size, 0.000, 0.000, 0.000);
    let b = p + Vec4::new(-size, 0.000, 0.000,0.000);
    let c = p + Vec4::new(0.000, size, 0.000, 0.000);
    let d = p + Vec4::new(0.000, -size, 0.000, 0.000);
    let e = p + Vec4::new(0.000, 0.000, size, 0.000);
    let f = p + Vec4::new(0.000, 0.000, -size,0.000);
    let g = p + Vec4::new(0.000, 0.000, 0.000, size);
    let h = p + Vec4::new(0.000, 0.000, 0.000, -size);

    let fa = get_dist(a, static_objects);
    let fb = get_dist(b, static_objects);
    let fc = get_dist(c, static_objects);
    let fd = get_dist(d, static_objects);
    let fe = get_dist(e, static_objects);
    let ff = get_dist(f, static_objects);
    let fg = get_dist(g, static_objects);
    let fh = get_dist(h, static_objects);

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
