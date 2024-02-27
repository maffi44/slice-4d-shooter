use crate::systems::transform::Transform;

use super::{
    kinematic_collider::KinematicCollider,
    // dynamic_collider::DynamicCollider,
    static_collider::StaticCollider,
    area::Area,
};

pub struct PhysicalElement<'a> {
    pub transform: &'a mut Transform,
    pub kinematic_collider: Option<&'a mut KinematicCollider>,
    // pub dynamic_colliders: Option<&'a mut Vec<DynamicCollider>>,
    pub static_colliders: Option<&'a mut Vec<StaticCollider>>,
    pub area: Option<&'a mut Area>,
}