pub mod player_input_master;
pub mod player_settings;

use crate::{
    actor::{
        device::{
            holegun::HoleGun, Device, DeviceType
        },
        Actor,
        ActorID,
        CommonActorsMessages,
        Component,
        Message,
        MessageType,
        SpecificActorMessage
    }, engine::{
        audio::{AudioSystem, Sound}, engine_handle::{
            Command,
            CommandType,
            EngineHandle
        }, net::{
            NetCommand,
            NetMessage,
            RemoteMessage
        }, physics::{
            colliders_container::PhysicalElement,
            kinematic_collider::KinematicCollider,
            PhysicsSystem
        }, render::VisualElement
    },
    transform::Transform
};

use self::{
    player_input_master::InputMaster,
    player_settings::PlayerSettings,
};

use std::f32::consts::PI;
use fyrox_core::pool::Handle;
use fyrox_sound::source::SoundSource;
use glam::{FloatExt, Mat4, Vec4};
use matchbox_socket::PeerId;

use super::{device::machinegun::MachineGun, players_death_explosion::PlayersDeathExplosion};


pub struct PlayerInnerState {
    pub collider: KinematicCollider,
    pub transform: Transform,
    pub hp: i32,
    pub is_alive: bool,
    pub is_enable: bool
    // pub weapon_offset: Vec4,
}


impl PlayerInnerState {
    pub fn new(transform: Transform, settings: &PlayerSettings, is_alive: bool, is_enable: bool) -> Self {

        PlayerInnerState {
            collider: KinematicCollider::new(
                settings.max_speed,
                settings.max_accel,
                settings.collider_radius,
                settings.friction_on_air,
                // settings.friction_on_ground,
            ),
            transform,
            hp: 0,
            is_alive,
            is_enable,
        }
    }
}


#[derive(PartialEq)]
enum ActiveHandsSlot {
    Zero,
    First,
    Second,
    Third,
}


pub enum PlayersDeviceSlotNumber {
    First,
    Second,
    Third,
    Fourth,
}

pub struct PlayerScreenEffects {
    pub w_scaner_is_active: bool,
    pub w_scaner_radius: f32,
    pub w_scaner_intesity: f32,

    pub death_screen_effect: f32,
    pub getting_damage_screen_effect: f32,
}


pub struct Player {
    id: Option<ActorID>,

    inner_state: PlayerInnerState,

    view_angle: Vec4,

    active_hands_slot: ActiveHandsSlot, 

    hands_slot_0: Box<dyn Device>,
    hands_slot_1: Option<Box<dyn Device>>,
    hands_slot_2: Option<Box<dyn Device>>,
    hands_slot_3: Option<Box<dyn Device>>,

    devices: [Option<Box<dyn Device>>; 4],

    is_gravity_y_enabled: bool,
    is_gravity_w_enabled: bool,

    player_settings: PlayerSettings,

    no_collider_veclocity: Vec4,

    explore_w_position: f32,
    explore_w_coefficient: f32,

    pub master: InputMaster,

    screen_effects: PlayerScreenEffects,

    w_scanner_enable: bool,
    w_scanner_radius: f32,
    w_scanner_reloading_time: f32,

    after_death_timer: f32,
    need_to_die_slowly: bool,

    rotating_around_w_sound_handle: Handle<SoundSource>,
    rotating_around_w_sound_pitch: f64,
    rotating_around_w_sound_gain: f32,
}

pub const PLAYER_MAX_HP: i32 = 100;

const MIN_TIME_BEFORE_RESPAWN: f32 = 1.5;
const MAX_TIME_BEFORE_RESPAWN: f32 = 5.0;

const W_SCANNER_RELOAD_TIME: f32 = 0.5;
const W_SCANNER_MAX_RADIUS: f32 = 22.0;
const W_SCANNER_EXPANDING_SPEED: f32 = 7.5;

pub const TIME_TO_DIE_SLOWLY: f32 = 0.5;


pub enum PlayerMessages {
    DealDamageAndAddForce(u32, Vec4, Vec4),
    NewPeerConnected(PeerId),
    Telefrag,
    DieImmediately,
    DieSlowly,
}


