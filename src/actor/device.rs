pub mod holegun;

use crate::{
    actor::{
        player::PlayerInnerState,
        ActorID,
    }, engine::{
        engine_handle::EngineHandle,
        input::ActionsFrameState, physics::PhysicsSystem, render::VisualElement,
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
        engine_handle: &mut EngineHandle,
        delta: f32,
    );

    fn get_device_type(&self) -> DeviceType;

    fn get_visual_element<'a>(&'a self, transform: &'a Transform) -> Option<VisualElement<'a>> {
        None
    }

    fn process_while_player_is_not_alive(
        &mut self,
        player_id: ActorID,
        player: &mut PlayerInnerState,
        input: &ActionsFrameState,
        physic_system: &PhysicsSystem,
        engine_handle: &mut EngineHandle,
        delta: f32,
    ) {}
}