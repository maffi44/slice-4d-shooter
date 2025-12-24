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
        main_player::{
            player_inner_state::PlayerInnerState,
            PlayerScreenEffects,
        }, shotgun_shot_source::ShotgunShotSource, ActorID, ActorWrapper
    },
    engine::{
        audio::AudioSystem,
        engine_handle::{
            Command,
            CommandType,
            EngineHandle
        },
        input::ActionsFrameState,
        physics::PhysicsSystem,
        render::ChildVisualElement,
        ui::{
            UIElement,
            UIElementType,
            UISystem
        }
    },
    transform::{Transform, FORWARD}
};

use client_server_protocol::{
    NetCommand, NetMessageToPlayer, RemoteMessage, Team
};

use super::{Device, DeviceType};


pub struct RotatorTool {}

impl RotatorTool {
    pub fn new() -> Self {
        RotatorTool {}
    }
}

impl Device for RotatorTool {
    fn get_device_type(&self) -> DeviceType {
        DeviceType::RotatorTool
    }

    fn get_visual_element<'a>(&'a self, transform: &'a Transform) -> Option<&'a ChildVisualElement> {
        None
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
        ) {
        
    }

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

    fn deactivate(
        &mut self,
        player_id: ActorID,
        player: &mut PlayerInnerState,
        physic_system: &PhysicsSystem,
        audio_system: &mut AudioSystem,
        ui_system: &mut UISystem,
        engine_handle: &mut EngineHandle,
        screen_effects: &mut PlayerScreenEffects,
    ) {
        // let img = ui_system.get_mut_ui_element(&UIElementType::RotatorDraft);

        // if let UIElement::Image(img) = img {
        //     img.ui_data.is_visible = false;
        // }

    }

    fn activate(
        &mut self,
        player_id: ActorID,
        player: &mut PlayerInnerState,
        physic_system: &PhysicsSystem,
        audio_system: &mut AudioSystem,
        ui_system: &mut UISystem,
        engine_handle: &mut EngineHandle,
    )
    {
        // let img = ui_system.get_mut_ui_element(&UIElementType::RotatorDraft);

        // if let UIElement::Image(img) = img {
        //     img.ui_data.is_visible = true;
        // }
    }
}