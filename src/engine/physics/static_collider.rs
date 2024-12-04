use crate::{
    actor::{
        Component,
        ActorID,
    },
    engine::physics::physics_system_data::ShapeType,
};

use glam::Vec4;


#[derive(Clone)]
pub enum StaticColliderMessages {
    
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
    pub actors_id: Option<ActorID>,
}

impl Component for StaticCollider {
    fn set_id(&mut self, id: ActorID) {
        self.actors_id = Some(id);
    }

    fn get_id(&self) -> Option<ActorID> {
        self.actors_id
    }
}