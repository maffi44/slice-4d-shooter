use crate::{
    actor::{
        ActorID,
    },
    engine::physics::physics_system_data::ShapeType,
};

use glam::Vec4;


#[derive(Clone)]
pub enum StaticColliderMessage {
    
}


#[derive(Debug, Clone)]
pub struct StaticCollider {
    pub position: Vec4,
    pub size: Vec4,
    pub is_positive: bool,
    pub roundness: f32,
    pub stickiness: bool,
    pub friction: f32,
    pub bounce_rate: f32,
    pub shape_type: ShapeType,
    pub undestroyable: bool,
    pub actor_id: Option<ActorID>,
}

impl StaticCollider
{
    pub fn set_id(&mut self, id: ActorID)
    {
        self.actor_id = Some(id);
    }

    pub fn get_id(&self) -> Option<ActorID>
    {
        self.actor_id
    }
}