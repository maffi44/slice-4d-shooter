use glam::{
    Vec2,
    Vec3,
    Vec4,
    Vec4Swizzles,
    FloatExt,
};

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
    let d1: f32 = p.xy().length() - b.x;
    let d2: f32 = p.xz().length() - b.y;
    let d3: f32 = p.yz().length() - b.z;
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

#[inline]
fn smin(a: f32, b: f32, k: f32) -> f32
{
    let x = (b-a)/k;
    let g = 0.5*(x-(x*x+0.25).sqrt());
    return a + k * g;
}


#[inline]
pub fn get_dist(
    p: Vec4,
    static_objects: &PhysicsState,
) -> f32 {
    let mut d = MAX_DIST;

    let stickiness = static_objects.stickiness;

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

    for collider in static_objects.dyn_spheres.iter() {
        d = d.min(sd_sphere(p - collider.position.clone(), collider.radius));
    }

    return d;
}


#[inline]
pub fn get_bounce_and_friction(
    p: Vec4,
    static_objects: &PhysicsState,
) -> (f32, f32) {
    let mut d = MAX_DIST;

    let mut bounce_coeficient = 0.0;
    let mut friction = 0.0;

    let stickiness = static_objects.stickiness;

    for collider in static_objects.cubes.iter_stickiness() {
        let new_d = sd_box(p - collider.position.clone(), collider.size.clone()) - collider.roundness;
        
        let dd = smin(d, new_d, stickiness);
        
        let coef = ((new_d - d) / (dd - d)).clamp(0.0, 1.0);
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
        
        let coef = ((new_d - d) / (dd - d)).clamp(0.0, 1.0);
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
        
        let coef = ((new_d - d) / (dd - d)).clamp(0.0, 1.0);
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
        
        let coef = ((new_d - d) / (dd - d)).clamp(0.0, 1.0);
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

    for collider in static_objects.dyn_spheres.iter() {
        let new_d = sd_sphere(p - collider.position.clone(), collider.radius);

        if new_d < d {
            bounce_coeficient = collider.bounce_rate;
            friction = collider.friction;

            d = new_d;
        };
    }

    (bounce_coeficient, friction)
}



#[inline]
pub fn get_normal(
    p: Vec4,
    static_objects: &PhysicsState,
) -> Vec4 {
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
) -> Vec4 {
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
