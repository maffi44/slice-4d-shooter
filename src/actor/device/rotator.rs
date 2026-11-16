// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2026  Timofei Molokov



use crate::{
    actor::{
        main_player::{
            player_inner_state::PlayerInnerState,
            PlayerScreenEffects,
        }, ActorID
    },
    engine::{
        shared_systems::audio::AudioSystem,
        world::messenger::Messenger,
        shared_systems::input::ActionsFrameState,
        world::physics::PhysicsSystem,
        shared_systems::render::ChildVisualElement,
        shared_systems::ui::UISystem
    },
    engine::transform::Transform
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

    fn get_visual_element<'a>(&'a self, _transform: &'a Transform) -> Option<&'a ChildVisualElement> {
        None
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
        ) {
        
    }

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

    fn deactivate(
        &mut self,
        _player_id: ActorID,
        _player: &mut PlayerInnerState,
        _physic_system: &PhysicsSystem,
        _audio_system: &mut AudioSystem,
        _ui_system: &mut UISystem,
        _messenger: &mut Messenger,
        _screen_effects: &mut PlayerScreenEffects,
    ) {
        // let img = ui_system.get_mut_ui_element(&UIElementType::RotatorDraft);

        // if let UIElement::Image(img) = img {
        //     img.ui_data.is_visible = false;
        // }

    }

    fn activate(
        &mut self,
        _player_id: ActorID,
        _player: &mut PlayerInnerState,
        _physic_system: &PhysicsSystem,
        _audio_system: &mut AudioSystem,
        _ui_system: &mut UISystem,
        _messenger: &mut Messenger,
    )
    {
        // let img = ui_system.get_mut_ui_element(&UIElementType::RotatorDraft);

        // if let UIElement::Image(img) = img {
        //     img.ui_data.is_visible = true;
        // }
    }
}