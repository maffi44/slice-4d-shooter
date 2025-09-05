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

use glam::{FloatExt, Vec3, Vec4};

use crate::{engine::{audio::AudioSystem, effects::EffectsSystem, engine_handle::{Command, CommandType, EngineHandle}, physics::PhysicsSystem, render::VisualElement, time::TimeSystem, ui::UISystem, world::static_object::VisualWave}, transform::Transform};

use super::{Actor, ActorID};


pub struct Wave
{
    transform: Transform,
    id: Option<ActorID>,
    visual_waves: Vec<VisualWave>,
    radii: Vec<f32>,
    colors: Vec<Vec3>,
    time_segments: Vec<f32>,

    current_time: f32,
}

impl Wave
{
    pub fn new(
        position: Vec4,
        radii: Vec<f32>,
        colors: Vec<Vec3>,
        time_segments: Vec<f32>,
    ) -> Self
    {
        assert!(
            radii.len() >= 2
        );

        assert!(
            radii.len() == colors.len() &&
            radii.len() == (time_segments.len() + 1)
        );

        let visual_wave = VisualWave {
            translation: Vec4::ZERO,
            radius: radii[0],
            color: colors[0],
        };

        let visual_waves = vec![visual_wave];

        Wave {
            transform: Transform::from_position(position),
            id: None,
            visual_waves,
            radii,
            colors,
            time_segments,

            current_time: 0.0,
        }
    }
}

impl Actor for Wave
{
    fn get_mut_transform(&mut self) -> &mut Transform {
        &mut self.transform
    }

    fn get_transform(&self) -> &Transform {
        &self.transform
    }

    fn get_id(&self) -> Option<ActorID> {
        self.id
    }

    fn set_id(&mut self, id: ActorID) {
        self.id = Some(id);
    }

    fn tick(
        &mut self,
        physic_system: &PhysicsSystem,
        engine_handle: &mut EngineHandle,
        audio_system: &mut AudioSystem,
        ui_system: &mut UISystem,
        time_system: &mut TimeSystem,
        effects_system: &mut EffectsSystem,
        delta: f32
    )
    {
        if self.time_segments.len() == 0
        {
            engine_handle.send_command(
                Command {
                    sender: self.id.expect("Wave is not have ActorID"),
                    command_type: CommandType::RemoveActor(
                        self.id.expect("Wave is not have ActorID")
                    )
                }
            );
            return;
        }

        self.current_time += delta;

        if self.current_time >= self.time_segments[0]
        {
            self.current_time = 0.0;
            
            self.time_segments.remove(0);
            self.colors.remove(0);
            self.radii.remove(0);

            if self.time_segments.len() == 0
            {
                engine_handle.send_command(
                    Command {
                        sender: self.id.expect("Wave is not have ActorID"),
                        command_type: CommandType::RemoveActor(
                            self.id.expect("Wave is not have ActorID")
                        )
                    }
                );
                return;
            }
        }

        let coef = self.current_time / self.time_segments[0];

        let radius = self.radii[0].lerp(self.radii[1], coef);
        let color = self.colors[0].lerp(self.colors[1], coef);

        self.visual_waves[0].color = color;
        self.visual_waves[0].radius = radius;
    }

    fn get_visual_element(&self) -> Option<VisualElement> {
        Some(VisualElement {
            transform: &self.transform,
            static_objects: None,
            coloring_areas: None,
            volume_areas: None,
            waves: Some(&self.visual_waves),
            player: None,
            child_visual_elem: None,
        })
    }
}