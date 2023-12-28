use glam::Vec4;
use super::transform::Transform;

#[derive(Debug)]
pub enum StaticObject {
    Cube(Transform, Vec4)
}
