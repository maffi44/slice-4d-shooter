use glam::Vec4;
use super::transform::Transform;

#[derive(Debug)]
pub enum StaticObject {
    Cube(Transform, Vec4),
    CubeInfW(Transform, Vec4),
    Sphere(Transform, Vec4),
}

