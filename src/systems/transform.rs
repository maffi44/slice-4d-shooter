use glam::{
    Vec4,
    Mat4,
};

#[derive(Debug)]
pub struct Transform {
    pub position: Vec4,
    pub rotation: Mat4,
}

impl Transform {

    pub fn new_zero() -> Self {
        Transform {
            position: Vec4::ZERO,
            rotation: Mat4::IDENTITY,
        }
    }

    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Transform {
            position: Vec4::new(x, y, z, w),
            rotation: Mat4::IDENTITY,
        }
    }

    pub fn increment_position(&mut self, increment: Vec4) {
        self.position += increment;
    }

    pub fn set_position(&mut self, position: Vec4) {
        self.position = position;
    } 

    pub fn get_position(&self) -> Vec4 {
        self.position
    }

    pub fn get_direction(&mut self) -> Vec4 {
        self.rotation * Vec4::Z
    }
}

pub struct Position {

}