impl Actor for Player {
    fn recieve_message(
        &mut self,
        message: &Message,
        engine_handle: &mut EngineHandle,
        physic_system: &PhysicsSystem,
        audio_system: &mut AudioSystem
    ) {
        let from = message.from;

        let message = &message.message;
        
        match message {
            MessageType::CommonActorsMessages(message) => {
                match message {
                    CommonActorsMessages::SetTransform(transform) => {
                        self.inner_state.transform = transform.clone();
                    },
                    CommonActorsMessages::Enable(switch) => {
                        self.inner_state.is_enable = *switch;
                    },
                    CommonActorsMessages::IncrementPosition(increment) => {
                        self.inner_state.transform.increment_position(increment.clone());
                    },
                    CommonActorsMessages::IWasChangedMyId(new_id) => {}
                }
            }
            MessageType::PhysicsMessages(message) => {
                match message {
                    _ => {}
                }
            },
            MessageType::SpecificActorMessage(message) => {
                match message {
                    SpecificActorMessage::PLayerMessages(message) => {
                        match message {
                            PlayerMessages::Telefrag => {
                                self.die(true, engine_handle, physic_system, audio_system);
                            }

                            PlayerMessages::DieImmediately => {
                                self.die(true, engine_handle, physic_system, audio_system);
                            }

                            PlayerMessages::DieSlowly => {
                                self.die(false, engine_handle, physic_system, audio_system);
                            }

                            PlayerMessages::DealDamageAndAddForce(damage, force, _) => {
                                self.get_damage_and_add_force(*damage as i32, *force, engine_handle, physic_system, audio_system);
                            }

                            PlayerMessages::NewPeerConnected(peer_id) => {

                                engine_handle.send_command(
                                    Command {
                                        sender: self.id.unwrap(),
                                        command_type: CommandType::NetCommand(
                                            NetCommand::SendDirectNetMessageReliable(
                                                NetMessage::RemoteCommand(
                                                    crate::engine::net::RemoteCommand::SpawnPlayersDollActor(
                                                        self.get_transform().to_serializable_transform(),
                                                        self.inner_state.collider.get_collider_radius(),
                                                        self.inner_state.is_alive
                                                    )
                                                ),
                                                peer_id.clone(),
                                            )
                                        )
                                    }
                                )
                            }
                        }
                    },
                    _ => {},
                }

            }  
        }
    }


    fn get_mut_transform(&mut self) -> &mut Transform {
        &mut self.inner_state.transform
    }


    fn get_transform(&self) -> &Transform {
        &self.inner_state.transform
    }


    fn init(&mut self, id: ActorID) {
        self.id = Some(id);

        self.inner_state.collider.init(id);
    }


    fn get_id(&self) -> Option<ActorID> {
        self.id
    }

    fn set_id(&mut self, id: ActorID, engine_handle: &mut EngineHandle) {
        
        if let Some(prev_id) = self.id {
            engine_handle.send_boardcast_message(Message {
                from: prev_id,
                message: MessageType::CommonActorsMessages(
                    CommonActorsMessages::IWasChangedMyId(
                        id
                    )
                )
            });
        }

        self.id = Some(id);
    }

    fn get_physical_element(&mut self) -> Option<PhysicalElement> {
        if self.inner_state.is_enable {
            let collider_container = PhysicalElement {
                transform: &mut self.inner_state.transform,
                kinematic_collider: Some(&mut self.inner_state.collider),
                static_colliders: None,
                dynamic_colliders: None,
                static_objects: None,
                area: None,
            };
    
            return Some(collider_container);
        }
            None
    }

    fn get_visual_element(&self) -> Option<VisualElement> {
        if self.inner_state.is_enable {
            match self.active_hands_slot {
                ActiveHandsSlot::Zero => {
                    return self.hands_slot_0.get_visual_element(self.get_transform());
                },
                ActiveHandsSlot::First => {
                    if let Some(device) = &self.hands_slot_1 {
                        return device.get_visual_element(self.get_transform());
                    } else {
                        return None;
                    }
                },
                ActiveHandsSlot::Second => {
                    if let Some(device) = &self.hands_slot_2 {
                        return device.get_visual_element(self.get_transform());
                    } else {
                        return None;
                    }
                },
                ActiveHandsSlot::Third => {
                    if let Some(device) = &self.hands_slot_3 {
                        return device.get_visual_element(self.get_transform());
                    } else {
                        return None;
                    }
                }
            }
        }
        None
    }

