use glam::{
    Vec4,
    Mat4
};
use alkahest::alkahest;

#[derive(Debug, Clone, Copy)]
pub struct Transform {
    position: Vec4,
    rotation: Mat4,
    scale: Vec4,
}

#[repr(C)]
#[alkahest(Formula, Serialize, Deserialize)]
#[derive(Clone)]
pub struct SerializableTransform {
    p: [f32; 4],
    r: [f32; 16],
    s: [f32; 4],
}

impl SerializableTransform {
    pub fn from_transform(transform: &Transform) -> Self {
        let p = transform.position.to_array();
        let r = transform.rotation.to_cols_array();
        let s = transform.scale.to_array();

        SerializableTransform {
            p, r, s
        }
    }
}

impl Transform {
    pub fn from_serializable_transform(tr: SerializableTransform) -> Self {
        Transform {
            position: Vec4::from_array(tr.p),
            rotation: Mat4::from_cols_array(&tr.r),
            scale: Vec4::from_array(tr.s),
        }
    }

    pub fn to_serializable_transform(&self) -> SerializableTransform {
        let p = self.position.to_array();
        let r = self.rotation.to_cols_array();
        let s = self.scale.to_array();

        SerializableTransform {
            p, r, s
        }
    }
    
    #[inline]
    pub fn new() -> Self {
        Transform {
            position: Vec4::ZERO,
            rotation: Mat4::IDENTITY,
            scale: Vec4::ONE,
        }
    }

    #[inline]
    pub fn from_coords(x: f32, y: f32, z: f32, w: f32) -> Self {
        Transform {
            position: Vec4::new(x, y, z, w),
            rotation: Mat4::IDENTITY,
            scale: Vec4::ONE,

        }
    }

    #[inline]
    pub fn from_position(position: Vec4) -> Self {
        Transform {
            position,
            rotation: Mat4::IDENTITY,
            scale: Vec4::ONE,
        }
    }

    #[inline]
    pub fn from_position_and_scale(position: Vec4, scale: Vec4) -> Self {
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
    pub fn set_rotation(&mut self, rotation: Mat4) {
        self.rotation = rotation;
    } 

    #[inline]
    pub fn set_scale(&mut self, scale: Vec4) {
        self.scale = scale;
    }
    
    #[inline]
    pub fn get_position(&self) -> Vec4 {
        self.position
    }

    #[inline]
    pub fn get_scale(&self) -> Vec4 {
        self.scale
    }

    #[inline]
    pub fn get_rotation(&self) -> Mat4 {
        self.rotation
    }

    #[inline]
    pub fn get_direction_for_audio_system(&self) -> Vec4 {
        self.rotation.inverse() * Vec4::Z
    }
}