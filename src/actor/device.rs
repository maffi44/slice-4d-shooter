pub mod holegun;
pub mod machinegun;

use crate::{
    actor::{
        player::PlayerInnerState,
        ActorID,
    }, engine::{
        audio::AudioSystem, engine_handle::EngineHandle, input::ActionsFrameState, physics::PhysicsSystem, render::VisualElement, ui::UISystem
    }, transform::Transform
};



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
        input: &ActionsFrameState,
        physic_system: &PhysicsSystem,
        audio_system: &mut AudioSystem,
        ui_system: &mut UISystem,
        engine_handle: &mut EngineHandle,
        delta: f32,
    );

    fn get_device_type(&self) -> DeviceType;

    fn get_visual_element<'a>(&'a self, transform: &'a Transform) -> Option<VisualElement<'a>> {
        None
    }

    fn deactivate(
        &mut self,
        player_id: ActorID,
        player: &mut PlayerInnerState,
        physic_system: &PhysicsSystem,
        audio_system: &mut AudioSystem,
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