    fn tick(
        &mut self,
        physic_system: &PhysicsSystem,
        engine_handle: &mut EngineHandle,
        audio_system: &mut AudioSystem,
        delta: f32
    ) {
        let my_id = self.id.expect("Player does not have id");

        let input = match &self.master {
            InputMaster::LocalMaster(master) => {
                master.current_input.clone()
            }
            InputMaster::RemoteMaster(master) => {
               master.current_input.clone()
            }   
        };

        if self.inner_state.is_alive {

            let mut x = self.view_angle.x;
            let mut y = self.view_angle.y;
            let mut xw = self.view_angle.z;
            let mut yw = self.view_angle.w;

            let prev_yw = yw;
    
            if input.second_mouse.is_action_pressed() {
                xw = input.mouse_axis.x + xw;
                yw = (input.mouse_axis.y + yw).clamp(-PI/2.0, PI/2.0);
                
            } else {
                xw *= 1.0 - delta * 3.0;
                yw *= 1.0 - delta * 3.0;
    
                x = input.mouse_axis.x + x;
                y = (input.mouse_axis.y + y).clamp(-PI/2.0, PI/2.0);
            }
    
    
            let normal_rotation = Mat4::from_cols_slice(&[
                x.cos(),    y.sin() * x.sin(),  y.cos() * x.sin(),  0.0,
                0.0,        y.cos(),            -y.sin(),           0.0,
                -x.sin(),   y.sin() * x.cos(),  y.cos()*x.cos(),    0.0,
                0.0,        0.0,                0.0,                1.0
            ]);
    
            // let xw_rotation = Mat4::from_cols_slice(&[
            //     yw.cos(),    0.0,    0.0,    yw.sin(),
            //     0.0,        1.0,    0.0,    0.0,
            //     0.0,        0.0,    1.0,    0.0,
            //     -yw.sin(),   0.0,    0.0,    yw.cos()
            // ]);
    
            let yw_rotation = Mat4::from_cols_slice(&[
                1.0,    0.0,    0.0,        0.0,
                0.0,    1.0,    0.0,        0.0,
                0.0,    0.0,    yw.cos(),   yw.sin(),
                0.0,    0.0,    -yw.sin(),   yw.cos()
            ]);
    
    
            self.set_rotation_matrix(yw_rotation * normal_rotation);
    
            // self.set_rotation_matrix(Mat4::from_cols_slice(&[
            //     y.cos(),    0.0,    0.0,    y.sin(),
            //     0.0,        1.0,    0.0,    0.0,
            //     0.0,        0.0,    1.0,    0.0,
            //     -y.sin(),   0.0,    0.0,    y.cos()
            // ]));
    
            // self.set_rotation_matrix(Mat4::from_cols_slice(&[
            //     1.0,    0.0,        0a0,    0.0,
            //     0.0,    y.cos(),    0.0,    y.sin(),
            //     0.0,    0.0,        1.0,    0.0,
            //     0.0,    -y.sin(),   0.0,    y.cos()
            // ]));
    
            let xz_player_rotation = Mat4::from_rotation_y(x);
            self.view_angle = Vec4::new(x, y, xw, yw);

            let base_pitch = {
                0.8.lerp(
                    1.5,
                    (std::f64::consts::PI/2.0 + yw as f64) / std::f64::consts::PI
                )
            };

            let addition_pitch = {
                self.rotating_around_w_sound_pitch * (1.0-delta as f64*22.0) +
                ((prev_yw - yw) as f64).abs() * 2.0
            };

            self.rotating_around_w_sound_pitch = addition_pitch;

            let gain = {
                self.rotating_around_w_sound_gain * (1.0-(delta*42.0)) +
                (prev_yw - yw).abs() * 10.0
            };

            self.rotating_around_w_sound_gain = gain;

            audio_system.sound_set_pitch_and_gain(
                self.rotating_around_w_sound_handle,
                base_pitch,//D + addition_pitch,
                gain
            );
    
            // self.inner_state.collision.transform.rotation *= new_rotation_matrix;
    
            match self.active_hands_slot {
                ActiveHandsSlot::Zero => {
                    self.hands_slot_0.process_input(my_id, &mut self.inner_state, &input, physic_system, audio_system, engine_handle, delta);

                    if let Some(device) = &mut self.hands_slot_1 {
                        device.process_while_deactive(my_id, &mut self.inner_state, &input, physic_system, audio_system, engine_handle, delta);
                    }
                    if let Some(device) = &mut self.hands_slot_2 {
                        device.process_while_deactive(my_id, &mut self.inner_state, &input, physic_system, audio_system, engine_handle, delta);
                    }
                    if let Some(device) = &mut self.hands_slot_3 {
                        device.process_while_deactive(my_id, &mut self.inner_state, &input, physic_system, audio_system, engine_handle, delta);
                    }
                },
                ActiveHandsSlot::First => {
                    if let Some(device) = self.hands_slot_1.as_mut() {
                        device.process_input(my_id, &mut self.inner_state, &input, physic_system, audio_system, engine_handle, delta);
                    }

                    self.hands_slot_0.process_while_deactive(my_id, &mut self.inner_state, &input, physic_system, audio_system, engine_handle, delta);
                    if let Some(device) = &mut self.hands_slot_2 {
                        device.process_while_deactive(my_id, &mut self.inner_state, &input, physic_system, audio_system, engine_handle, delta);
                    }
                    if let Some(device) = &mut self.hands_slot_3 {
                        device.process_while_deactive(my_id, &mut self.inner_state, &input, physic_system, audio_system, engine_handle, delta);
                    }
                },
                ActiveHandsSlot::Second => {
                    if let Some(device) = self.hands_slot_2.as_mut() {
                        device.process_input(my_id, &mut self.inner_state, &input, physic_system, audio_system, engine_handle, delta);
                    }

                    self.hands_slot_0.process_while_deactive(my_id, &mut self.inner_state, &input, physic_system, audio_system, engine_handle, delta);
                    if let Some(device) = &mut self.hands_slot_1 {
                        device.process_while_deactive(my_id, &mut self.inner_state, &input, physic_system, audio_system, engine_handle, delta);
                    }
                    if let Some(device) = &mut self.hands_slot_3 {
                        device.process_while_deactive(my_id, &mut self.inner_state, &input, physic_system, audio_system, engine_handle, delta);
                    }
                },
                ActiveHandsSlot::Third => {
                    if let Some(device) = self.hands_slot_3.as_mut() {
                        device.process_input(my_id, &mut self.inner_state, &input, physic_system, audio_system, engine_handle, delta);
                    }

                    self.hands_slot_0.process_while_deactive(my_id, &mut self.inner_state, &input, physic_system, audio_system, engine_handle, delta);
                    if let Some(device) = &mut self.hands_slot_1 {
                        device.process_while_deactive(my_id, &mut self.inner_state, &input, physic_system, audio_system, engine_handle, delta);
                    }
                    if let Some(device) = &mut self.hands_slot_2 {
                        device.process_while_deactive(my_id, &mut self.inner_state, &input, physic_system, audio_system, engine_handle, delta);
                    }
                }
            }
    
            for device in self.devices.iter_mut() {
                if let Some(device) = device {
                    device.process_input(my_id, &mut self.inner_state, &input, physic_system, audio_system, engine_handle, delta);
                }
            }
    
            if input.mode_1.is_action_just_pressed() {
                self.is_gravity_y_enabled = !self.is_gravity_y_enabled;
            }
    
            if input.mode_2.is_action_just_pressed() {
                self.is_gravity_w_enabled = !self.is_gravity_w_enabled;
            }
    
            if input.mode_3.is_action_just_pressed() {
                self.inner_state.collider.is_enable = !self.inner_state.collider.is_enable;
            }
    
            if input.activate_hand_slot_0.is_action_just_pressed() {
                self.deavctivate_previous_device(
                    ActiveHandsSlot::Zero,
                    physic_system,
                    audio_system,
                    engine_handle,
                );
                self.active_hands_slot = ActiveHandsSlot::Zero;
            }
    
            if input.activate_hand_slot_1.is_action_just_pressed() {
                if self.hands_slot_1.is_some() {
                    self.deavctivate_previous_device(
                        ActiveHandsSlot::First,
                        physic_system,
                        audio_system,
                        engine_handle,
                    );
                    self.active_hands_slot = ActiveHandsSlot::First;
                }
            }
    
            if input.activate_hand_slot_2.is_action_just_pressed() {
                if self.hands_slot_2.is_some() {
                    self.deavctivate_previous_device(
                        ActiveHandsSlot::Second,
                        physic_system,
                        audio_system,
                        engine_handle,
                    );
                    self.active_hands_slot = ActiveHandsSlot::Second;
                }
            }
    
            if input.activate_hand_slot_3.is_action_just_pressed() {
                if self.hands_slot_3.is_some() {
                    self.deavctivate_previous_device(
                        ActiveHandsSlot::Third,
                        physic_system,
                        audio_system,
                        engine_handle,
                    );
                    self.active_hands_slot = ActiveHandsSlot::Third;
                }
            }
    
            let mut movement_vec = Vec4::ZERO;
    
            if input.move_forward.is_action_pressed() {
                movement_vec += Vec4::NEG_Z;
            }
    
            if input.move_backward.is_action_pressed() {
                movement_vec += Vec4::Z;
            }
    
            if input.move_right.is_action_pressed() {
                movement_vec += Vec4::X;
            }
    
            if input.move_left.is_action_pressed() {
                movement_vec += Vec4::NEG_X;
            }
    
            if let Some(vec) = movement_vec.try_normalize() {
                movement_vec = vec;
            }
    
            if input.jump.is_action_just_pressed() {
    
                if self.inner_state.collider.is_on_ground {
                    self.inner_state.collider.add_force(Vec4::Y * self.player_settings.jump_y_speed);
    
                }
            }
    
            if input.w_up.is_action_pressed() {
                if self.inner_state.collider.is_enable {
                    self.inner_state.collider.add_force(Vec4::W * self.player_settings.jetpak_w_speed);
                } else {
                    self.no_collider_veclocity += Vec4::W * self.player_settings.jetpak_w_speed;
                }
            }
    
            if input.w_down.is_action_pressed() {
                if self.inner_state.collider.is_enable {
                    self.inner_state.collider.add_force(Vec4::NEG_W * self.player_settings.jetpak_w_speed);
                } else {
                    self.no_collider_veclocity += Vec4::NEG_W * self.player_settings.jetpak_w_speed;
                }
            }
    
            if input.w_scaner.is_action_just_pressed() {
                if !self.w_scanner_enable {
                    if self.w_scanner_reloading_time >= W_SCANNER_RELOAD_TIME {
                        self.w_scanner_enable = true;
    
                        self.w_scanner_radius = self.inner_state.collider.get_collider_radius() + 0.1;
                    }
                }
            }
    
            if self.w_scanner_enable {
                self.w_scanner_radius += delta * W_SCANNER_EXPANDING_SPEED;
    
                if self.w_scanner_radius >= W_SCANNER_MAX_RADIUS {
                    self.w_scanner_enable = false;
                    self.w_scanner_reloading_time = 0.0;
                }
            }
    
            if !self.w_scanner_enable {
    
                if self.w_scanner_reloading_time < W_SCANNER_RELOAD_TIME {
                    self.w_scanner_reloading_time += delta;
                }
            }
    
            if input.jump_w.is_action_just_pressed() {
                self.inner_state.collider.add_force(Vec4::W * self.player_settings.jump_w_speed);
                // self.inner_state.collider.add_force(Vec4::Y * self.player_settings.jump_y_speed);
            };
    
            if self.inner_state.collider.is_enable {
    
                if self.is_gravity_y_enabled {
                    movement_vec = self.get_rotation_matrix().inverse() * movement_vec;
    
                    if self.inner_state.collider.is_on_ground {
                        self.inner_state.collider.set_wish_direction(
                            movement_vec,
                            1.0
                        );
                    } else {
                        self.inner_state.collider.set_wish_direction(
                            movement_vec,
                            self.player_settings.air_speed_mult
                        );
                    }
    
                    self.inner_state.collider.add_force(Vec4::NEG_Y * self.player_settings.gravity_y_speed);
    
                } else {
                   movement_vec = self.get_rotation_matrix().inverse() * movement_vec;
    
                   self.inner_state.collider.set_wish_direction(movement_vec, 1.0);
    
                }
    
                if self.is_gravity_w_enabled {
                    self.inner_state.collider.add_force(Vec4::NEG_W * self.player_settings.gravity_w_speed);
                }
    
            } else {
                
                movement_vec = self.get_rotation_matrix().inverse() * movement_vec;
    
                const MAX_SPEED: f32 = 24.0;
                const MAX_ACCEL: f32 = 32.0;
    
                if movement_vec.length().is_normal() {
                    let current_speed_in_wishdir = self.no_collider_veclocity.dot(movement_vec);
        
                    let speed = MAX_SPEED - current_speed_in_wishdir;
        
                    let add_speed = 0.0_f32.max(speed.min(MAX_ACCEL * delta));
        
                    self.no_collider_veclocity += movement_vec * add_speed;
        
                }
    
                self.inner_state.transform.increment_position(self.no_collider_veclocity * delta);
            }
    
            self.no_collider_veclocity *= 1.0 - delta*3.4;
    
            log::info!("Position: {:.2}", self.get_position());
    
            self.screen_effects.w_scaner_is_active = self.w_scanner_enable;
            self.screen_effects.w_scaner_radius = self.w_scanner_radius;
            self.screen_effects.w_scaner_intesity = {
                let mut intensity = W_SCANNER_MAX_RADIUS - self.w_scanner_radius;
    
                intensity /= W_SCANNER_MAX_RADIUS/3.0;
    
                intensity.clamp(0.0, 1.0)
            };
        } else {
            //while player is not alive

            self.after_death_timer += delta;

            match self.active_hands_slot {
                ActiveHandsSlot::Zero => {
                    self.hands_slot_0.process_while_player_is_not_alive(my_id, &mut self.inner_state, &input, physic_system, audio_system, engine_handle, delta);

                },
                ActiveHandsSlot::First => {
                    if let Some(device) = self.hands_slot_1.as_mut() {
                        device.process_while_player_is_not_alive(my_id, &mut self.inner_state, &input, physic_system, audio_system, engine_handle, delta);
                    }

                },
                ActiveHandsSlot::Second => {
                    if let Some(device) = self.hands_slot_2.as_mut() {
                        device.process_while_player_is_not_alive(my_id, &mut self.inner_state, &input, physic_system, audio_system, engine_handle, delta);
                    }

                },
                ActiveHandsSlot::Third => {
                    if let Some(device) = self.hands_slot_3.as_mut() {
                        device.process_while_player_is_not_alive(my_id, &mut self.inner_state, &input, physic_system, audio_system, engine_handle, delta);
                    }

                }
            }
    
            for device in self.devices.iter_mut() {
                if let Some(device) = device {
                    device.process_while_player_is_not_alive(my_id, &mut self.inner_state, &input, physic_system, audio_system, engine_handle, delta);
                }
            }

            if self.need_to_die_slowly {
                if self.after_death_timer >= TIME_TO_DIE_SLOWLY {
                    self.need_to_die_slowly = false;
                    self.inner_state.is_enable = false;
                    
                    self.play_die_effects(engine_handle);
                }
            }

            if self.is_gravity_w_enabled {
                self.inner_state.collider.add_force(Vec4::NEG_W * self.player_settings.gravity_w_speed);
            }

            if self.is_gravity_y_enabled {
                self.inner_state.collider.add_force(Vec4::NEG_Y * self.player_settings.gravity_y_speed);
            }

            if self.after_death_timer >= MAX_TIME_BEFORE_RESPAWN {
                engine_handle.send_command(
                    Command {
                        sender: self.get_id().expect("Player have not ActorID"),
                        command_type: CommandType::RespawnPlayer(
                            self.get_id().expect("Player have not ActorID")
                        )
                    }
                );
                return;
            }

            if input.first_mouse.is_action_just_pressed() {
                if self.after_death_timer >= MIN_TIME_BEFORE_RESPAWN {
                    engine_handle.send_command(
                        Command {
                            sender: self.get_id().expect("Player have not ActorID"),
                            command_type: CommandType::RespawnPlayer(
                                self.get_id().expect("Player have not ActorID")
                            )
                        }
                    );
                    return;
                }
            }

            self.screen_effects.w_scaner_is_active = self.w_scanner_enable;
            self.screen_effects.w_scaner_radius = self.w_scanner_radius;
            self.screen_effects.w_scaner_intesity = {
                let mut intensity = W_SCANNER_MAX_RADIUS - self.w_scanner_radius;
    
                intensity /= W_SCANNER_MAX_RADIUS/3.0;
    
                intensity = intensity.clamp(0.0, 1.0);

                intensity -= self.after_death_timer * 2.0;
                    
                intensity.clamp(0.0, 1.0)
            };
        }

        audio_system.set_listener_position_and_look_vector(
            self.get_position(),
            self.get_transform().get_direction_for_audio_system()
        );

        engine_handle.send_command(Command{
            sender: my_id,
            command_type: CommandType::NetCommand(
                NetCommand::SendBoardcastNetMessageUnreliable(
                    NetMessage::RemoteDirectMessage(
                        my_id,
                        RemoteMessage::SetTransform(
                            self.inner_state.transform.to_serializable_transform()
                        )
                    )
                )
            )
        });

    }
}



