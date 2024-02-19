use glam::Vec4;

use super::physics_system_data::ShapeType;



#[derive(Debug, Clone)]
pub struct StaticCollider {
    pub position: Vec4,
    pub size: Vec4,
    pub is_positive: bool,
    pub roundness: f32,
    pub stickiness: f32,
    pub friction: f32,
    pub bounce_rate: f32,
    pub shape_type: ShapeType
}