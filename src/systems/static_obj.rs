use glam::Vec4;
use super::transform::Transform;

#[derive(Debug)]
pub enum StaticObject {
    Cube(Transform, Vec4, bool),
    CubeInfW(Transform, Vec4, bool),
    Sphere(Transform, Vec4, bool),
    SphCube(Transform, Vec4, bool),
}

