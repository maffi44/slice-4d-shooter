use glam::Vec4;

use crate::actor::{
    Component,
    ActorID,
};

pub enum DynamicColliderMessages {
    
}


#[derive(Clone)]
pub struct PlayersDollCollider {
    pub position: Vec4,
    pub radius: f32,
    pub friction: f32,
    pub bounce_rate: f32,
    pub actors_id: Option<ActorID>,
    pub weapon_offset: Vec4,
}

// impl DynamicCollider {
//     pub fn new(radius: f32) -> Self {
//         DynamicCollider {
//             actors_id: None,
//             radius,
//         }
//     }
// }

impl Component for PlayersDollCollider {
    fn set_id(&mut self, id: ActorID) {
        self.actors_id = Some(id);
    }

    fn get_id(&self) -> Option<ActorID> {
        let id = self.actors_id.expect("DynamicCollider is not initialized");

        Some(id)
    }
}