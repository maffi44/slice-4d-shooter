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

    pub fn physics_tick(&mut self, delta: f32, static_objects: &StaticObjectsData) {

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

            let (position_increment, is_collided) = translate_collider(self.transform.get_position(), self.current_velocity * delta, 0.5, static_objects);
            
            if is_collided {
                self.current_velocity *= 1.0 - delta*6.0;
            }
            self.transform.increment_position(position_increment);

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

const THRESHOLD: f32 = 0.005;
const HALF_THRESHOLD: f32 = 0.00025;
const MAX_COLLIDING_ITERATIONS: u32 = 50;

/*
fn translate_collider(mut position: Vec4, mut translation: Vec4, collider_radius: f32, static_objects: &StaticObjectsData) -> (Vec4, bool) {

    let mut is_collided = false;

    let start_postition = position;
    
    let mut distance_to_nearest_obj = get_dist(position, static_objects);
    
    // if the collider stuck in some object let's push it out
    if distance_to_nearest_obj <= 0.0 {

        while distance_to_nearest_obj <= collider_radius + THRESHOLD {
            log::warn!("INSIDE distance to near obj is {}", distance_to_nearest_obj);

            let mut normal = get_normal(position, static_objects);

            log::warn!("NORMAL IS {:?}", normal.normalize());


            if distance_to_nearest_obj < 0.0 {
                normal *= -1.0;
            }
            
            position += 
                normal*(distance_to_nearest_obj.abs()+collider_radius+THRESHOLD);
            
            distance_to_nearest_obj = get_dist(position, static_objects);
        }
        return (position - start_postition, true);
    }

    let mut iteration_counter = 0u32;
    // if the collider is overlaping the object let's push it out
    while distance_to_nearest_obj < collider_radius {

        log::warn!("OVERLAP iteration number {}", iteration_counter);
            
        if iteration_counter > MAX_COLLIDING_ITERATIONS {
            
            panic!("(DEBUG) Physics system error: Colliging iteration OVERLAP obj more then {}", MAX_COLLIDING_ITERATIONS);
        }

        is_collided = true;

        let overlap = collider_radius - distance_to_nearest_obj;

        log::warn!("OVERLAP is {}", overlap);

        let mut normal = get_normal(position, static_objects);

        log::warn!("OVERLAP normal is {:?}", normal);

        position += normal * (overlap + THRESHOLD);

        log::warn!("OVERLAP position after move is {:?}", position);

        distance_to_nearest_obj = get_dist(position, static_objects);

        log::warn!("OVERLAP distance to near obj is {}", distance_to_nearest_obj);

        iteration_counter += 1;
    }

    // normal collide
    let allowed_lenght = collider_radius - THRESHOLD; 
    
    let mut translation_lenght = translation.length();

    let mut iter_index = 0u32;
    
    while translation_lenght > 0.0 {

        log::warn!("translation length interation index is {}", iter_index);
        iter_index += 1;    

        log::warn!("translation length is {}", translation_lenght);
        
        if translation_lenght < allowed_lenght {

            translation = translation.normalize() * translation_lenght;

            translation_lenght = 0.0;
        } else {
            translation_lenght -= allowed_lenght;

            translation = translation.normalize() * allowed_lenght;
        }

        for i in 0..MAX_COLLIDING_ITERATIONS {

            log::warn!("ITRATION {}", i);

            let direction = translation.normalize();

            log::warn!("direction is {}", direction);
            log::warn!("position is {}", position);
            log::warn!("translation is {}", translation);
            
            log::warn!("position after first translation is {}", position + translation);

            let mut distance_to_nearest_obj = get_dist(position + translation, static_objects);

            log::warn!("distance to near obj is {}", distance_to_nearest_obj);

            // if not colliding end colider translation
            if distance_to_nearest_obj >= collider_radius - THRESHOLD {
                return ((position + translation) - start_postition, is_collided);
            }

            is_collided = true;

            while distance_to_nearest_obj < collider_radius {

                let distance_to_collide = distance_to_nearest_obj - collider_radius;
                
                position += direction * distance_to_collide;
                
                translation -= direction * distance_to_collide;
                
                distance_to_nearest_obj = get_dist(position, static_objects);
            }

            // if collider transltion is overlapping some object
            // let's move collider to collide with object and calculate new translation

            // let full_distance_to_translate = translation.length();

            // let distance_to_collide = distance_to_nearest_obj - (collider_radius + THRESHOLD * 0.5);


            // position += direction * distance_to_collide;
            

            let normal = get_normal(position, static_objects);

            // log::warn!("normal is {}", normal);

            // let coof = normal.dot(direction);
            
            // log::warn!("coof is {}", coof);

            // let backtrace = if coof.abs() < 0.0001 {
            //     position += normal * overlap;

            //     -translation.length()
            // } else {

            //     overlap * 1.0/coof
            // };

            // log::warn!("backtrace is {}", backtrace);

            // position += direction * backtrace;

            log::warn!("position after backtrace is {}", position);

            // collider rebound
            translation = translation.reject_from(-normal * translation.length());

            log::warn!("new translation is {}", translation);

        }

        panic!("(DEBUG) Physics system error: Colliging iteration more then {}", MAX_COLLIDING_ITERATIONS);
    }
    (position - start_postition, is_collided)
}*/


/*
fn translate_collider(mut position: Vec4, mut translation: Vec4, collider_radius: f32, static_objects: &StaticObjectsData) -> (Vec4, bool) {

    let mut is_collided = false;

    let start_position = position;

    let mut distance_to_nearest_obj = get_dist(position, static_objects);
    
    log::warn!("START TRASLATE COLLIDER");

    log::warn!("start position is {:?}", position);
    
    log::warn!("start translation is {:?}", translation);
    
    log::warn!("start distance to nearest object is {}", distance_to_nearest_obj);
    
    // before moving the collider if collider is stuck inside object let's push it out
    if distance_to_nearest_obj <= 0.0 {

        log::warn!("Center of collider is INSIDE object");

        is_collided = true;

        let mut iteration_counter = 0u32;

        while distance_to_nearest_obj <= collider_radius + THRESHOLD {

            log::warn!("INSIDE iteration number {}", iteration_counter);
            
            if iteration_counter > MAX_COLLIDING_ITERATIONS {
                
                panic!("(DEBUG) Physics system error: Colliging iteration INSIDE obj more then {}", MAX_COLLIDING_ITERATIONS);
            }

            log::warn!("INSIDE distance to near obj is {}", distance_to_nearest_obj);

            let mut normal = get_normal(position, static_objects);

            log::warn!("INSIDE normal is {:?}", normal);


            if distance_to_nearest_obj < 0.0 {
                normal *= -1.0;
            }
            
            position += 
                normal*(distance_to_nearest_obj.abs()+collider_radius+THRESHOLD);
            
            log::warn!("INSIDE position after translation is {:?}", position);
            
            distance_to_nearest_obj = get_dist(position, static_objects);
        }
    }

    let mut iteration_counter = 0u32;
    // if the collider is overlaping the object let's push it out
    while distance_to_nearest_obj < collider_radius {

        log::warn!("OVERLAP iteration number {}", iteration_counter);
            
        if iteration_counter > MAX_COLLIDING_ITERATIONS {
            
            panic!("(DEBUG) Physics system error: Colliging iteration OVERLAP obj more then {}", MAX_COLLIDING_ITERATIONS);
        }

        is_collided = true;

        let overlap = collider_radius - distance_to_nearest_obj;

        log::warn!("OVERLAP is {}", overlap);

        let mut normal = get_normal(position, static_objects);

        log::warn!("OVERLAP normal is {:?}", normal);

        position += normal * (overlap + THRESHOLD);

        log::warn!("OVERLAP position after move is {:?}", position);

        distance_to_nearest_obj = get_dist(position, static_objects);

        log::warn!("OVERLAP distance to near obj is {}", distance_to_nearest_obj);

        iteration_counter += 1;
    }

    let allowed_lenght = collider_radius - THRESHOLD; 
    
    let mut translation_lenght = translation.length();

    let mut iter_index = 0u32;
    
    while translation_lenght > 0.0 {

        log::warn!("translation length interation index is {}", iter_index);
        iter_index += 1;    

        log::warn!("translation length is {}", translation_lenght);
        
        if translation_lenght < allowed_lenght {

            translation = translation.normalize() * translation_lenght;

            translation_lenght = 0.0;
        } else {
            translation_lenght -= allowed_lenght;

            translation = translation.normalize() * allowed_lenght;
        }

        let mut iteration_counter = 0u32;
        // here the collider is guaranteed located outside and does not overlap the object
        while translation.length().is_normal() {

            log::warn!("start iteration {}", iteration_counter);

            if iteration_counter > MAX_COLLIDING_ITERATIONS {
                
                panic!("(DEBUG) Physics system error: Colliging iteration OVERLAP obj more then {}", MAX_COLLIDING_ITERATIONS);
            }

            log::warn!("position is {:?}", position);

            log::warn!("translation is {:?}", translation);
                
            log::warn!("distance to nearest object is {}", distance_to_nearest_obj);

            let distance_from_fin_position = get_dist(position + translation, static_objects);

            if distance_from_fin_position > collider_radius + THRESHOLD {
                return ((position + translation) - start_position, is_collided);
            }

            is_collided = true;
            
            let direction = translation.normalize();

            // move until collide or end of translation vector
            
            let mut distance_to_nearest_obj = get_dist(position, static_objects);

            while
                distance_to_nearest_obj < collider_radius + THRESHOLD ||
                !translation.length().is_normal()
            {

                let current_translation =
                    (distance_from_fin_position - collider_radius)
                    .min(translation.length());
                
                position += direction * current_translation;

                translation -= direction * current_translation;

                distance_to_nearest_obj = get_dist(position, static_objects);
            }

            

            // TODO - maybe here need to write code of pushing out collider if if overlaping object



            iteration_counter += 1;
        }
    }

    return (position - start_position, is_collided);
} */

//     let current_distance = get_dist(position, static_objects);

//     if distance_to_nearest_obj <= 0.0 {

//         while distance_to_nearest_obj <= collider_radius + THRESHOLD {
//             log::warn!("INSIDE distance to near obj is {}", distance_to_nearest_obj);

//             let mut normal = get_normal(position, static_objects);

//             log::warn!("NORMAL IS {:?}", normal.normalize());


//             if distance_to_nearest_obj < 0.0 {
//                 normal *= -1.0;
//             }
            
//             position += 
//                 normal*(distance_to_nearest_obj.abs()+collider_radius+THRESHOLD);
            
//             distance_to_nearest_obj = get_dist(position, static_objects);
//         }
//         return (position - start_postition, true);
//     }


//     let start_postition = position;
    
//     let allowed_lenght = collider_radius * 0.95; 

    
//     let mut distance_to_nearest_obj = get_dist(position, static_objects);
    
//     // if the collider stuck in some object let's push it out
//     if distance_to_nearest_obj <= 0.0 {

//         while distance_to_nearest_obj <= collider_radius + THRESHOLD {
//             log::warn!("INSIDE distance to near obj is {}", distance_to_nearest_obj);

//             let mut normal = get_normal(position, static_objects);

//             log::warn!("NORMAL IS {:?}", normal.normalize());


//             if distance_to_nearest_obj < 0.0 {
//                 normal *= -1.0;
//             }
            
//             position += 
//                 normal*(distance_to_nearest_obj.abs()+collider_radius+THRESHOLD);
            
//             distance_to_nearest_obj = get_dist(position, static_objects);
//         }
//         return (position - start_postition, true);
//     }

//     // normal collide
//     let mut translation_lenght = translation.length();

//     let mut iter_index = 0u32;
    
//     while translation_lenght > 0.0 {


//         log::warn!("translation length interation index is {}", iter_index);
//         iter_index += 1;    

//         log::warn!("translation length is {}", translation_lenght);
        
//         if translation_lenght > allowed_lenght {

//             translation_lenght -= allowed_lenght;

//             translation = translation.normalize() * allowed_lenght;
//         } else {
//             translation = translation.normalize() * translation_lenght;

//             translation_lenght = 0.0;
//         }

//         for i in 0..MAX_COLLIDING_ITERATIONS {

//             log::warn!("ITRATION {}", i);

//             let direction = translation.normalize();

//             log::warn!("direction is {}", direction);
//             log::warn!("position is {}", position);
//             log::warn!("translation is {}", translation);
            
//             position += translation;

//             log::warn!("position after translation is {}", position);

//             let mut distance_to_nearest_obj = get_dist(position, static_objects);

//             log::warn!("distance to near obj is {}", distance_to_nearest_obj);

//             // if not colliding end colider translation
//             if distance_to_nearest_obj > collider_radius - THRESHOLD {
//                 return (position - start_postition, is_collided);
//             }

//             is_collided = true;

//             // moving collider back if collided  
//             let overlap = collider_radius - distance_to_nearest_obj;

//             log::warn!("overlap is {}", overlap);

//             let normal = get_normal(position, static_objects);

//             log::warn!("normal is {}", normal);

//             let coof = normal.dot(direction);
            
//             log::warn!("coof is {}", coof);

//             let backtrace = if coof.abs() < 0.0001 {
//                 position += normal * overlap;

//                 -translation.length()
//             } else {

//                 overlap * 1.0/coof
//             };

//             log::warn!("backtrace is {}", backtrace);

//             position += direction * backtrace;

//             log::warn!("position after backtrace is {}", position);

//             // collider rebound
//             translation = direction.reject_from(-normal) * -backtrace;

//             log::warn!("new translation is {}", translation);

//         }

//         panic!("(DEBUG) Physics system error: Colliging iteration more then {}", MAX_COLLIDING_ITERATIONS);
//     }
//     (position - start_postition, is_collided)
// }


fn translate_collider(
    mut position: Vec4,
    mut translation: Vec4,
    collider_radius: f32,
    static_objects: &StaticObjectsData
) -> (Vec4, bool) {

    let mut is_collide = false;

    let start_position = position;

    log::warn!("start position is {}", start_position);

    // if collider stuck inside object let's push it out
    if get_dist(position, static_objects) < collider_radius {
        position = move_collider_outside(position, collider_radius, static_objects);
        is_collide = true;
    }

    let mut counter = 0u32;

    while translation.length().is_normal() {

        log::warn!("ITERATION number {}", counter);

        if counter > MAX_COLLIDING_ITERATIONS {
            panic!("More then max colliding iterations");
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
                    translation.length() - (collider_radius - THRESHOLD)
                );

                continue;
            }
        }
    
        // get distance from center of the object to the nearest object    
        let distance_from_center = get_dist(position, static_objects);

        log::warn!("distance from the center is {}", distance_from_center);

        let mut distance_from_edge = distance_from_center - collider_radius;

        log::warn!("distance from the edge is {}", distance_from_edge);

        // bound if collide
        if distance_from_edge < THRESHOLD {

            log::warn!("position before moving out is {}", position);

            position = move_collider_outside(position, collider_radius, static_objects);

            distance_from_edge = get_dist(position, static_objects) - collider_radius;

            log::warn!("position after moving out is {}", position);

            log::warn!("BOUND");
            
            if distance_from_edge < 0.0 {
                panic!("ERROR Collideris overlaped object")
            }

            is_collide = true;

            
            let normal = get_normal(position, static_objects);
            
            log::warn!("normal is {}", normal);

            log::warn!("translation len before reject is {}", translation.length());

            log::warn!("direction is {}", translation.normalize());

            translation = translation.reject_from_normalized(-normal);

            log::warn!("direction after bound is {}", translation.normalize());

            log::warn!("translation len after reject is {}", translation.length());

        }

        if distance_from_edge >= translation.length() {
            position += translation;

            log::warn!("MOVED");

            return (position - start_position, is_collide);
        }

        position += translation.clamp_length_max(distance_from_edge);

        log::warn!("position after move is {}", position);

        log::warn!("translation length is {}", translation.length());

        translation = translation.clamp_length_max(translation.length() - distance_from_edge);

        log::warn!("translation after change is {}", translation);

        log::warn!("translation length after chage is {}", translation.length());
        
        counter += 1;
    }


    (position - start_position, is_collide)
}

