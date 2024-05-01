use glam::Vec4;
use web_sys::js_sys::Math::random;

use crate::{
    actor::{
        machinegun_shot::MachinegunShot, player::{PlayerInnerState, PlayerMessages}, ActorID, ActorWrapper, Message, MessageType, SpecificActorMessage
    },
    engine::{
        engine_handle::{Command, CommandType, EngineHandle}, input::ActionsFrameState, net::{NetCommand, NetMessage, RemoteMessage}, physics::PhysicsSystem, render::VisualElement
    },
    transform::Transform
};

use super::{Device, DeviceType};


const FIRE_RATE: f32 = 0.11;
const MAX_TEMPERTURE: f32 = 60.0;
const MAX_SHOOTING_RANGE: f32 = 0.009;
const SHOOTING_RANGE_INCR_SPEED: f32 = 15.0;
const SHOOTING_RANGE_DCR_SPEED: f32 = 15.0;
const TEMPERATURE_SHOT_INCR: f32 = 4.15;
const TEMPERTURE_TO_DELTA_MULT: f32 = 21.5;
const DAMAGE: u32 = 5;
const FORCE_ON_HIT: f32 = 0.8;

pub struct MachineGun {
    temperature: f32,
    shooting_range: f32,
    time_from_prev_shot: f32,
    is_overheating: bool,

    shooted_from_pivot_point_dir: Vec4,
}

impl MachineGun {
    pub fn new() -> Self {
        let shooted_from_pivot_point_dir = Vec4::new(
            1.0,
            -0.42,
            -1.0,
            0.0
        );

        MachineGun {
            temperature: 0.0,
            shooting_range: 0.0,
            time_from_prev_shot: 0.0,
            is_overheating: false,
            shooted_from_pivot_point_dir,
        }
    }

    fn cool_machinegun(&mut self, delta: f32) {
        if self.temperature > delta * TEMPERTURE_TO_DELTA_MULT {
            self.temperature -= delta * TEMPERTURE_TO_DELTA_MULT;
        } else {
            self.temperature = 0.0;
        }
    }

    fn shoot(
        &mut self,
        player_id: ActorID,
        player: &mut PlayerInnerState,
        physic_system: &PhysicsSystem,
        engine_handle: &mut EngineHandle,
    ) {
        let from = player.transform.get_position() + Vec4::Y * player.collider.get_collider_radius() * 0.98;
                
        let random_dir_y = glam::Mat4::from_rotation_y((random() - 0.5) as f32 * (self.shooting_range));
        let random_dir_x = glam::Mat4::from_rotation_x((random() - 0.5) as f32 * (self.shooting_range));
        
        let forward_dir = random_dir_x * random_dir_y * Vec4::NEG_Z;
        
        let direction = player.transform.rotation.inverse() * forward_dir;
        // direction = random_dir_x * direction;

        let weapon_offset = {
            (Vec4::Y * player.collider.get_collider_radius() * 0.98) +
            (player.transform.rotation.inverse() *
            (self.shooted_from_pivot_point_dir.normalize() * player.collider.get_collider_radius()))
        };

        let hit = physic_system.ray_cast(from, direction, 700.0);

        if let Some(hit) = hit {

            let position = hit.hit_point;
            let shooted_from = player.transform.get_position() + weapon_offset;


            if let Some(hited_id) = hit.hited_actors_id {
                
                let force = hit.hit_normal * -FORCE_ON_HIT;
    
                engine_handle.send_direct_message(
                    hited_id,
                    Message {
                        from: player_id,
                        message: MessageType::SpecificActorMessage(
                            SpecificActorMessage::PLayerMessages(
                                PlayerMessages::DealDamageAndAddForce(
                                    DAMAGE,
                                    force,
                                )
                            )
                        )
                    }
                );
            }

            let shot = MachinegunShot::new(
                position,
                shooted_from,
                1.0,
                1.0,
                false,
            );
    
            engine_handle.send_command(
                Command {
                    sender: player_id,
                    command_type: CommandType::SpawnActor(
                        ActorWrapper::MachinegunShot(shot)
                    )
                }
            );

            engine_handle.send_command(
                Command {
                    sender: player_id,
                    command_type: CommandType::NetCommand(
                        NetCommand::SendBoardcastNetMessageReliable(
                            NetMessage::RemoteDirectMessage(
                                player_id,
                                RemoteMessage::SpawnMachineGunShot(
                                    position.to_array(),
                                    false,
                                )
                            )
                        )
                    )
                }
            )

        } else {
            let position = from + (direction * 700.0);
            let shooted_from = player.transform.get_position() + weapon_offset;

            let shot = MachinegunShot::new(
                position,
                shooted_from,
                1.0,
                1.0,
                true,
            );
    
            engine_handle.send_command(
                Command {
                    sender: player_id,
                    command_type: CommandType::SpawnActor(
                        ActorWrapper::MachinegunShot(shot)
                    )
                }
            );

            engine_handle.send_command(
                Command {
                    sender: player_id,
                    command_type: CommandType::NetCommand(
                        NetCommand::SendBoardcastNetMessageReliable(
                            NetMessage::RemoteDirectMessage(
                                player_id,
                                RemoteMessage::SpawnMachineGunShot(
                                    position.to_array(),
                                    true,
                                )
                            )   
                        )
                    )
                }
            )
        }
    }
}

impl Device for MachineGun {
    fn get_device_type(&self) -> DeviceType {
        DeviceType::Gun
    }

    fn get_visual_element<'a>(&'a self, transform: &'a Transform) -> Option<VisualElement<'a>> {
        None
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
            if input.first_mouse.is_action_pressed() && self.temperature < MAX_TEMPERTURE {
                if self.time_from_prev_shot >= FIRE_RATE {
                    self.shoot(
                        player_id,
                        player,
                        physic_system,
                        engine_handle,
                    );
                    self.temperature += TEMPERATURE_SHOT_INCR;
                    if self.shooting_range < MAX_SHOOTING_RANGE {
                        self.shooting_range += MAX_SHOOTING_RANGE * delta * SHOOTING_RANGE_INCR_SPEED * 1.0/FIRE_RATE;
                    }
                    self.time_from_prev_shot = 0.0;
                } else {
                    self.cool_machinegun(delta);
                    self.time_from_prev_shot += delta;
                }
            } else {
               self.cool_machinegun(delta);
               self.time_from_prev_shot += delta;
               if self.shooting_range > MAX_SHOOTING_RANGE * delta * SHOOTING_RANGE_DCR_SPEED {
                self.shooting_range -= MAX_SHOOTING_RANGE * delta * SHOOTING_RANGE_DCR_SPEED;
            } else {
                self.shooting_range = 0.0;
            }
            }
    }

    fn process_while_player_is_not_alive(
            &mut self,
            player_id: ActorID,
            player: &mut PlayerInnerState,
            input: &ActionsFrameState,
            physic_system: &PhysicsSystem,
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
            engine_handle: &mut EngineHandle,
            delta: f32,
        ) {
            self.cool_machinegun(delta);
            self.time_from_prev_shot += delta;
    }
}