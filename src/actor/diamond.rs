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

// use crate::systems::{
//     physics::collider::Area,
//     engine_handle::EngineHandle,
//     transform::Transform,
// };

// use super::{
//     Actor,
//     ActorID,
//     Message,
// };

// const DIAMOND_COLLIDER_RADIUS: f32 = 0.2;

// pub struct Diamond {
//     id: Option<ActorID>,
//     collider: Area,
// }

// impl Diamond {
//     pub fn new(transform: Transform) -> Self {
//         let collider = Area::new(transform, DIAMOND_COLLIDER_RADIUS);

//         Diamond {
//             id: None,
//             collider,
//         }
//     }
// }

// impl Actor for Diamond {
//     fn recieve_message(&mut self, message: Message, engine_handle: &mut EngineHandle) {
//         let from = message.from;

//         let message = message.message;
        
//         // match message {
//         //     MessageType::DealDamage(damage) => {
//         //         self.inner_state.hp -= damage as i32;
//         //     },
//         //     MessageType::SetTransform(transform) => {
//         //         self.inner_state.collider.transform = transform;
//         //     }
//         //     MessageType::EnableCollider(enable) => {
//         //         self.inner_state.collider.is_enable = enable;
//         //     }
//         // }
//     }

//     fn recieve_boardcast_message(&mut self, message: &Message, engine_handle: &mut EngineHandle) {
//         let from = message.from;

//         let message = &message.message;
        
//         // match message {
//         //     MessageType::DealDamage(damage) => {
//         //         self.inner_state.hp -= *damage as i32;
//         //     },
//         //     MessageType::SetTransform(transform) => {
//         //         self.inner_state.collider.transform = transform.clone();
//         //     }
//         //     MessageType::EnableCollider(enable) => {
//         //         self.inner_state.collider.is_enable = *enable;
//         //     }
//         // }
//     }

//     fn set_id(&mut self, id: ActorID) {
//         self.id = Some(id);
//     }

//     fn get_id(&self) -> Option<ActorID> {
//         self.id
//     }
// }