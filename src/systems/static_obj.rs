use glam::Vec4;
use super::transform::Transform;

pub enum StaticObject {
    Cube(Transform, Vec4)
}
