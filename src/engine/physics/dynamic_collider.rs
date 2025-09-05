// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2025  Timofei Molokov

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

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