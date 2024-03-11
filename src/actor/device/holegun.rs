use glam::Vec4;
use wgpu::util::DrawIndirectArgs;

use crate::{
    actor::{
        device::{
            Device,
            DeviceType,
        }, holegun_hole::HoleGunHole, player::PlayerInnerState, ActorID, ActorWrapper, Message, MessageType
    },
    engine::{
        effects::EffectType, engine_handle::{
            Command,
            CommandType,
            EngineHandle,
        }, input::ActionsFrameState, physics::PhysicsSystem
    }, transform::Transform,
};

pub struct HoleGun {

}

impl Device for HoleGun {
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
    )
    {
        if input.first_mouse.is_action_just_pressed() {
            log::warn!("HOLEGUN PRESSED");

            let from = player.transform.get_position() + Vec4::Y * player.collider.get_collider_radius() * 0.98;
            
            let direction = player.transform.rotation.inverse() * Vec4::NEG_Z;
            
            let hit = physic_system.ray_cast(from, direction, 100.0);

            if let Some(hit) = hit {
                log::warn!("HOLEGUN HIT");

                let mut hole = HoleGunHole::new();

                hole.set_transform(Transform::new_from_pos(hit.hit_point));

                engine_handle.send_command(
                    Command {
                        sender: player_id,
                        command_type: CommandType::SpawnActor(
                            ActorWrapper::HoleGunHole(hole)
                        )
                    }
                );
            }
        }
    }
}