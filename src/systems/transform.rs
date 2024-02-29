use glam::{
    Vec4,
    Mat4,
};

#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub position: Vec4,
    pub rotation: Mat4,
    pub scale: Vec4,
}

impl Transform {

    #[inline]
    pub fn new_zero() -> Self {
        Transform {
            position: Vec4::ZERO,
            rotation: Mat4::IDENTITY,
            scale: Vec4::ONE,
        }
    }

    #[inline]
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Transform {
            position: Vec4::new(x, y, z, w),
            rotation: Mat4::IDENTITY,
            scale: Vec4::ONE,

        }
    }

    #[inline]
    pub fn new_from_pos(position: Vec4) -> Self {
        Transform {
            position,
            rotation: Mat4::IDENTITY,
            scale: Vec4::ONE,
        }
    }

    #[inline]
    pub fn new_from_pos_and_scale(position: Vec4, scale: Vec4) -> Self {
        Transform {
            position,
            rotation: Mat4::IDENTITY,
            scale,
        }
    }

    #[inline]
    pub fn increment_position(&mut self, increment: Vec4) {
        self.position += increment;
    }

    #[inline]
    pub fn increment_scale(&mut self, increment: Vec4) {
        self.scale += increment;
    }

    #[inline]
    pub fn set_position(&mut self, position: Vec4) {
        self.position = position;
    } 

    #[inline]
    pub fn get_position(&self) -> Vec4 {
        self.position
    }

    #[inline]
    pub fn set_scale(&mut self, scale: Vec4) {
        self.scale = scale;
    }

    #[inline]
    pub fn get_scale(&self) -> Vec4 {
        self.scale
    }

    #[inline]
    pub fn get_direction(&mut self) -> Vec4 {
        self.rotation * Vec4::Z
    }
}

pub struct Position {

}