impl Player {

    pub fn new(master: InputMaster, player_settings: PlayerSettings, audio_system: &mut AudioSystem) -> Self {
        
        let screen_effects = PlayerScreenEffects {
            w_scaner_is_active: false,
            w_scaner_radius: 0.0,
            w_scaner_intesity: 0.0,
            death_screen_effect: 0.0,
            getting_damage_screen_effect: 0.0,
        };

        let rotating_around_w_sound_handle = audio_system.spawn_non_spatial_sound(
            Sound::RotatingAroundW,
            0.0,
            1.0,
            true,
            false,
            fyrox_sound::source::Status::Playing
        );
        
        Player {
            id: None,

            inner_state: PlayerInnerState::new(Transform::new(), &player_settings, false, false),
            active_hands_slot: ActiveHandsSlot::Zero,

            hands_slot_0: Box::new(HoleGun::new()),
            hands_slot_1: Some(Box::new(MachineGun::new())),
            hands_slot_2: None,
            hands_slot_3: None,

            is_gravity_y_enabled: true,
            is_gravity_w_enabled: true,

            devices: [None, None, None, None],
            
            player_settings,

            master,

            explore_w_position: 0.0,
            explore_w_coefficient: 0.0,

            no_collider_veclocity: Vec4::ZERO,

            view_angle: Vec4::ZERO,

            screen_effects,

            w_scanner_enable: false,
            w_scanner_radius: 0.0,
            w_scanner_reloading_time: W_SCANNER_RELOAD_TIME,

            after_death_timer: MIN_TIME_BEFORE_RESPAWN,
            need_to_die_slowly: false,

            rotating_around_w_sound_handle,
            rotating_around_w_sound_pitch: 1.0,
            rotating_around_w_sound_gain: 0.0,
        }
    }

