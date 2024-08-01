use crate::engine::physics::static_collider::StaticCollider;
use glam::{Vec3, Vec4};

#[derive(Debug, Clone, Copy)]
pub struct ObjectMaterial {
    pub color: Vec3,
    pub roughness: f32,
}

impl ObjectMaterial {
    pub fn new(color: Vec3, roughness: f32) -> Self {
        ObjectMaterial {
            color,
            roughness
        }
    }
}

#[derive(Debug)]
pub struct  StaticObject {
    pub collider: StaticCollider,
    pub material_index: i32,
}

#[derive(Clone)]
pub struct WFloor {
    pub w_pos: f32 
}


#[derive(Clone)]
pub struct WRoof {
    pub w_pos: f32 
}



#[derive(Clone)]
pub struct  ColoringArea {
    pub translation: Vec4,
    pub radius: f32,
    pub color: Vec3,
}

#[derive(Clone)]
pub enum VolumeArea {
    SphericalVolumeArea(SphericalVolumeArea),
    BeamVolumeArea(BeamVolumeArea),
}

#[derive(Clone)]
pub struct  SphericalVolumeArea {
    pub translation: Vec4,
    pub radius: f32,
    pub color: Vec3,
}

#[derive(Clone)]
pub struct  BeamVolumeArea {
    pub translation_pos_1: Vec4,
    pub translation_pos_2: Vec4,
    pub radius: f32,
    pub color: Vec3,
}