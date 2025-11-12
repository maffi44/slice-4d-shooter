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

use glam::{
    Vec4,
    Mat4
};

#[derive(Debug, Clone, Copy)]
pub struct Transform {
    position: Vec4,
    rotation: Mat4,
    scale: Vec4,
}

pub const FORWARD: Vec4 = Vec4::NEG_Z;
pub const BACKWARD: Vec4 = Vec4::Z;
pub const RIGHT: Vec4 = Vec4::X;
pub const LEFT: Vec4 = Vec4::NEG_X;
pub const UP: Vec4 = Vec4::Y;
pub const DOWN: Vec4 = Vec4::NEG_Y;
pub const W_UP: Vec4 = Vec4::W;
pub const W_DOWN: Vec4 = Vec4::NEG_W;
pub const ATA: Vec4 = Vec4::W;
pub const KETA: Vec4 = Vec4::NEG_W;

pub type SerializableTransform = ([f32; 4], [f32; 16]);

impl Transform {
    pub fn from_serializable_transform(tr: SerializableTransform) -> Self {
        Transform {
            position: Vec4::from_array(tr.0),
            rotation: Mat4::from_cols_array(&tr.1),
            scale: Vec4::ONE,
        }
    }

    pub fn to_serializable_transform(&self) -> SerializableTransform {
        let p = self.position.to_array();
        let r = self.rotation.to_cols_array();
        // let s = self.scale.to_array();

        (p, r)
    }
    
    #[inline]
    pub fn new() -> Self {
        Transform {
            position: Vec4::ZERO,
            rotation: Mat4::IDENTITY,
            scale: Vec4::ONE,
        }
    }

    #[inline]
    pub fn from_position_coords(x: f32, y: f32, z: f32, w: f32) -> Self {
        Transform {
            position: Vec4::new(x, y, z, w),
            rotation: Mat4::IDENTITY,
            scale: Vec4::ONE,

        }
    }

    #[inline]
    pub fn from_position(position: Vec4) -> Self {
        Transform {
            position,
            rotation: Mat4::IDENTITY,
            scale: Vec4::ONE,
        }
    }

    #[inline]
    pub fn from_position_and_scale(position: Vec4, scale: Vec4) -> Self {
        Transform {
            position,
            rotation: Mat4::IDENTITY,
            scale,
        }
    }

    #[inline]
    pub fn increment_position(&mut self, increment: Vec4) {
        self.position += increment;
    }

    #[inline]
    pub fn increment_scale(&mut self, increment: Vec4) {
        self.scale += increment;
    }

    #[inline]
    pub fn set_position(&mut self, position: Vec4) {
        self.position = position;
    }

    #[inline]
    pub fn set_rotation(&mut self, rotation: Mat4) {
        self.rotation = rotation;
    } 

    #[inline]
    pub fn set_scale(&mut self, scale: Vec4) {
        self.scale = scale;
    }
    
    #[inline]
    pub fn get_position(&self) -> Vec4 {
        self.position
    }

    #[inline]
    pub fn get_scale(&self) -> Vec4 {
        self.scale
    }

    #[inline]
    pub fn get_rotation(&self) -> Mat4 {
        self.rotation
    }

    #[inline]
    pub fn get_direction_for_audio_system(&self) -> Vec4 {
        self.rotation * BACKWARD
    }
}