    pub fn get_explore_w_position(&self) -> f32 {
        self.explore_w_position
    }

    pub fn get_explore_w_coefficient(&self) -> f32 {
        self.explore_w_coefficient
    }

    pub fn get_position(&self) -> Vec4 {
        self.inner_state.transform.get_position()
    }


    pub fn get_rotation_matrix(&self) -> Mat4 {
        self.inner_state.transform.get_rotation()
    }

    pub fn get_player_visual_effects(&self) -> &PlayerScreenEffects {
        &self.screen_effects
    }


    pub fn set_rotation_matrix(&mut self, new_rotation: Mat4) {
        self.inner_state.transform.set_rotation(new_rotation)
    }


    pub fn get_collider_radius(&self) -> f32 {
        self.inner_state.collider.get_collider_radius()
    }


    fn get_damage_and_add_force(
        &mut self,
        damage: i32,
        force: Vec4,
        engine_handle: &mut EngineHandle,
        physic_system: &PhysicsSystem,
        audio_system: &mut AudioSystem,
    ) {

        self.inner_state.hp -= damage;
        self.inner_state.collider.add_force(force);

        if self.inner_state.hp <= 0 {
            if damage >= PLAYER_MAX_HP {
                self.die(true, engine_handle, physic_system, audio_system);
            } else {

                // self.die(false, engine_handle, physic_system, audio_system);
                
                // temproral solution
                self.die(true, engine_handle, physic_system, audio_system);
            }
        }
    }

