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

use glam::{Vec3, Vec4};

use crate::{
    actor::{
        wave::Wave,
        ActorWrapper
    },
    engine::engine_handle::{
        Command,
        CommandType
    }
};

use super::engine_handle::EngineHandle;


pub enum EffectType {
    DefaultPistolDecal,
}

pub struct EffectsSystem
{

}

impl EffectsSystem
{
    pub fn new() -> Self
    {
        EffectsSystem
        {

        }
    }

    pub fn spawn_wave(
        &mut self,
        engine_handle: &mut EngineHandle,
        position: Vec4,
        radii: Vec<f32>,
        colors: Vec<Vec3>,
        time_segments: Vec<f32>,
    )
    {
        assert!(
            radii.len() >= 2
        );

        assert!(
            radii.len() == colors.len() &&
            radii.len() == (time_segments.len() + 1)
        );

        let wave = Wave::new(
            position,
            radii,
            colors,
            time_segments
        );

        engine_handle.send_command(
            Command {
                sender: 0u128,
                command_type: CommandType::SpawnActor(
                    ActorWrapper::Wave(wave)
                )
            }
        );
    }
}