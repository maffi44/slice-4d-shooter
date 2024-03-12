use glam::Vec4;

use crate::{
    actor::{
        device::{
            Device,
            DeviceType,
        }, holegun_hole::HoleGunHole, player::PlayerInnerState, ActorID, ActorWrapper, Message, MessageType
    },
    engine::{
        effects::EffectType,
        physics::PhysicsSystem,
        input::ActionsFrameState,
        engine_handle::{
            Command,
            CommandType,
            EngineHandle,
        },
    }, transform::Transform,
};

pub struct HoleGun {
    charging_time: f32
}


impl HoleGun {
    pub fn new() -> Self {
        HoleGun {
            charging_time: 0.0
        }
    }
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
        engine_handle: &mut EngineHandle,
        delta: f32,
    )
    {
        if input.first_mouse.is_action_pressed() {

            self.charging_time += delta * 1.6;

            if self.charging_time > 3.0 {
                
                shoot(
                    player_id,
                    player,
                    physic_system,
                    engine_handle,
                    self.charging_time
                );

                self.charging_time = 0.0;
            }

           
        } else {

            if self.charging_time > 0.0 {

                shoot(
                    player_id,
                    player,
                    physic_system,
                    engine_handle,
                    self.charging_time
                );

                self.charging_time = 0.0;
            }
        }
    }

}

fn shoot(
    player_id: ActorID,
    player: &mut PlayerInnerState,
    physic_system: &PhysicsSystem,
    engine_handle: &mut EngineHandle,
    charging_time: f32,
) {
    let from = player.transform.get_position() + Vec4::Y * player.collider.get_collider_radius() * 0.98;
            
    let direction = player.transform.rotation.inverse() * Vec4::NEG_Z;
    
    let hit = physic_system.ray_cast(from, direction, 100.0);

    if let Some(hit) = hit {

        let mut hole = HoleGunHole::new(charging_time);

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