    fn deavctivate_previous_device(&mut self,
        new_active_slot: ActiveHandsSlot,
        physic_system: &PhysicsSystem,
        audio_system: &mut AudioSystem,
        engine_handle: &mut EngineHandle,
    ) {
        let my_id = self.get_id().expect("Player have nit ActorID");

        match self.active_hands_slot {
            ActiveHandsSlot::Zero => {
                if new_active_slot != ActiveHandsSlot::Zero {
                    self
                        .hands_slot_0
                        .deactivate(
                            my_id,
                            &mut self.inner_state,
                            physic_system,
                            audio_system,
                            engine_handle
                        );
                }
            },
            ActiveHandsSlot::First => {
                if new_active_slot != ActiveHandsSlot::First {
                    self
                        .hands_slot_1.as_mut().expect("Player have not any device in active hand's slot")
                        .deactivate(
                            my_id,
                            &mut self.inner_state,
                            physic_system,
                            audio_system,
                            engine_handle
                        );
                }
            }
            ActiveHandsSlot::Second => {
                if new_active_slot != ActiveHandsSlot::Second {
                    self
                        .hands_slot_2.as_mut().expect("Player have not any device in active hand's slot")
                        .deactivate(
                            my_id,
                            &mut self.inner_state,
                            physic_system,
                            audio_system,
                            engine_handle
                        );
                }
            }
            ActiveHandsSlot::Third => {
                if new_active_slot != ActiveHandsSlot::Third {
                    self
                        .hands_slot_3.as_mut().expect("Player have not any device in active hand's slot")
                        .deactivate(
                            my_id,
                            &mut self.inner_state,
                            physic_system,
                            audio_system,
                            engine_handle
                        );
                }
            }
        }
    }

