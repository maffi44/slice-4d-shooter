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
pub struct  ColoringArea {
    pub translation: Vec4,
    pub radius: f32,
    pub color: Vec3,
}

#[derive(Clone)]
pub struct  VolumeArea {
    pub translation: Vec4,
    pub radius: f32,
    pub color: Vec3,
}