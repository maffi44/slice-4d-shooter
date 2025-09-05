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

use crate::engine::physics::static_collider::StaticCollider;
use glam::{Vec3, Vec4};

#[derive(Debug, Clone, Copy)]
pub struct ObjectMaterial {
    pub color: Vec3,
    pub roughness: f32,
}

impl ObjectMaterial {
    pub fn new(color: Vec3, roughness: f32) -> Self {
        ObjectMaterial {
            color,
            roughness
        }
    }
}

#[derive(Debug)]
pub struct  StaticObject {
    pub collider: StaticCollider,
    pub material_index: i32,
}

#[derive(Clone)]
pub struct WFloor {
    pub w_pos: f32 
}


#[derive(Clone)]
pub struct WRoof {
    pub w_pos: f32 
}



#[derive(Clone)]
pub struct  ColoringArea {
    pub translation: Vec4,
    pub radius: f32,
    pub color: Vec3,
}

#[derive(Clone)]
pub enum VolumeArea {
    SphericalVolumeArea(SphericalVolumeArea),
    BeamVolumeArea(BeamVolumeArea),
}

#[derive(Clone)]
pub struct  SphericalVolumeArea {
    pub translation: Vec4,
    pub radius: f32,
    pub color: Vec3,
}

#[derive(Clone)]
pub struct  BeamVolumeArea {
    pub translation_pos_1: Vec4,
    pub translation_pos_2: Vec4,
    pub radius: f32,
    pub color: Vec3,
}

#[derive(Clone)]
pub struct VisualWave
{
    pub translation: Vec4,
    pub radius: f32,
    pub color: Vec3,
}