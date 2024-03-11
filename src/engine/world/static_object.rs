use crate::engine::physics::static_collider::StaticCollider;
use glam::Vec3;

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
pub struct StaticObject {
    pub collider: StaticCollider,
    pub material: ObjectMatrial,
}

