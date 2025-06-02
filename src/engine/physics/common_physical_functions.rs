use client_server_protocol::Team;
use glam::{
    Vec2,
    Vec3,
    Vec4,
    Vec4Swizzles,
};
use web_sys::js_sys::Math::max;

use crate::actor::ActorID;

use super::physics_system_data::PhysicsState;

pub const THRESHOLD: f32 = 0.009;
pub const MAX_DIST: f32 = 700_f32;
// pub const HALF_THRESHOLD: f32 = 0.00025;

#[inline]
pub fn sd_inf_box(p: Vec4, b: Vec3) -> f32 {
    let d = Vec3::new(p.x, p.y, p.z).abs() - b;
    return f32::min(f32::max(d.x, f32::max(d.y, d.z)),0.0) + (d.max(Vec3::ZERO).length());
}

#[inline]
pub fn sd_sphere(p: Vec4, r: f32) -> f32 {
    p.length() - r
}

#[inline]
pub fn sd_sph_box(p: Vec4, b: Vec4) -> f32 {
    let d1: f32 = p.xy().length() - b.z;
    let d2: f32 = p.xz().length() - b.y;
    let d3: f32 = p.yz().length() - b.x;
    let d4: f32 = p.wx().length() - b.w;
    let d5: f32 = p.wy().length() - b.w;
    let d6: f32 = p.wz().length() - b.w;
    return d6.max(d5.max(d4.max(d1.max(d2.max(d3)))));
}

#[inline]
pub fn sd_sph_inf_box(p: Vec4, b: Vec4) -> f32 {
    let d1 = Vec2::new(p.w, p.x).length() - b.x;
    let d2 = Vec2::new(p.w, p.y).length() - b.y;
    let d = Vec2::new(p.x, p.y).abs() - Vec2::new(b.x,b.y);
    return f32::max(d1,f32::max(d2,f32::min(f32::max(d.x,d.y),0.0) + (d.max(Vec2::ZERO)).length()));
}

#[inline]
pub fn sd_box(p: Vec4, b: Vec4) -> f32 {
    let d = p.abs() - b;
    return f32::min(f32::max(d.x,f32::max(d.y,f32::max(d.z, d.w))),0.0) + d.max(Vec4::ZERO).length();
}

// #[inline]
// fn smin(a: f32, b: f32, k: f32) -> f32
// {
//     let x = (b-a)/k;
//     let g = 0.5*(x-(x*x+0.25).sqrt());
//     return a + k * g;
// }

#[inline]
fn smin( a: f32, b: f32, k: f32 ) -> f32
{
    let kk = k * 1.0/(1.0-0.5_f32.sqrt());
    let h = (kk-(a-b).abs()).max(0.0)/kk;
    return a.min(b) - kk*0.5*(1.0+h-(1.0-h*(h-2.0)).sqrt());
}


#[inline]
fn smax( a: f32, b: f32, k: f32 ) -> f32
{
    let kk = k * 1.0/(1.0-0.5_f32.sqrt());
    let h = (kk-(a-b).abs()).max(0.0)/kk;
    return a.max(b) + kk*0.5*(1.0+h-(1.0-h*(h-2.0)).sqrt());
}

// pub fn get_id(p: Vec4, static_objects: &PhysicsState) -> Option<ActorID> {
//     let mut d = MAX_DIST;

//     let mut nearest_collider = None;

//     let stickiness = static_objects.stickiness;

//     for collider in static_objects.cubes.iter_stickiness() {
//         let new_d = sd_box(p - collider.position, collider.size) - collider.roundness;
        
//         let dd = smin(d, new_d, stickiness);
        
//         if dd < d {
//             nearest_collider = Some(collider);
//         }

//         d = dd;
//     }
//     for collider in static_objects.inf_w_cubes.iter_stickiness() {
//         let new_d = sd_inf_box(p - collider.position, collider.size.xyz()) - collider.roundness;

//         let dd = smin(d, new_d, stickiness);
        
//         if dd < d {
//             nearest_collider = Some(collider);
//         }

//         d = dd;
//     }
//     for collider in static_objects.spheres.iter_stickiness() {
//         let new_d = sd_sphere(p - collider.position, collider.size.x) - collider.roundness;

//         let dd = smin(d, new_d, stickiness);
        
//         if dd < d {
//             nearest_collider = Some(collider);
//         }

//         d = dd;
//     }
//     for collider in static_objects.sph_cubes.iter_stickiness() {
//         let new_d = sd_sph_box(p - collider.position, collider.size) - collider.roundness;

