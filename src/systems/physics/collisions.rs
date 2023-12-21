use glam::Vec4;

use super::super::transform::Transform;

pub enum Collision<'a> {
    Static(&'a mut StaticCollision),
    Dynamic(&'a mut DynamicCollision),
    StaticArea(&'a mut StaticArea),
    DynamicArea(&'a mut DynamicArea),
}

pub struct StaticCollision {}

pub struct DynamicCollision {
    pub transform: Transform,
    max_speed: f32,
    max_accel: f32,
    wish_direction: Vec4,
    current_velocity: Vec4,
}

impl DynamicCollision {
    pub fn new(transform: Transform, max_speed: f32, max_accel: f32,) -> Self {
        DynamicCollision {
            transform,
            max_speed,
            max_accel,
            wish_direction: Vec4::ZERO,
            current_velocity: Vec4::ZERO,
        }
    }

    pub fn set_wish_direction(&mut self, wish_direction: Vec4) {
        self.wish_direction = wish_direction
    }

    pub fn physics_tick(&mut self, delta: f32) {

        if self.wish_direction.length() > 0.0 {
            // self.wish_direction = self.wish_direction.normalize();

            let current_speed_in_wishdir = self.current_velocity.dot(self.wish_direction);

            let speed = self.max_speed - current_speed_in_wishdir;

            let add_speed = 0.0_f32.max(speed.min(self.max_accel * delta));

            self.current_velocity += self.wish_direction * add_speed;
        }

        self.transform.increment_position(self.current_velocity * delta);
        self.current_velocity *= 1.0 - delta*4.0;
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