    fn telefrag(&mut self, engine_handle: &mut EngineHandle) {
        self.die_immediately(engine_handle);
    }

    fn die_immediately(
        &mut self,
        engine_handle: &mut EngineHandle,
    ) {
        if self.inner_state.is_alive {

            self.inner_state.is_alive = false;
            self.inner_state.is_enable = false;
            self.need_to_die_slowly = false;
            self.after_death_timer = 0.0;

            self.play_die_effects(engine_handle);

            engine_handle.send_command(
                Command {
                    sender: self.get_id().expect("Player have not ActorID"),
                    command_type: CommandType::NetCommand(
                        NetCommand::SendBoardcastNetMessageReliable(
                            NetMessage::RemoteDirectMessage(
                                self.get_id().expect("Player have not ActorID"),
                                RemoteMessage::DieImmediately
                            )
                        )
                    )
                }
            );
        }
    }

    fn play_die_effects(&mut self, engine_handle: &mut EngineHandle) {
        
        let players_death_explode = PlayersDeathExplosion::new(
            self.get_transform().get_position()
        );

        engine_handle.send_command(
            Command {
                sender: self.get_id().expect("Player have not ActorID"),
                command_type: CommandType::SpawnActor(
                    super::ActorWrapper::PlayersDeathExplosion(players_death_explode)
                )
            }
        );
    }


    fn die_slowly(&mut self, engine_handle: &mut EngineHandle) {
        if self.inner_state.is_alive {
        
            self.inner_state.is_alive = false;
            self.inner_state.is_enable = true;
            self.need_to_die_slowly = true;
            self.after_death_timer = 0.0;

            engine_handle.send_command(
                Command {
                    sender: self.get_id().expect("Player have not ActorID"),
                    command_type: CommandType::NetCommand(
                        NetCommand::SendBoardcastNetMessageReliable(
                            NetMessage::RemoteDirectMessage(
                                self.get_id().expect("Player have not ActorID"),
                                RemoteMessage::DieSlowly
                            )
                        )
                    )
                }
            );
        }
    }