//         let dd = smin(d, new_d, stickiness);
        
//         if dd < d {
//             nearest_collider = Some(collider);
//         }

//         d = dd;
//     }
    

//     for collider in static_objects.cubes.iter_normal() {
//         let new_d = sd_box(p - collider.position, collider.size) - collider.roundness;

//         if new_d < d {
//             nearest_collider = Some(collider);

//             d = new_d;
//         };
//     }
//     for collider in static_objects.inf_w_cubes.iter_normal() {
//         let new_d = sd_inf_box(p - collider.position, collider.size.xyz()) - collider.roundness;

//         if new_d < d {
//             nearest_collider = Some(collider);

//             d = new_d;
//         };
//     }
//     for collider in static_objects.spheres.iter_normal() {
//         let new_d = sd_sphere(p - collider.position, collider.size.x) - collider.roundness;

//         if new_d < d {
//             nearest_collider = Some(collider);

//             d = new_d;
//         };
//     }
//     for collider in static_objects.sph_cubes.iter_normal() {
//         let new_d = sd_sph_box(p - collider.position, collider.size) - collider.roundness;

//         if new_d < d {
//             nearest_collider = Some(collider);

//             d = new_d;
//         };
//     }

//     let mut dd = MAX_DIST;

//     for collider in static_objects.cubes.iter_neg_stickiness() {
//         dd = smin(
//             dd,
//             sd_box(p - collider.position, collider.size) - collider.roundness,
//             stickiness
//         );
//     }
//     for collider in static_objects.inf_w_cubes.iter_neg_stickiness() {
//             dd = smin(
//             dd,
//             sd_inf_box(p - collider.position, collider.size.xyz()) - collider.roundness,
//             stickiness
//         );
//     }
//     for collider in static_objects.spheres.iter_neg_stickiness() {
//             dd = smin(
//             dd,
//             sd_sphere(p - collider.position, collider.size.x) - collider.roundness,
//             stickiness
//         );
//     }
//     for collider in static_objects.sph_cubes.iter_neg_stickiness() {
//             dd = smin(
//             dd,
//             sd_sph_box(p - collider.position, collider.size) - collider.roundness,
//             stickiness
//         );
//     }

//     if -dd > d {
//         nearest_collider = None;
//     }

//     d = d.max(-dd);

//     let prev_d = d;
    

//     for collider in static_objects.cubes.iter_negative() {
//         d = d.max(-(sd_box(p - collider.position, collider.size) - collider.roundness));
//     }
//     for collider in static_objects.inf_w_cubes.iter_negative() {
//         d = d.max(-(sd_inf_box(p - collider.position, collider.size.xyz()) - collider.roundness));
//     }
//     for collider in static_objects.spheres.iter_negative() {
//         d = d.max(-(sd_sphere(p - collider.position,collider.size.x) - collider.roundness));
//     }
//     for collider in static_objects.sph_cubes.iter_negative() {
//         d = d.max(-(sd_sph_box(p - collider.position, collider.size) - collider.roundness));
//     }

//     if d > prev_d {
//         nearest_collider = None;
//     }

//     let mut new_new_d = MAX_DIST;

//     let mut nearest_player = None;

//     for collider in static_objects.dyn_spheres.iter() {
//         let new_d = sd_sphere(p - collider.position, collider.radius);

//         if new_d < new_new_d {
//             nearest_player = Some(collider);

//             new_new_d = new_d;
//         };
//     }

//     if nearest_collider.is_none() && nearest_player.is_none() {
//         return None;
//     }

//     if !nearest_collider.is_none() && nearest_player.is_none() {

//         let collider = nearest_collider.unwrap();

//         let mut d = MAX_DIST;

//         match collider.shape_type {
//             ShapeType::Cube => {
//                 d = sd_box(p, collider.size) - collider.roundness;
//             }
//             ShapeType::CubeInfW => {
//                 d = sd_inf_box(p, collider.size.xyz()) - collider.roundness;
//             }
//             ShapeType::Sphere => {
//                 d = sd_sphere(p, collider.size.x) - collider.roundness;
//             }
//             ShapeType::SphCube => {
//                 d = sd_sph_box(p, collider.size) - collider.roundness;
//             }
//         }

//         if d < THRESHOLD {
//             return collider.get_id();
//         } else {
//             return None;
//         }
//     }

//     if nearest_collider.is_none() && !nearest_player.is_none() {

//         let player = nearest_player.unwrap();

