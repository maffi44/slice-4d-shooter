use crate::{
    engine::{physics::{
        area::Area, kinematic_collider::KinematicCollider, static_collider::StaticCollider
    }, world::static_object::StaticObject}, transform::Transform
};

use super::dynamic_collider::PlayersDollCollider;



pub struct PhysicalElement<'a> {
    pub transform: &'a mut Transform,
    pub kinematic_collider: Option<(&'a mut KinematicCollider, Option<&'a mut Transform>)>,
    pub dynamic_colliders: Option<&'a mut Vec<PlayersDollCollider>>,
    pub static_colliders: Option<&'a mut Vec<StaticCollider>>,
    pub static_objects: Option<&'a mut Vec<StaticObject>>,
    pub area: Option<&'a mut Area>,
}