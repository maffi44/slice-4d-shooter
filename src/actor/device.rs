pub mod holegun;

use crate::{
    actor::{
        player::PlayerInnerState,
        ActorID,
    }, engine::{
        engine_handle::EngineHandle,
        input::ActionsFrameState, physics::PhysicsSystem,
    }
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
            engine_handle: &mut EngineHandle
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
        engine_handle: &mut EngineHandle
    );

    fn get_device_type(&self) -> DeviceType;
}