    fn die(
        &mut self,
        die_immediately: bool,
        engine_handle: &mut EngineHandle,
        physic_system: &PhysicsSystem,
        audio_system: &mut AudioSystem,

    ) {
        let my_id = self.get_id().expect("Player have not ActorID");

        match self.active_hands_slot {
            ActiveHandsSlot::Zero => {
                self.hands_slot_0.deactivate(my_id, &mut self.inner_state, physic_system, audio_system, engine_handle);

            },
            ActiveHandsSlot::First => {
                if let Some(device) = self.hands_slot_1.as_mut() {
                    device.deactivate(my_id, &mut self.inner_state, physic_system, audio_system, engine_handle);
                }

            },
            ActiveHandsSlot::Second => {
                if let Some(device) = self.hands_slot_2.as_mut() {
                    device.deactivate(my_id, &mut self.inner_state, physic_system, audio_system, engine_handle);
                }

            },
            ActiveHandsSlot::Third => {
                if let Some(device) = self.hands_slot_3.as_mut() {
                    device.deactivate(my_id, &mut self.inner_state, physic_system, audio_system, engine_handle);
                }

            }
        }

        for device in self.devices.iter_mut() {
            if let Some(device) = device {
                device.deactivate(my_id, &mut self.inner_state, physic_system, audio_system, engine_handle);
            }
        }

        if die_immediately {
            self.die_immediately(engine_handle);
        } else {
            self.die_slowly(engine_handle);
        }
    }



    pub fn respawn(&mut self, spawn_position: Vec4, engine_handle: &mut EngineHandle, physics_system: &PhysicsSystem) {
        self.inner_state.is_alive = true;
        self.inner_state.is_enable = true;
        self.inner_state.hp = PLAYER_MAX_HP;

        self.inner_state.collider.reset_forces_and_velocity();

        self.inner_state.transform = Transform::from_position(spawn_position);

        let hits = physics_system.sphere_cast_on_dynamic_colliders(spawn_position, self.get_collider_radius());

        for hit in hits {
            engine_handle.send_direct_message(
                hit.hited_actors_id.expect("In respawn func in death on resapwn hit have not ActorID"),
                Message {
                    from: self.get_id().expect("Player have not ID in respawn func"),
                    message: MessageType::SpecificActorMessage(
                        SpecificActorMessage::PLayerMessages(
                            PlayerMessages::Telefrag
                        )
                    )
                }
            )
        }

        engine_handle.send_command(
            Command {
                sender: self.get_id().expect("Player have not ActorID"),
                command_type: CommandType::NetCommand(
                    NetCommand::SendBoardcastNetMessageReliable(
                        NetMessage::RemoteDirectMessage(
                            self.get_id().expect("Player have not ActorID"),
                            RemoteMessage::PlayerRespawn(spawn_position.to_array())
                        )
                    )
                )
            }
        )
    }


    fn set_gun_to_1_slot(
        &mut self,
        device: Box<dyn Device>
    ) -> Option<Box<dyn Device>>
    {

        match device.get_device_type() {
            DeviceType::Gun => {
                let prev_device = self.hands_slot_1.take();
                self.hands_slot_1 = Some(device);

                return prev_device;
            }
            _ => {
                Some(device)
            }
        }
    }


    fn set_gun_to_2_slot(
        &mut self,
        device: Box<dyn Device>
    ) -> Option<Box<dyn Device>>
    {

        match device.get_device_type() {
            DeviceType::Gun => {
                let prev_device = self.hands_slot_2.take();
                self.hands_slot_2 = Some(device);

                return prev_device;
            }
            _ => {
                Some(device)
            }
        }
    }


    fn set_gun_to_3_slot(
        &mut self,
        device: Box<dyn Device>
    ) -> Option<Box<dyn Device>>
    {

        match device.get_device_type() {
            DeviceType::Gun => {
                let prev_device = self.hands_slot_3.take();
                self.hands_slot_3 = Some(device);

                return prev_device;
            }
            _ => {
                Some(device)
            }
        }
    }


    fn set_device_to_device_slot(
        &mut self,
        slot_number: PlayersDeviceSlotNumber,
        device: Box<dyn Device>
    ) -> Option<Box<dyn Device>> {

        match device.get_device_type() {
            DeviceType::Device => {
                match slot_number {
                    PlayersDeviceSlotNumber::First => {
                        let prev_device = self.devices[0].take();
                        self.devices[0] = Some(device);
                        prev_device
                    }
                    PlayersDeviceSlotNumber::Second => {
                        let prev_device = self.devices[1].take();
                        self.devices[1] = Some(device);
                        prev_device
                    }
                    PlayersDeviceSlotNumber::Third => {
                        let prev_device = self.devices[2].take();
                        self.devices[2] = Some(device);
                        prev_device
                    }
                    PlayersDeviceSlotNumber::Fourth => {
                        let prev_device = self.devices[3].take();
                        self.devices[3] = Some(device);
                        prev_device
                    }
                }

                
            },
            _ => {Some(device)}
        }
    }
}