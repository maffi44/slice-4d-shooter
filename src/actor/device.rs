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

pub mod holegun;
pub mod obstaclesgun;
pub mod machinegun;
pub mod shotgun;

use crate::{
    actor::{
        main_player::player_inner_state::PlayerInnerState,
        ActorID,
    }, engine::{
        audio::AudioSystem, engine_handle::EngineHandle, input::ActionsFrameState, physics::PhysicsSystem, render::ChildVisualElement, ui::UISystem
    }, transform::Transform
};

use super::main_player::PlayerScreenEffects;



const DEFAULT_PISTOL_DAMAGE: u32 = 5;

pub struct DefaultPointer {
    damage: u32,

}

impl Default for DefaultPointer {
    fn default() -> Self {
        DefaultPointer {
            damage: DEFAULT_PISTOL_DAMAGE,
        }
    }
}

impl Device for DefaultPointer {
    fn get_device_type(&self) -> DeviceType {
        DeviceType::Gun
    }

    fn process_input(
            &mut self,
            player_id: ActorID,
            player: &mut PlayerInnerState,
            screen_effects: &mut PlayerScreenEffects,
            input: &ActionsFrameState,
            physic_system: &PhysicsSystem,
            audio_system: &mut AudioSystem,
            ui_system: &mut UISystem,
            engine_handle: &mut EngineHandle,
            delta: f32,
    ) {
        // TODO - make pointing
    }
}





pub enum DeviceType {
    Gun,
    Device,
}

pub trait Device {

    fn process_input(
        &mut self,
        player_id: ActorID,
        player: &mut PlayerInnerState,
        screen_effects: &mut PlayerScreenEffects,
        input: &ActionsFrameState,
        physic_system: &PhysicsSystem,
        audio_system: &mut AudioSystem,
        ui_system: &mut UISystem,
        engine_handle: &mut EngineHandle,
        delta: f32,
    );

    fn get_device_type(&self) -> DeviceType;

    fn get_visual_element<'a>(&'a self, transform: &'a Transform) -> Option<&'a ChildVisualElement> {
        None
    }

    fn deactivate(
        &mut self,
        player_id: ActorID,
        player: &mut PlayerInnerState,
        physic_system: &PhysicsSystem,
        audio_system: &mut AudioSystem,
        ui_system: &mut UISystem,
        engine_handle: &mut EngineHandle,
        screen_effects: &mut PlayerScreenEffects,
    ) {}

    fn activate(
        &mut self,
        player_id: ActorID,
        player: &mut PlayerInnerState,
        physic_system: &PhysicsSystem,
        audio_system: &mut AudioSystem,
        ui_system: &mut UISystem,
        engine_handle: &mut EngineHandle,
    ) {}

    fn process_while_player_is_not_alive(
        &mut self,
        player_id: ActorID,
        player: &mut PlayerInnerState,
        input: &ActionsFrameState,
        physic_system: &PhysicsSystem,
        audio_system: &mut AudioSystem,
        ui_system: &mut UISystem,
        engine_handle: &mut EngineHandle,
        delta: f32,
    ) {}

fn process_while_deactive(
        &mut self,
        player_id: ActorID,
        player: &mut PlayerInnerState,
        input: &ActionsFrameState,
        physic_system: &PhysicsSystem,
        audio_system: &mut AudioSystem,
        ui_system: &mut UISystem,
        engine_handle: &mut EngineHandle,
        delta: f32,
    ) {}
}