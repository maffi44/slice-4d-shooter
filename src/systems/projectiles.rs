
use super::{
    physics::collider::{
        Collider,
        DynamicArea,
    },
    transform::Transform,
};
pub enum ProjectileType {
    Rocket, 
}


const ROCKET_MAX_SPEED: f32 = 1000.0;
const ROCKET_MAX_ACCEL: f32 = 0.0;

pub struct Rocket {
    collision: DynamicArea,
}

impl Rocket {
    fn new(spawn_transform: Transform) -> Self {
        Rocket {
            collision: DynamicArea::new(
                spawn_transform,
            ),
        }
    }
}

impl Projectile for Rocket {
    fn get_collision(&mut self) -> Collider {
        Collider::DynamicArea(&mut self.collision)
    }

    fn tick(&mut self) {
        
    }
}

pub trait Projectile {
    fn get_collision(&mut self) -> Collider;

    fn tick(&mut self) {}
}