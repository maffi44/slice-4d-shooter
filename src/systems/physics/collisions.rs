use glam::{
    Vec4, Vec3, Vec2
};

use super::super::transform::Transform;

use crate::systems::static_obj::StaticObject;

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
    max_speed: f32,
    max_accel: f32,
    wish_direction: Vec4,
    current_velocity: Vec4,
    forces: Vec<Vec4>,
}

impl DynamicCollision {
    pub fn new(transform: Transform, max_speed: f32, max_accel: f32,) -> Self {
        DynamicCollision {
            is_enable: true,
            transform,
            max_speed,
            max_accel,
            wish_direction: Vec4::ZERO,
            current_velocity: Vec4::ZERO,
            forces: Vec::with_capacity(10),
        }
    }

    pub fn set_wish_direction(&mut self, wish_direction: Vec4) {
        self.wish_direction = wish_direction
    }

    pub fn add_force(&mut self, force: Vec4) {
        self.forces.push(force);
    }

    pub fn physics_tick(&mut self, delta: f32, map: &Vec<StaticObject>) {

        let mut frame_postition_inctrement = Vec4::ZERO;

        if self.wish_direction.length() > 0.0 {
            // self.wish_direction = self.wish_direction.normalize();

            let current_speed_in_wishdir = self.current_velocity.dot(self.wish_direction);

            let speed = self.max_speed - current_speed_in_wishdir;

            let add_speed = 0.0_f32.max(speed.min(self.max_accel * delta));

            self.current_velocity += self.wish_direction * add_speed;

        }

        while let Some(force) = self.forces.pop() {
            self.current_velocity += force;
        }

        if self.is_enable {

            translate_collider(self.transform.get_position(), self.current_velocity * delta, 0.5);

        } else {

            // maybe temporal

            // if collider is not enable we nned to add some friction for movement
            self.current_velocity *= 1.0 - delta*4.0;
            
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

use std::f32::consts::PI;
// use glm::Vec3;
// use glm::Vec4;
// use glm::{cos, sin};
// use glm::vec3 as vec3;
// use glm::vec4 as vec4;
// use glm::vec2 as vec2;
// use glm::min as min;
// use glm::max as max;
// use glm::length as length;
// use glm::abs as abs;
// use glm::clamp;
// use glm::cross;
// use glm::normalize;
// use glm::dot;
// use glam::Vec3::dot;

const THRESHOLD: f32 = 0.005;
const MAX_COLLIDING_ITERATIONS: i32 = 15;

fn translate_collider(mut position: Vec4, mut translation: Vec4, collider_radius: f32) -> Vec4 {

    let start_postition = position;

    if translation.length() == 0.0 {
        return position - start_postition;
    }

    for _ in (0..MAX_COLLIDING_ITERATIONS) {

        let direction = translation.normalize();
        
        position += translation;

        let distance_to_nearest_obj = get_dist(position);
        
        // if not colliding end collider translation
        if distance_to_nearest_obj > collider_radius {
            return position - start_postition;
        }

        // moving collider back if collided
        let overlap = collider_radius - distance_to_nearest_obj;

        let normal = get_normal(position);

        let coof = normal.dot(direction);

        let backtrace = overlap * coof;

        position -= direction * backtrace;

        // collider rebound
        translation = direction.reject_from(normal) * backtrace;
    }

    panic!("Physics system error: Colliging iteration more then {}", MAX_COLLIDING_ITERATIONS);
}

// fn ray_march(mut ray_origin: Vec4, ray_direction: Vec4, max_dist: f32) -> f32 {
//     let mut total_distance = 0.0;

//     let mut i = 0;
//     while i < 128 {
//         ray_origin = ray_origin + (ray_direction * 0.2);
//         let d = get_dist(ray_origin);
//         total_distance += d;

//         if d < 0.35 {
//             // if  get_dist(ray_origin + (ray_direction * -d)) >= 0.2 {
//             //     return total_distance - 0.2;
//             // }
//             return (
//                 (total_distance - 0.35).clamp(0.0, max_dist),
//             )
//             get_normal(ray_origin)
//         }
//         if total_distance >= max_dist {
//             return max_dist;
//         }
        
        
//         i += 1;
//     }
//     return max_dist;
// }

fn get_dist(p: Vec4) -> f32 {
    let d = MAX_DIST * 2.2;

    for (let i = 0u; i < shapes_array_metadata.cubes.amount; i++) {
        let index = i + shapes_array_metadata.cubes.first_index;
        d = min(d, sd_box(p - shapes[index].pos, shapes[index].size));
    }
    for (let i = 0u; i < shapes_array_metadata.cubes_inf_w.amount; i++) {
        let index = i + shapes_array_metadata.cubes_inf_w.first_index;
        d = min(d, sd_inf_box(p - shapes[index].pos, shapes[index].size.xyz));
    }
    for (let i = 0u; i < shapes_array_metadata.spheres.amount; i++) {
        let index = i + shapes_array_metadata.spheres.first_index;
        d = min(d, sd_sphere(p - shapes[index].pos, shapes[index].size.x));
    }
    for (let i = 0u; i < shapes_array_metadata.sph_cube.amount; i++) {
        let index = i + shapes_array_metadata.sph_cube.first_index;
        d = min(d, sd_sph_box(p - shapes[index].pos, shapes[index].size));
    }

    for (let i = 0u; i < shapes_array_metadata.neg_cubes.amount; i++) {
        let index = i + shapes_array_metadata.neg_cubes.first_index;
        d = max(d, -sd_box(p - shapes[index].pos, shapes[index].size));
    }
    for (let i = 0u; i < shapes_array_metadata.neg_cubes_inf_w.amount; i++) {
        let index = i + shapes_array_metadata.neg_cubes_inf_w.first_index;
        d = max(d, -sd_inf_box(p - shapes[index].pos, shapes[index].size.xyz));
    }
    for (let i = 0u; i < shapes_array_metadata.neg_spheres.amount; i++) {
        let index = i + shapes_array_metadata.neg_spheres.first_index;
        d = max(d, -sd_sphere(p - shapes[index].pos, shapes[index].size.x));
    }
    for (let i = 0u; i < shapes_array_metadata.neg_sph_cube.amount; i++) {
        let index = i + shapes_array_metadata.neg_sph_cube.first_index;
        d = max(d, -sd_sph_box(p - shapes[index].pos, shapes[index].size));
    }

    return d;
}

fn sd_inf_box(p: Vec4, b: Vec3) -> f32 {
    let d = Vec3::new(p.x, p.y, p.z).abs() - b;
    return f32::min(f32::max(d.x, f32::max(d.y, d.z)),0.0) + (d.max(Vec3::ZERO).length());
}


fn sd_sph_inf_box(p: Vec4, b: Vec4) -> f32 {
    let d1 = Vec2::new(p.w, p.x).length() - b.x;
    let d2 = Vec2::new(p.w, p.y).length() - b.y;
    let d = Vec2::new(p.x, p.y).abs() - Vec2::new(b.x,b.y);
    return f32::max(d1,f32::max(d2,f32::min(f32::max(d.x,d.y),0.0) + (d.max(Vec2::ZERO)).length()));
}

fn sd_box(p: Vec4, b: Vec4) -> f32 {
    let d = p.abs() - b;
    return f32::min(f32::max(d.x,f32::max(d.y,f32::max(d.z, d.w))),0.0) + d.max(Vec4::ZERO).length();
}

fn get_normal(p: Vec4) -> Vec4 {
    let a = p + Vec4::new(-0.001, 0.001, 0.001, 0.0);
    let b = p + Vec4::new(0.001, -0.001, 0.001, 0.0);
    let c = p + Vec4::new(0.001, 0.001, -0.001, 0.0);
    let d = p + Vec4::new(-0.001, -0.001, -0.001, 0.0);

    let fa = get_dist(a);
    let fb = get_dist(b);
    let fc = get_dist(c);
    let fd = get_dist(d);

    let normal = 
        Vec4::new(-0.001, 0.001, 0.001, 0.0) * fa +
        Vec4::new(0.001, -0.001, 0.001, 0.0) * fb +
        Vec4::new(0.001, 0.001, -0.001, 0.0) * fc +
        Vec4::new(-0.001, -0.001, -0.001, 0.0) * fd;
    
    normal.normalize()
}