//         let d = sd_sphere(p, player.radius);

//         if d < THRESHOLD {
//             return player.get_id();
//         } else {
//             return None;
//         }
//     }

//     if !nearest_collider.is_none() && !nearest_player.is_none() {

//         let collider = nearest_collider.unwrap();

//         let mut collider_d = MAX_DIST;

//         match collider.shape_type {
//             ShapeType::Cube => {
//                 collider_d = sd_box(p, collider.size) - collider.roundness;
//             }
//             ShapeType::CubeInfW => {
//                 collider_d = sd_inf_box(p, collider.size.xyz()) - collider.roundness;
//             }
//             ShapeType::Sphere => {
//                 collider_d = sd_sphere(p, collider.size.x) - collider.roundness;
//             }
//             ShapeType::SphCube => {
//                 collider_d = sd_sph_box(p, collider.size) - collider.roundness;
//             }
//         }

//         let player = nearest_player.unwrap();

//         let player_d = sd_sphere(p, player.radius);

//         if player_d < THRESHOLD {
//             return player.get_id();
//         } else if collider_d < THRESHOLD {
//             return collider.get_id();
//         } else {
//             return None;
//         }
//     }
    
//     None
// }

pub fn get_id_and_team(p: Vec4, static_objects: &PhysicsState) -> Option<(Option<ActorID>, Option<Team>)> {

    for collider in static_objects.player_forms.iter() {
        let d = sd_sphere(p - collider.position, collider.radius);

        if d < THRESHOLD {
            return  Some((collider.get_id(), Some(collider.actors_team)));
        }
    }

    None
}


#[inline]
pub fn get_dist(
    p: Vec4,
    static_objects: &PhysicsState,
    excluding_ids: Option<ActorID>
) -> f32 {
    let mut d = MAX_DIST;

    let stickiness = static_objects.stickiness;

    for collider in static_objects.cubes.iter_normal() {
         d = d.min(sd_box(p - collider.position, collider.size) - collider.roundness);
    }
    for collider in static_objects.inf_w_cubes.iter_normal() {
        d = d.min(sd_inf_box(p - collider.position, collider.size.xyz()) - collider.roundness);
    }
    for collider in static_objects.spheres.iter_normal() {
        d = d.min(sd_sphere(p - collider.position, collider.size.x) - collider.roundness);
    }
    for collider in static_objects.sph_cubes.iter_normal() {
        d = d.min(sd_sph_box(p - collider.position, collider.size) - collider.roundness);
    }

    for collider in static_objects.cubes.iter_stickiness() {
        d = smin(
           d,
           sd_box(p - collider.position, collider.size) - collider.roundness,
           stickiness
       );
   }
   for collider in static_objects.inf_w_cubes.iter_stickiness() {
       d = smin(
           d,
           sd_inf_box(p - collider.position, collider.size.xyz()) - collider.roundness,
           stickiness
       );
   }
   for collider in static_objects.spheres.iter_stickiness() {
       d = smin(
           d,
           sd_sphere(p - collider.position, collider.size.x) - collider.roundness,
           stickiness
       );
   }
   for collider in static_objects.sph_cubes.iter_stickiness() {
       d = smin(
           d,
           sd_sph_box(p - collider.position, collider.size) - collider.roundness,
           stickiness
       );
   }



    

    for collider in static_objects.cubes.iter_negative() {
        d = d.max(-(sd_box(p - collider.position, collider.size) - collider.roundness));
    }
    for collider in static_objects.inf_w_cubes.iter_negative() {
        d = d.max(-(sd_inf_box(p - collider.position, collider.size.xyz()) - collider.roundness));
    }
    for collider in static_objects.spheres.iter_negative() {
        d = d.max(-(sd_sphere(p - collider.position,collider.size.x) - collider.roundness));
    }
    for collider in static_objects.sph_cubes.iter_negative() {
        d = d.max(-(sd_sph_box(p - collider.position, collider.size) - collider.roundness));
    }


    // let mut dd = MAX_DIST;

    for collider in static_objects.cubes.iter_neg_stickiness() {
        d = smax(
            d,
            -(sd_box(p - collider.position, collider.size) - collider.roundness),
            stickiness
        );
    }
    for collider in static_objects.inf_w_cubes.iter_neg_stickiness() {
            d = smax(
            d,
            -(sd_inf_box(p - collider.position, collider.size.xyz()) - collider.roundness),
            stickiness
        );
    }
    for collider in static_objects.spheres.iter_neg_stickiness() {
            d = smax(
            d,
            -(sd_sphere(p - collider.position, collider.size.x) - collider.roundness),
            stickiness
        );
    }
    for collider in static_objects.sph_cubes.iter_neg_stickiness() {
            d = smax(
            d,
            -(sd_sph_box(p - collider.position, collider.size) - collider.roundness),
            stickiness
        );
    }

    for collider in static_objects.cubes.iter_undestroyable() {
        d = d.min(sd_box(p - collider.position, collider.size) - collider.roundness);
    }

    match excluding_ids {
        Some(id) =>
        {
            for collider in static_objects.player_forms.iter() {
                if id != collider.actor_id.expect("Some PlayerDollCollider have not actor_id during physics tick")
                {
                    d = d.min(sd_sphere(p - collider.position, collider.radius));
                }
            }
        }
        None =>
        {
            for collider in static_objects.player_forms.iter() {
                d = d.min(sd_sphere(p - collider.position, collider.radius));
            }
        }
    }

    

    // if let Some(w_floor) = &static_objects.w_floor {
    //     d = d.min(p.w - w_floor.w_pos);
    // }

    // if let Some(w_roof) = &static_objects.w_roof {
    //     d = d.min(w_roof.w_pos - p.w);
    // }

    return d;
}


