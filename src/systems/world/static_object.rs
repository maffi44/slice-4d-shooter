use glam::{Vec3, Vec4};
use super::super::transform::Transform;

#[derive(Debug)]
pub struct ObjectMatrial {
    color: Vec3
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
    pub shape_type: ShapeType,
    pub transform: Transform,
    pub size: Vec4,
    pub is_positive: bool,
    pub friction: f32,
    pub roundness: f32,
    pub bounce_rate: f32,
    pub material: ObjectMatrial,
    pub stickiness: f32
}

#[derive(Debug)]
pub enum ShapeType {
    Cube,
    CubeInfW,
    Sphere,
    SphCube,
}