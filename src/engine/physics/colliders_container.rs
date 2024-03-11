use crate::{
    engine::{physics::{
        area::Area, kinematic_collider::KinematicCollider, static_collider::StaticCollider
    }, world::static_object::StaticObject}, transform::Transform
};



pub struct PhysicalElement<'a> {
    pub transform: &'a mut Transform,
    pub kinematic_collider: Option<&'a mut KinematicCollider>,
    // pub dynamic_colliders: Option<&'a mut Vec<DynamicCollider>>,
    pub static_colliders: Option<&'a mut Vec<StaticCollider>>,
    pub static_objects: Option<&'a mut Vec<StaticObject>>,
    pub area: Option<&'a mut Area>,
}