#[inline]
pub fn get_bounce_and_friction(
    position: Vec4,
    collider_radius: f32,
    static_objects: &PhysicsState,
) -> (f32, f32) {
    let mut d = MAX_DIST;

    let mut bounce_coeficient: f32 = 0.0;
    let mut friction: f32 = 0.0;

    
    

    for collider in static_objects.cubes.iter_normal() {
        let new_d = sd_box(position - collider.position, collider.size) - collider.roundness;

        if new_d < d{
            bounce_coeficient = collider.bounce_rate;
            friction = collider.friction;

            d = new_d;
        };
    }

    for collider in static_objects.spheres.iter_normal() {
        let new_d = sd_sphere(position - collider.position, collider.size.x) - collider.roundness;

        if new_d < d{
            bounce_coeficient = collider.bounce_rate;
            friction = collider.friction;

            d = new_d;
        };
    }
    for collider in static_objects.sph_cubes.iter_normal() {
        let new_d = sd_sph_box(position - collider.position, collider.size) - collider.roundness;

        if new_d < d{
            bounce_coeficient = collider.bounce_rate;
            friction = collider.friction;

            d = new_d;
        };
    }

    let stickiness = static_objects.stickiness;

    for collider in static_objects.cubes.iter_stickiness() {
        let mut new_d = sd_box(position - collider.position, collider.size) - collider.roundness;

        new_d = smin(d, new_d, stickiness);

        if new_d < d{
            bounce_coeficient = collider.bounce_rate;
            friction = collider.friction;

            d = new_d;
        };
    }

    for collider in static_objects.spheres.iter_stickiness() {
        let mut new_d = sd_sphere(position - collider.position, collider.size.x) - collider.roundness;

        new_d = smin(d, new_d, stickiness);
        
        if d < d{
            bounce_coeficient = collider.bounce_rate;
            friction = collider.friction;

            d = new_d;


        };
    }
    for collider in static_objects.sph_cubes.iter_stickiness() {
        let mut new_d = sd_sph_box(position - collider.position, collider.size) - collider.roundness;

        new_d = smin(d, new_d, stickiness);

        if new_d < d{
            bounce_coeficient = collider.bounce_rate;
            friction = collider.friction;

            d = new_d;


        };
    }

    for collider in static_objects.cubes.iter_negative() {
        let new_d = sd_box(position - collider.position, collider.size) - collider.roundness;

        d = d.max(-new_d);
    }

    for collider in static_objects.spheres.iter_negative() {
        let new_d = sd_sphere(position - collider.position, collider.size.x) - collider.roundness;

        d = d.max(-new_d);
    }
    for collider in static_objects.sph_cubes.iter_negative() {
        let new_d = sd_sph_box(position - collider.position, collider.size) - collider.roundness;

        d = d.max(-new_d);
    }

    let stickiness = static_objects.stickiness;

    for collider in static_objects.cubes.iter_neg_stickiness() {
        let new_d = sd_box(position - collider.position, collider.size) - collider.roundness;

        d = smax(d, -new_d, stickiness);
    }

    for collider in static_objects.spheres.iter_neg_stickiness() {
        let new_d = sd_sphere(position - collider.position, collider.size.x) - collider.roundness;
        
        d = smax(d, -new_d, stickiness);
    }
    for collider in static_objects.sph_cubes.iter_neg_stickiness() {
        let new_d = sd_sph_box(position - collider.position, collider.size) - collider.roundness;

        d = smax(d, -new_d, stickiness);
    }

    if d - THRESHOLD*2.5 > collider_radius {
        bounce_coeficient = 0.0;
        friction = 0.0;
    };

    for collider in static_objects.cubes.iter_undestroyable() {
        let new_d = sd_box(position - collider.position, collider.size) - collider.roundness;

        if new_d < d{
            bounce_coeficient = bounce_coeficient.max(collider.bounce_rate);
            friction = friction.max(collider.friction);

            d = new_d;
        };
    }


    (bounce_coeficient, friction)
}