#[inline]
fn move_collider_outside(
    mut position: Vec4,
    collider_radius: f32,
    static_objects: &StaticObjectsData
) -> Vec4 {
    
    let mut distance_from_center = get_dist(position, static_objects);

    let mut counter = 0u32;

    while distance_from_center < 0.0 {
        if counter > MAX_COLLIDING_ITERATIONS {
            panic!("'move_collider_outside' More the max colliding iterations inside the object")
        }
        let normal = get_normal(position, static_objects);
        position -= normal * (distance_from_center + THRESHOLD);
        
        distance_from_center = get_dist(position, static_objects);
        
        counter += 1;
    }

    let mut distance_from_edge = distance_from_center - (collider_radius + THRESHOLD);

    let mut counter = 0u32;

    while distance_from_edge < -0.0001 {
        if counter > MAX_COLLIDING_ITERATIONS {
            panic!("'move_collider_outside' More the max colliding iterations when overlapping the object");
        }
        let normal = get_normal(position, static_objects);

        position += normal * distance_from_edge.abs();

        distance_from_edge = get_dist(position, static_objects) - (collider_radius + THRESHOLD);
        
        log::warn!("'move_collider_outside' disatnce from th edge is {}", distance_from_edge);

        counter += 1;
    }

    position
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

    for (position, size) in static_objects.cubes.iter() {
        d = d.max(-sd_box(p - position.clone(), size.clone()));
    }
    for (position, size) in static_objects.inf_w_cubes.iter() {
        d = d.max(-sd_inf_box(p - position.clone(), size.xyz()));
    }
    for (position, size) in static_objects.spheres.iter() {
        d = d.max(-sd_sphere(p - position.clone(),size.x));
    }
    for (position, size) in static_objects.shpcubes.iter() {
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

/*#[inline]
fn get_normal(p: Vec4, static_objects: &StaticObjectsData) -> Vec4 {
    let a = p + Vec4::new(-0.001, 0.001, 0.001, 0.001);
    let b = p + Vec4::new(0.001, -0.001, 0.001,-0.001);
    let c = p + Vec4::new(0.001, 0.001, -0.001, 0.001);
    let d = p + Vec4::new(-0.001, -0.001, -0.001, -0.001);

    let fa = get_dist(a, static_objects);
    let fb = get_dist(b, static_objects);
    let fc = get_dist(c, static_objects);
    let fd = get_dist(d, static_objects);

    let mut normal = 
        Vec4::new(-0.001, 0.001, 0.001, 0.0) * fa +
        Vec4::new(0.001, -0.001, 0.001, 0.0) * fb +
        Vec4::new(0.001, 0.001, -0.001, 0.0) * fc +
        Vec4::new(-0.001, -0.001, -0.001, 0.0) * fd;

    // if we are stuck in surface normal will be zero length
    // let's make some random normal in this case 
    while normal.try_normalize().is_none() {

        // let mut bytes : [u8;4] = [0,0,0,0];
        // let res = getrandom::getrandom(&mut bytes);
        
        // if let Err(err) = res {
        //     panic!("Can't make random f32 in get_normal fnction");
        // }
        // let x: f32 = f32::from_be_bytes(bytes);


        // let mut bytes : [u8;4] = [0,0,0,0];
        // let res = getrandom::getrandom(&mut bytes);
        
        // if let Err(err) = res {
        //     panic!("Can't make random f32 in get_normal fnction");
        // }
        // let y: f32 = f32::from_be_bytes(bytes);


        // let mut bytes : [u8;4] = [0,0,0,0];
        // let res = getrandom::getrandom(&mut bytes);
        
        // if let Err(err) = res {
        //     panic!("Can't make random f32 in get_normal fnction");
        // }
        // let z: f32 = f32::from_be_bytes(bytes);


        // let mut bytes : [u8;4] = [0,0,0,0];
        // let res = getrandom::getrandom(&mut bytes);
        
        // if let Err(err) = res {
        //     panic!("Can't make random f32 in get_normal fnction");
        // }
        // let w: f32 = f32::from_be_bytes(bytes);
        

        // normal = Vec4::new(x, y, z, w);
        normal = Vec4::new(0.0, 1.0, 0.0, 1.0);
    }
    normal.normalize()
}*/


#[inline]
fn get_normal(p: Vec4, static_objects: &StaticObjectsData) -> Vec4 {
    let a = p + Vec4::new(THRESHOLD, 0.000, 0.000, 0.000);
    let b = p + Vec4::new(-THRESHOLD, 0.000, 0.000,0.000);
    let c = p + Vec4::new(0.000, THRESHOLD, 0.000, 0.000);
    let d = p + Vec4::new(0.000, -THRESHOLD, 0.000, 0.000);
    let e = p + Vec4::new(0.000, 0.000, THRESHOLD, 0.000);
    let f = p + Vec4::new(0.000, 0.000, -THRESHOLD,0.000);
    let g = p + Vec4::new(0.000, 0.000, 0.000, THRESHOLD);
    let h = p + Vec4::new(0.000, 0.000, 0.000, -THRESHOLD);

    let fa = get_dist(a, static_objects);
    let fb = get_dist(b, static_objects);
    let fc = get_dist(c, static_objects);
    let fd = get_dist(d, static_objects);
    let fe = get_dist(e, static_objects);
    let ff = get_dist(f, static_objects);
    let fg = get_dist(g, static_objects);
    let fh = get_dist(h, static_objects);

    let mut normal = 
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
    while normal.try_normalize().is_none() {

        // let mut bytes : [u8;4] = [0,0,0,0];
        // let res = getrandom::getrandom(&mut bytes);
        
        // if let Err(err) = res {
        //     panic!("Can't make random f32 in get_normal fnction");
        // }
        // let x: f32 = f32::from_be_bytes(bytes);


        // let mut bytes : [u8;4] = [0,0,0,0];
        // let res = getrandom::getrandom(&mut bytes);
        
        // if let Err(err) = res {
        //     panic!("Can't make random f32 in get_normal fnction");
        // }
        // let y: f32 = f32::from_be_bytes(bytes);


        // let mut bytes : [u8;4] = [0,0,0,0];
        // let res = getrandom::getrandom(&mut bytes);
        
        // if let Err(err) = res {
        //     panic!("Can't make random f32 in get_normal fnction");
        // }
        // let z: f32 = f32::from_be_bytes(bytes);


        // let mut bytes : [u8;4] = [0,0,0,0];
        // let res = getrandom::getrandom(&mut bytes);
        
        // if let Err(err) = res {
        //     panic!("Can't make random f32 in get_normal fnction");
        // }
        // let w: f32 = f32::from_be_bytes(bytes);
        

        // normal = Vec4::new(x, y, z, w);
        normal = Vec4::new(0.0, 1.0, 0.0, 1.0);
    }
    normal.normalize()
}

