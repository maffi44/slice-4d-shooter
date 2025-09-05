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