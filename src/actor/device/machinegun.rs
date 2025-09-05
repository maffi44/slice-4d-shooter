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

use glam::{Mat4, Vec4};

use crate::{
    actor::{
        machinegun_shot::MachinegunShot,
        main_player::{
            player_inner_state::PlayerInnerState,
            PlayerMessage, PlayerScreenEffects
        },
        ActorID,
        ActorWrapper,
        Message,
        MessageType,
        SpecificActorMessage
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


const FIRE_RATE: f32 = 0.11;
const MAX_TEMPERTURE: f32 = 60.0;
const MAX_SHOOTING_SPREAD: f32 = 0.0023;
const SHOOTING_SPREAD_INCR_SPEED: f32 = 15.0;
const SHOOTING_SPREAD_DCR_SPEED: f32 = 15.0;
const CROSSHAIR_INCREASE_ON_SHOOT: f32 = 0.2;

pub struct MachineGun {
    temperature: f32,
    shooting_spread: f32,
    time_from_prev_shot: f32,
    is_overheating: bool,

    shooted_from_pivot_point_dir: Vec4,

    machinegun_damage: f32,
    machinegun_add_force: f32, 
    machinegun_heat_add_on_shot: f32, 
    machinegun_cooling_speed: f32
}

impl MachineGun {
    pub fn new(
        machinegun_damage: f32,
        machinegun_add_force: f32, 
        machinegun_heat_add_on_shot: f32, 
        machinegun_cooling_speed: f32,
        shooted_from_pivot_point_dir: Vec4,

    ) -> Self {

        MachineGun {
            temperature: 0.0,
            shooting_spread: 0.0,
            time_from_prev_shot: 0.0,
            is_overheating: false,
            shooted_from_pivot_point_dir,

            machinegun_damage,
            machinegun_add_force, 
            machinegun_heat_add_on_shot, 
            machinegun_cooling_speed
        }
    }

    fn cool_machinegun(&mut self, delta: f32) {
        if self.temperature > delta * self.machinegun_cooling_speed {
            self.temperature -= delta * self.machinegun_cooling_speed;
        } else {
            self.temperature = 0.0;
        }
    }

    fn shoot(
        &mut self,
        player_id: ActorID,
        player: &mut PlayerInnerState,
        screen_effects: &mut PlayerScreenEffects,
        physic_system: &PhysicsSystem,
        audio_system: &mut AudioSystem,
        engine_handle: &mut EngineHandle,
    ) {
        audio_system.spawn_non_spatial_sound(
            crate::engine::audio::Sound::MachinegunShot,
            0.33,
            1.1,
            false,
            true,
            fyrox_sound::source::Status::Playing,
        );

        let from = player.get_eyes_position();

        let mut bytes = [0_u8;4];
        getrandom::getrandom(&mut bytes).expect("Func getrandom is fail in mahinegun shoot fn");
        let mut rnd_x = f32::from_be_bytes(bytes);
        if rnd_x.is_normal() {
            rnd_x = rnd_x.sin();
        } else {
            rnd_x = 0.0;
        }

        let mut bytes = [0_u8;4];
        getrandom::getrandom(&mut bytes).expect("Func getrandom is fail in mahinegun shoot fn");
        let mut rnd_y = f32::from_be_bytes(bytes);
        if rnd_y.is_normal() {
            rnd_y = rnd_y.sin();
        } else {
            rnd_y = 0.0;
        }

                
        let random_dir_y = Mat4::from_rotation_y((rnd_y - 0.5) as f32 * (self.shooting_spread));
        let random_dir_x = Mat4::from_rotation_x((rnd_x - 0.5) as f32 * (self.shooting_spread));
        
        let forward_dir = random_dir_x * random_dir_y * FORWARD;
        
        let direction = player.transform.get_rotation() * forward_dir;
        // direction = random_dir_x * direction;

        let weapon_offset = {
            player.get_eyes_offset() +
            (player.transform.get_rotation() *
            (self.shooted_from_pivot_point_dir.normalize() * player.collider.get_collider_radius()))
        };

        let hit = physic_system.ray_cast(from, direction, 700.0, Some(player_id));

        if let Some(hit) = hit {

            let position = hit.hit_point;
            let shooted_from = player.transform.get_position() + weapon_offset;


            if let Some(hited_id) = hit.hited_actors_id {

                if let Some(hited_actors_team) = hit.hited_actors_team
                {
                    if hited_actors_team != player.team
                    {                       
                        let force = hit.hit_normal * -self.machinegun_add_force;
            
                        engine_handle.send_direct_message(
                            hited_id,
                            Message {
                                from: player_id,
                                remote_sender: false,
                                message: MessageType::SpecificActorMessage(
                                    SpecificActorMessage::PlayerMessage(
                                        PlayerMessage::GetDamageAndForce(
                                            self.machinegun_damage as u32,
                                            force,
                                            position,
                                            player.team,
                                            player_id,
                                        )
                                    )
                                )
                            }
                        );
                    }
                }
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
                            NetMessageToPlayer::RemoteDirectMessage(
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
            let position_local = from + (direction * 17.0);
            let position_remote = from + (direction * 1500.0);
            let shooted_from = player.transform.get_position() + weapon_offset;

            let shot = MachinegunShot::new(
                position_local,
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
                            NetMessageToPlayer::RemoteDirectMessage(
                                player_id,
                                RemoteMessage::SpawnMachineGunShot(
                                    position_remote.to_array(),
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
    ) {
        if input.first_mouse.is_action_pressed() && self.temperature < MAX_TEMPERTURE {
            if self.time_from_prev_shot >= FIRE_RATE {
                self.shoot(
                    player_id,
                    player,
                    screen_effects,
                    physic_system,
                    audio_system,
                    engine_handle,
                );

                player.crosshair_target_size += CROSSHAIR_INCREASE_ON_SHOOT;

                self.temperature += self.machinegun_heat_add_on_shot;
                if self.shooting_spread < MAX_SHOOTING_SPREAD {
                    self.shooting_spread += MAX_SHOOTING_SPREAD * delta * SHOOTING_SPREAD_INCR_SPEED * 1.0/FIRE_RATE;
                }
                self.time_from_prev_shot = 0.0;
            } else {
                self.cool_machinegun(delta);
                self.time_from_prev_shot += delta;
            }
        } else {
            self.cool_machinegun(delta);
            self.time_from_prev_shot += delta;
            if self.shooting_spread > MAX_SHOOTING_SPREAD * delta * SHOOTING_SPREAD_DCR_SPEED {
                self.shooting_spread -= MAX_SHOOTING_SPREAD * delta * SHOOTING_SPREAD_DCR_SPEED;
            } else {
                self.shooting_spread = 0.0;
            }
        }

        let bar = match player.team {
            Team::Red => ui_system.get_mut_ui_element(&UIElementType::MachinegunBarRed),
            Team::Blue => ui_system.get_mut_ui_element(&UIElementType::MachinegunBarBlue),
        };

        if let UIElement::ProgressBar(bar) = bar {
            let value = {
                (self.temperature / MAX_TEMPERTURE)
                    .clamp(0.0, 1.0)
            };
            
            bar.set_bar_value(value)
        }
    }

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
        ) {
            self.cool_machinegun(delta);
            self.time_from_prev_shot += delta;

            let bar = match player.team {
                Team::Red => ui_system.get_mut_ui_element(&UIElementType::MachinegunBarRed),
                Team::Blue => ui_system.get_mut_ui_element(&UIElementType::MachinegunBarBlue),
            };

            if let UIElement::ProgressBar(bar) = bar {
                let value = {
                    (self.temperature / MAX_TEMPERTURE)
                        .clamp(0.0, 1.0)
                };
                
                bar.set_bar_value(value)
            }
    }

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

            let bar = ui_system.get_mut_ui_element(&UIElementType::MachinegunBarRed);

            if let UIElement::ProgressBar(bar) = bar {
                bar.ui_data.is_visible = false;
            }
    
            let bar = ui_system.get_mut_ui_element(&UIElementType::MachinegunBarBlue);
    
            if let UIElement::ProgressBar(bar) = bar {
                bar.ui_data.is_visible = false;
            }
    
            let img = ui_system.get_mut_ui_element(&UIElementType::MachinegunImage);
    
            if let UIElement::Image(img) = img {
                img.ui_data.is_visible = false;
            }
    }

    fn activate(
            &mut self,
            player_id: ActorID,
            player: &mut PlayerInnerState,
            physic_system: &PhysicsSystem,
            audio_system: &mut AudioSystem,
            ui_system: &mut UISystem,
            engine_handle: &mut EngineHandle,
        ) {
            let img = ui_system.get_mut_ui_element(&UIElementType::MachinegunImage);

            if let UIElement::Image(img) = img {
                img.ui_data.is_visible = true;
            }
    
            match player.team
            {
                Team::Red =>
                {
                    let bar = ui_system.get_mut_ui_element(&UIElementType::MachinegunBarRed);
    
                    if let UIElement::ProgressBar(bar) = bar {
                        bar.ui_data.is_visible = true;
                    }
    
                    let bar = ui_system.get_mut_ui_element(&UIElementType::MachinegunBarBlue);
    
                    if let UIElement::ProgressBar(bar) = bar {
                        bar.ui_data.is_visible = false;
                    }
                }
    
                Team::Blue =>
                {
                    let bar = ui_system.get_mut_ui_element(&UIElementType::MachinegunBarBlue);
    
                    if let UIElement::ProgressBar(bar) = bar {
                        bar.ui_data.is_visible = true;
                    }
    
                    let bar = ui_system.get_mut_ui_element(&UIElementType::MachinegunBarRed);
    
                    if let UIElement::ProgressBar(bar) = bar {
                        bar.ui_data.is_visible = false;
                    }
                }
            }
    }
}