// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2026  Timofei Molokov



pub mod holegun;
pub mod obstaclesgun;
pub mod machinegun;
pub mod shotgun;
pub mod rotator;

use crate::{
    actor::{
        main_player::player_inner_state::PlayerInnerState,
        ActorID,
    }, engine::{
        shared_systems::audio::AudioSystem, world::messenger::Messenger, shared_systems::input::ActionsFrameState, world::physics::PhysicsSystem, shared_systems::render::ChildVisualElement, shared_systems::ui::UISystem
    }, engine::transform::Transform
};

use super::main_player::PlayerScreenEffects;



pub struct EmptyHand {
}

impl Default for EmptyHand {
    fn default() -> Self {
        EmptyHand {}
    }
}

impl Device for EmptyHand {
    fn get_device_type(&self) -> DeviceType {
        DeviceType::Gun
    }

    fn process_input(
            &mut self,
            _player_id: ActorID,
            _player: &mut PlayerInnerState,
            _screen_effects: &mut PlayerScreenEffects,
            _input: &ActionsFrameState,
            _physic_system: &PhysicsSystem,
            _audio_system: &mut AudioSystem,
            _ui_system: &mut UISystem,
            _messenger: &mut Messenger,
            _delta: f32,
    ) {
    }
}





pub enum DeviceType {
    Gun,
    #[allow(dead_code)] // game content that is not wired into the current maps/modes; kept for later
    Device,
    RotatorTool,
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
        messenger: &mut Messenger,
        delta: f32,
    );

    fn get_device_type(&self) -> DeviceType;

    fn get_visual_element<'a>(&'a self, _transform: &'a Transform) -> Option<&'a ChildVisualElement> {
        None
    }

    fn deactivate(
        &mut self,
        _player_id: ActorID,
        _player: &mut PlayerInnerState,
        _physic_system: &PhysicsSystem,
        _audio_system: &mut AudioSystem,
        _ui_system: &mut UISystem,
        _messenger: &mut Messenger,
        _screen_effects: &mut PlayerScreenEffects,
    ) {}

    fn activate(
        &mut self,
        _player_id: ActorID,
        _player: &mut PlayerInnerState,
        _physic_system: &PhysicsSystem,
        _audio_system: &mut AudioSystem,
        _ui_system: &mut UISystem,
        _messenger: &mut Messenger,
    ) {}

    fn process_while_player_is_not_alive(
        &mut self,
        _player_id: ActorID,
        _player: &mut PlayerInnerState,
        _input: &ActionsFrameState,
        _physic_system: &PhysicsSystem,
        _audio_system: &mut AudioSystem,
        _ui_system: &mut UISystem,
        _messenger: &mut Messenger,
        _delta: f32,
    ) {}

fn process_while_deactive(
        &mut self,
        _player_id: ActorID,
        _player: &mut PlayerInnerState,
        _input: &ActionsFrameState,
        _physic_system: &PhysicsSystem,
        _audio_system: &mut AudioSystem,
        _ui_system: &mut UISystem,
        _messenger: &mut Messenger,
        _delta: f32,
    ) {}
}