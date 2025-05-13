use client_server_protocol::Team;
use glam::Vec4;

use crate::actor::{
    ActorID,
};

#[derive(Clone)]
pub enum DynamicColliderMessage {
    
}


#[derive(Clone)]
pub struct PlayersDollCollider {
    pub position: Vec4,
    pub radius: f32,
    pub friction: f32,
    pub bounce_rate: f32,
    pub actor_id: Option<ActorID>,
    pub actors_team: Team,
    pub weapon_offset: Vec4,
}

impl PlayersDollCollider
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