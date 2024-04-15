use crate::engine::physics::static_collider::StaticCollider;
use glam::{Vec3, Vec4};

#[derive(Debug, Clone, Copy)]
pub struct ObjectMatrial {
    pub color: Vec3
}

impl ObjectMatrial {
    pub fn new(color: Vec3) -> Self {
        ObjectMatrial {
            color
        }
    }
}

#[derive(Debug)]
pub struct  StaticObject {
    pub collider: StaticCollider,
    pub material: ObjectMatrial,
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