use crate::{
    engine::{
        engine_handle::{
            EngineHandle,
            Command,
            CommandType,
        },
        input::ActionsFrameState,
        effects::EffectType,
    },
    actor::{
        player::PlayerInnerState,
        ActorID,
        Message,
        MessageType
    },
};



const DEFAULT_PISTOL_DAMAGE: u32 = 5;

pub struct DefaultPistol {
    damage: u32,

}

impl Default for DefaultPistol {
    fn default() -> Self {
        DefaultPistol {
            damage: DEFAULT_PISTOL_DAMAGE,
        }
    }
}

impl Device for DefaultPistol {
    fn get_device_type(&self) -> DeviceType {
        DeviceType::Gun
    }

    fn process_input(
            &mut self,
            player_id: ActorID,
            player: &mut PlayerInnerState,
            input: &ActionsFrameState,
            engine_handle: &mut EngineHandle
        ) {
        // if input.fire.is_action_just_pressed() {
        //     let hit = engine_handle.physics_state.ray_cast(
        //         player.collider.transform.get_position(),
        //         player.collider.transform.get_direction(),
        //         1000.0
        //     );

        //     if let Some(hit) = hit {
        //         if let Some(id) = hit.hited_players_id {
        //             engine_handle.send_direct_message(
        //                 id,
        //                 // send message to the damaged player
        //                 Message {
        //                     from: player_id,
        //                     message: MessageType::DealDamage(self.damage)
        //                 }
        //             )
        //         } else {
        //             engine_handle.send_command(
        //                 // spawn shoot effect on point
        //                 Command {
        //                     sender: player_id,
        //                     command_type: CommandType::SpawnEffect(EffectType::DefaultPistolDecal),
        //                 }
        //            )
        //         }
        //     }
        // }
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
        engine_handle: &mut EngineHandle
    ) {}

    fn get_device_type(&self) -> DeviceType;
}