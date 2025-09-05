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

// use super::{
//     physics::collider::{
//         Collider,
//         DynamicArea,
//     },
//     transform::Transform,
// };
// pub enum ProjectileType {
//     Rocket, 
// }


// const ROCKET_MAX_SPEED: f32 = 1000.0;
// const ROCKET_MAX_ACCEL: f32 = 0.0;

// pub struct Rocket {
//     collision: DynamicArea,
// }

// impl Rocket {
//     fn new(spawn_transform: Transform) -> Self {
//         Rocket {
//             collision: DynamicArea::new(
//                 spawn_transform,
//             ),
//         }
//     }
// }

// impl Projectile for Rocket {
//     fn get_collision(&mut self) -> Collider {
//         Collider::DynamicArea(&mut self.collision)
//     }

//     fn tick(&mut self) {
        
//     }
// }

// pub trait Projectile {
//     fn get_collision(&mut self) -> Collider;

//     fn tick(&mut self) {}
// }