#[inline]
pub fn get_normal(
    p: Vec4,
    static_objects: &PhysicsState,
    excluding_ids: Option<ActorID>,
) -> Vec4 {
    let a = p + Vec4::new(THRESHOLD, 0.000, 0.000, 0.000);
    let b = p + Vec4::new(-THRESHOLD, 0.000, 0.000,0.000);
    let c = p + Vec4::new(0.000, THRESHOLD, 0.000, 0.000);
    let d = p + Vec4::new(0.000, -THRESHOLD, 0.000, 0.000);
    let e = p + Vec4::new(0.000, 0.000, THRESHOLD, 0.000);
    let f = p + Vec4::new(0.000, 0.000, -THRESHOLD,0.000);
    let g = p + Vec4::new(0.000, 0.000, 0.000, THRESHOLD);
    let h = p + Vec4::new(0.000, 0.000, 0.000, -THRESHOLD);

    let fa = get_dist(a, static_objects, excluding_ids);
    let fb = get_dist(b, static_objects, excluding_ids);
    let fc = get_dist(c, static_objects, excluding_ids);
    let fd = get_dist(d, static_objects, excluding_ids);
    let fe = get_dist(e, static_objects, excluding_ids);
    let ff = get_dist(f, static_objects, excluding_ids);
    let fg = get_dist(g, static_objects, excluding_ids);
    let fh = get_dist(h, static_objects, excluding_ids);

    let normal = 
        Vec4::new(1.000, 0.000, 0.000, 0.000) * fa +
        Vec4::new(-1.000, 0.000, 0.000,0.000) * fb +
        Vec4::new(0.000, 1.000, 0.000, 0.000) * fc +
        Vec4::new(0.000, -1.000, 0.000, 0.000) * fd +
        Vec4::new(0.000, 0.000, 1.000, 0.000) * fe +
        Vec4::new(0.000, 0.000, -1.000,0.000) * ff +
        Vec4::new(0.000, 0.000, 0.000, 1.000) * fg +
        Vec4::new(0.000, 0.000, 0.000, -1.000) * fh;

    // if the collider is stuck in object's surface or in object's
    // absolute center normal will be zero length let's make some
    // random normal in this case 
    if let Some(normal) = normal.try_normalize() {
        return normal;
    } else {
        return random_vec().normalize();
    }
}



#[inline]
pub fn get_big_normal(
    p: Vec4,
    size: f32,
    static_objects: &PhysicsState,
    excluding_ids: Option<ActorID>,
) -> Vec4 {
    let a = p + Vec4::new(size, 0.000, 0.000, 0.000);
    let b = p + Vec4::new(-size, 0.000, 0.000,0.000);
    let c = p + Vec4::new(0.000, size, 0.000, 0.000);
    let d = p + Vec4::new(0.000, -size, 0.000, 0.000);
    let e = p + Vec4::new(0.000, 0.000, size, 0.000);
    let f = p + Vec4::new(0.000, 0.000, -size,0.000);
    let g = p + Vec4::new(0.000, 0.000, 0.000, size);
    let h = p + Vec4::new(0.000, 0.000, 0.000, -size);

    let fa = get_dist(a, static_objects, excluding_ids);
    let fb = get_dist(b, static_objects, excluding_ids);
    let fc = get_dist(c, static_objects, excluding_ids);
    let fd = get_dist(d, static_objects, excluding_ids);
    let fe = get_dist(e, static_objects, excluding_ids);
    let ff = get_dist(f, static_objects, excluding_ids);
    let fg = get_dist(g, static_objects, excluding_ids);
    let fh = get_dist(h, static_objects, excluding_ids);

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



pub fn random_vec() -> Vec4 {
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
