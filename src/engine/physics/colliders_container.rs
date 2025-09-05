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

use crate::{
    actor::ActorID, engine::{physics::{
        area::Area, kinematic_collider::KinematicCollider, static_collider::StaticCollider
    }, world::static_object::StaticObject}, transform::Transform
};

use super::dynamic_collider::PlayersDollCollider;



pub struct PhysicalElement<'a> {
    pub id: ActorID,
    pub transform: &'a mut Transform,
    pub kinematic_collider: Option<(&'a mut KinematicCollider, Option<&'a mut Transform>)>,
    pub dynamic_colliders: Option<(&'a mut Vec<PlayersDollCollider>, Team)>,
    pub static_colliders: Option<&'a mut Vec<StaticCollider>>,
    pub static_objects: Option<&'a mut Vec<StaticObject>>,
    pub area: Option<&'a mut Area>,
}