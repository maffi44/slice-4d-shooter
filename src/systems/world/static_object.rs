use glam::{Vec3, Vec4};
use super::super::transform::Transform;

#[derive(Debug)]
pub struct StaticObjectData {
    transform: Transform,
    size: Vec4,
    is_positive: bool,
    friction: f32,
    roundness: f32,
    bound_rate: f32,
    material: Vec3,
    // stickiness: f32
}

pub enum StaticObject {
    Cube(StaticObjectData),
    CubeInfW(StaticObjectData),
    Sphere(StaticObjectData),
    SphCube(StaticObjectData),
}