pub mod player_input_master;
pub mod player_settings;

use client_server_protocol::{
    NetCommand, NetMessageToPlayer, RemoteCommand, RemoteMessage, Team
};

use crate::{
    actor::{
        device::{
            holegun::HoleGun, Device, DeviceType
        }, players_doll::PlayerDollInputState, Actor, ActorID, CommonActorsMessages, Component, Message, MessageType, SpecificActorMessage
    }, engine::{
        audio::{
            AudioSystem,
            Sound
        }, engine_handle::{
            Command,
            CommandType,
            EngineHandle
        }, physics::{
            colliders_container::PhysicalElement,
            dynamic_collider::PlayersDollCollider,
            kinematic_collider::{
                KinematicCollider,
                KinematicColliderMessages
            },
            PhysicsSystem
        }, render::VisualElement, time::TimeSystem, ui::{
            RectSize,
            UIElement,
            UIElementType,
            UISystem,
        }, world::level::Spawn
    },
    transform::Transform
};

use self::{
    player_input_master::InputMaster,
    player_settings::PlayerSettings,
};

use core::panic;
use std::f32::consts::PI;
use fyrox_core::pool::Handle;
use fyrox_sound::source::SoundSource;
use glam::{FloatExt, Mat4, Vec2, Vec4};

use super::{
    device::machinegun::MachineGun, players_death_explosion::PlayersDeathExplosion, session_controller::DEFAULT_TEAM, PhysicsMessages
};


pub struct PlayerInnerState {
    pub team: Team,
    pub collider: KinematicCollider,
    pub collider_for_others: Vec<PlayersDollCollider>,
    pub transform: Transform,
    pub hp: i32,
    pub is_alive: bool,
    pub is_enable: bool,
    pub crosshair_target_size: f32,
    pub crosshair_size: f32,

    pub zw_rotation: Mat4,
    pub zy_rotation: Mat4,
    pub zx_rotation: Mat4,
    // pub weapon_offset: Vec4,
}


impl PlayerInnerState {
    pub fn new(transform: Transform, settings: &PlayerSettings, is_alive: bool, is_enable: bool) -> Self {

        let collider_for_others = {
            let mut vec = Vec::with_capacity(1);
            
            vec.push(PlayersDollCollider {
                position: Vec4::ZERO,
                radius: settings.collider_radius,
                friction: 0_f32,
                bounce_rate: 0_f32,
                actors_id: None,
                weapon_offset: Vec4::ZERO,
            });
            vec
        };

        PlayerInnerState {
            team: DEFAULT_TEAM,
            collider: KinematicCollider::new(
                settings.max_speed,
                settings.max_accel,
                settings.collider_radius,
                settings.friction_on_air,
                // settings.friction_on_ground,
            ),
            collider_for_others,
            transform,
            hp: 0,
            is_alive,
            is_enable,
            crosshair_target_size: 0.04,
            crosshair_size: 0.04,

            zw_rotation: Mat4::IDENTITY,
            zy_rotation: Mat4::IDENTITY,
            zx_rotation: Mat4::IDENTITY,
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
    pub w_scanner_is_active: bool,
    pub w_scanner_radius: f32,
    pub w_scanner_ring_intesity: f32,
    pub w_scanner_enemies_intesity: f32,

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

    pub player_settings: PlayerSettings,

    no_collider_veclocity: Vec4,

    explore_w_position: f32,
    explore_w_coefficient: f32,

    pub master: InputMaster,

    screen_effects: PlayerScreenEffects,

    w_scanner_enable: bool,
    w_scanner_radius: f32,
    w_scanner_reloading_time: f32,
    w_scanner_enemies_show_time: f32,

    after_death_timer: f32,
    need_to_die_slowly: bool,

    rotating_around_w_sound_handle: Handle<SoundSource>,
    rotating_around_w_sound_pitch: f64,
    rotating_around_w_sound_gain: f32,

    shifting_along_w_sound_handle: Handle<SoundSource>,
    shifting_along_w_sound_pitch: f64,
    shifting_along_w_sound_gain: f32,
    player_previous_w_position: f32,

    jumped_to_y_on_current_action: bool,
    jumped_to_w_on_current_action: bool,
    jumped_to_wy_on_current_action: bool,

    w_jump_reloading_time: f32,

    need_to_rotate_w_to_zero: bool,
    time_from_previos_second_mouse_click: f32,

    w_levels_of_map: Vec<f32>,
    current_w_level: usize,
}

pub const PLAYER_MAX_HP: i32 = 100;

const MIN_TIME_BEFORE_RESPAWN: f32 = 1.5;
const MAX_TIME_BEFORE_RESPAWN: f32 = 5.0;

// const self.player_settings.scanner_reloading_time: f32 = 6.5;
// const self.player_settings.scanner_show_enemies_time: f32 = 5.5;
const W_SCANNER_MAX_RADIUS: f32 = 43.0;
const W_SCANNER_EXPANDING_SPEED: f32 = 17.0;

pub const TIME_TO_DIE_SLOWLY: f32 = 0.5;

const CROSSHAIR_INCREASING_SPEED: f32 = 0.35f32;
const CROSSHAIR_DECREASING_SPEED: f32 = 0.04f32;
const CROSSHAIR_MAX_SIZE: f32 = 0.038;
const CROSSHAIR_MIN_SIZE: f32 = 0.028;

const GETTING_DAMAGE_EFFECT_COEF_DECREASE_SPEED: f32 = 5.0;
const DEATH_EFFECT_COEF_INCREASE_SPEED: f32 = 10.0;
const DEATH_EFFECT_COEF_DECREASE_SPEED: f32 = 3.0;

#[derive(Clone)]
pub enum PlayerMessage {
    DealDamageAndAddForce(u32, Vec4, Vec4),
    NewPeerConnected(u128),
    Telefrag,
    DieImmediately,
    DieSlowly,
}


impl Actor for Player {
    fn recieve_message(
        &mut self,
        message: Message,
        engine_handle: &mut EngineHandle,
        physic_system: &PhysicsSystem,
        audio_system: &mut AudioSystem,
        ui_system: &mut UISystem,
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
                    PhysicsMessages::KinematicColliderMessage(message) => {
                        match message {
                            KinematicColliderMessages::ColliderIsStuckInsideObject => {
                                
                                self.die(
                                    true,
                                    engine_handle,
                                    physic_system,
                                    audio_system,
                                    ui_system
                                );
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            },
            MessageType::SpecificActorMessage(message) => {
                match message {
                    SpecificActorMessage::PLayerMessage(message) => {
                        match message {
                            PlayerMessage::Telefrag => {
                                self.die(
                                    true,
                                    engine_handle,
                                    physic_system,
                                    audio_system,
                                    ui_system,
                                );
                            }

                            PlayerMessage::DieImmediately => {
                                self.die(
                                    true,
                                    engine_handle,
                                    physic_system,
                                    audio_system,
                                    ui_system,
                                );
                            }

                            PlayerMessage::DieSlowly => {
                                self.die(
                                    true,
                                    engine_handle,
                                    physic_system,
                                    audio_system,
                                    ui_system,
                                );
                            }

                            PlayerMessage::DealDamageAndAddForce(damage, force, _) => {
                                self.get_damage_and_add_force(
                                    *damage as i32,
                                    *force,
                                    physic_system,
                                    audio_system,
                                    ui_system,
                                    engine_handle,
                                );
                            }

                            PlayerMessage::NewPeerConnected(peer_id) => {

                                // println!("new peer {} connected, replicate my body", peer_id);
                                
                                engine_handle.send_command(
                                    Command {
                                        sender: self.id.unwrap(),
                                        command_type: CommandType::NetCommand(
                                            NetCommand::SendDirectNetMessageReliable(
                                                NetMessageToPlayer::RemoteCommand(
                                                    RemoteCommand::SpawnPlayersDollActor(
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


    fn set_id(&mut self, id: ActorID) {
        self.id = Some(id);

        self.inner_state.collider.set_id(id);
        self.inner_state.collider_for_others[0].set_id(id);
    }


    fn get_id(&self) -> Option<ActorID> {
        self.id
    }

    fn get_physical_element(&mut self) -> Option<PhysicalElement> {
        if self.inner_state.is_enable {
            let collider_container = PhysicalElement {
                transform: &mut self.inner_state.transform,
                kinematic_collider: Some((&mut self.inner_state.collider, None)),
                static_colliders: None,
                dynamic_colliders: Some(&mut self.inner_state.collider_for_others),
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
        ui_system: &mut UISystem,
        time_system: &mut TimeSystem,
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

        let crosshair = ui_system.get_mut_ui_element(&UIElementType::Crosshair);

        if let UIElement::Image(crosshair) = crosshair {
            crosshair.ui_data.rect.size = RectSize::LockedHeight(self.inner_state.crosshair_size);
        }

        self.inner_state.crosshair_target_size = self.inner_state.crosshair_target_size
            .min(CROSSHAIR_MAX_SIZE); 

        if self.inner_state.crosshair_size < self.inner_state.crosshair_target_size {

            self.inner_state.crosshair_size += CROSSHAIR_INCREASING_SPEED*delta;

            if self.inner_state.crosshair_size >= self.inner_state.crosshair_target_size {
                self.inner_state.crosshair_size = self.inner_state.crosshair_target_size;
                
                self.inner_state.crosshair_target_size = CROSSHAIR_MIN_SIZE;
            }
        } else {
            self.inner_state.crosshair_size =
                (self.inner_state.crosshair_size - CROSSHAIR_DECREASING_SPEED*delta)
                .max(CROSSHAIR_MIN_SIZE);
        }


        self.screen_effects.getting_damage_screen_effect -= delta * GETTING_DAMAGE_EFFECT_COEF_DECREASE_SPEED;
        self.screen_effects.getting_damage_screen_effect = self.screen_effects.getting_damage_screen_effect.clamp(0.0, 1.0);

        self.make_hud_transparency_as_death_screen_effect(ui_system);

        
        let mut player_doll_input_state = PlayerDollInputState {
            move_forward: false,
            move_backward: false,
            move_right: false,
            move_left: false,
            will_jump: false,
            current_w_level: self.current_w_level as u32,
        };


        if self.inner_state.is_alive {

            self.screen_effects.death_screen_effect -= delta*DEATH_EFFECT_COEF_DECREASE_SPEED;
            self.screen_effects.death_screen_effect = self.screen_effects.death_screen_effect.clamp(0.0, 1.0);

            let mut xz = self.view_angle.x;
            let mut yz = self.view_angle.y;
            let mut zw = self.view_angle.w;

            let prev_zw = zw;

            self.time_from_previos_second_mouse_click += delta;

            if self.player_settings.rotation_along_w_standard_method {

                if input.second_mouse.is_action_pressed() {
                    zw = (input.mouse_axis.y * self.player_settings.mouse_sensivity + zw).clamp(-PI/2.0, PI/2.0);
    
                    // xz = input.mouse_axis.x + xz;
                    
                } else {
                    zw *= 1.0 - delta * 3.0;
                    if zw.abs() < 0.00001 {
                        zw = 0.0;
                    }
                    
                    xz = input.mouse_axis.x * self.player_settings.mouse_sensivity + xz;
                    yz = (input.mouse_axis.y * self.player_settings.mouse_sensivity + yz).clamp(-PI/2.0, PI/2.0);
                }
            } else {

                if input.second_mouse.is_action_just_pressed() {
                    self.need_to_rotate_w_to_zero = false;

                    if self.time_from_previos_second_mouse_click < 0. {
                        self.need_to_rotate_w_to_zero = true;
                    }

                    self.time_from_previos_second_mouse_click = 0.0
                }

                if input.second_mouse.is_action_pressed() {
                    if !self.need_to_rotate_w_to_zero {
                        
                        zw = (input.mouse_axis.y * self.player_settings.mouse_sensivity + zw).clamp(-PI/2.0, PI/2.0);
                    
                    } else {
                        zw *= 1.0 - delta * 3.0;
                        if zw.abs() < 0.00001 {
                            zw = 0.0;
                        }

                        // xz = input.mouse_axis.x * self.player_settings.mouse_sensivity + xz;
                        // yz = (input.mouse_axis.y * self.player_settings.mouse_sensivity + yz).clamp(-PI/2.0, PI/2.0);
                    }
    
                    // xz = input.mouse_axis.x + xz;
                    
                } else {
                    if !self.need_to_rotate_w_to_zero {

                        xz = input.mouse_axis.x * self.player_settings.mouse_sensivity + xz;
                        yz = (input.mouse_axis.y * self.player_settings.mouse_sensivity + yz).clamp(-PI/2.0, PI/2.0);
                    
                    } else {
                        
                        zw *= 1.0 - delta * 3.0;
                        if zw.abs() < 0.00001 {
                            zw = 0.0;
                        }
                        
                        xz = input.mouse_axis.x * self.player_settings.mouse_sensivity + xz;
                        yz = (input.mouse_axis.y * self.player_settings.mouse_sensivity + yz).clamp(-PI/2.0, PI/2.0);
                    }
                }
            }
    


            let zw_arrow = ui_system.get_mut_ui_element(&UIElementType::ZWScannerArrow);

            if let UIElement::Image(arrow) = zw_arrow {
                arrow.set_rotation_around_screen_center(-zw+PI/2.0);
            } else {
                panic!("UI Element ZWScannerArrow is not UIImage")
            }

            let zx_arrow = ui_system.get_mut_ui_element(&UIElementType::ZXScannerArrow);

            if let UIElement::Image(arrow) = zx_arrow {
                arrow.set_rotation_around_screen_center(xz-PI/2.0);
            } else {
                panic!("UI Element ZXScannerArrow is not UIImage")
            }

            let h_pointer = ui_system.get_mut_ui_element(&UIElementType::ScannerHPointer);

            if let UIElement::Image(h_pointer) = h_pointer {
                let h = {
                    ((self.get_position().w / 6.0) - 0.7)
                        .clamp(-0.7, 0.8)
                };
                
                h_pointer.set_position(Vec2::new(0.002, h));
            } else {
                panic!("UI Element ScannerHPointer is not UIImage")
            }
    
    
            // let normal_rotation = Mat4::from_cols_slice(&[
            //     x.cos(),    y.sin() * x.sin(),  y.cos() * x.sin(),  0.0,
            //     0.0,        y.cos(),            -y.sin(),           0.0,
            //     -x.sin(),   y.sin() * x.cos(),  y.cos()*x.cos(),    0.0,
            //     0.0,        0.0,                0.0,                1.0
            // ]);

            let zy_rotation = Mat4::from_rotation_x(-yz);

            let zx_rotation = Mat4::from_rotation_y(-xz);
    
            let zw_rotation = Mat4::from_cols_slice(&[
                1.0,    0.0,    0.0,        0.0,
                0.0,    1.0,    0.0,        0.0,
                0.0,    0.0,    zw.cos(),   zw.sin(),
                0.0,    0.0,    -zw.sin(),   zw.cos()
            ]);

            self.inner_state.zw_rotation = zw_rotation;
            self.inner_state.zy_rotation = zy_rotation;
            self.inner_state.zx_rotation = zx_rotation;
    
            self.set_rotation_matrix(zw_rotation * zy_rotation * zx_rotation);
    
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
    
            let xz_player_rotation = Mat4::from_rotation_y(xz);
            self.view_angle = Vec4::new(xz, yz, 0.0, zw);

            let base_pitch = {
                0.8.lerp(
                    1.5,
                    (std::f64::consts::PI/2.0 + zw as f64) / std::f64::consts::PI
                )
            };

            let addition_pitch = {
                self.rotating_around_w_sound_pitch * (1.0-delta as f64*22.0) +
                ((prev_zw - zw) as f64).abs() * 2.0
            };

            self.rotating_around_w_sound_pitch = addition_pitch;

            let gain = {
                self.rotating_around_w_sound_gain * (1.0-(delta*42.0)) +
                (prev_zw - zw).abs() * 10.0
            };

            self.rotating_around_w_sound_gain = gain;

            audio_system.sound_set_pitch_and_gain(
                self.rotating_around_w_sound_handle,
                base_pitch,//D + addition_pitch,
                gain
            );




            let shift_pitch = {
                1.0.lerp(
                    1.5,
                    0.5 +
                    (
                        (self.get_position().w - self.player_previous_w_position) *
                        10.0
                    ).clamp(-0.5, 0.5)
                )
            };

            let shift_gain = {
                0.0.lerp(
                    1.0,
                    (
                        (self.get_position().w - self.player_previous_w_position).abs() *
                        20.0
                    ).clamp(0.0, 1.0)
                )
            };

            // let addition_pitch = {
            //     self.rotating_around_w_sound_pitch * (1.0-delta as f64*22.0) +
            //     ((prev_zw - zw) as f64).abs() * 2.0
            // };

            // self.rotating_around_w_sound_pitch = addition_pitch;

            // let gain = {
            //     self.rotating_around_w_sound_gain * (1.0-(delta*42.0)) +
            //     (prev_zw - zw).abs() * 10.0
            // };

            self.rotating_around_w_sound_gain = gain;

            audio_system.sound_set_pitch_and_gain(
                self.shifting_along_w_sound_handle,
                shift_pitch as f64,//D + addition_pitch,
                shift_gain
            );
    
            // self.inner_state.collision.transform.rotation *= new_rotation_matrix;
    
            match self.active_hands_slot {
                ActiveHandsSlot::Zero => {
                    self.hands_slot_0.process_input(my_id, &mut self.inner_state, &input, physic_system, audio_system, ui_system, engine_handle, delta);

                    if let Some(device) = &mut self.hands_slot_1 {
                        device.process_while_deactive(my_id, &mut self.inner_state, &input, physic_system, audio_system, ui_system, engine_handle, delta);
                    }
                    if let Some(device) = &mut self.hands_slot_2 {
                        device.process_while_deactive(my_id, &mut self.inner_state, &input, physic_system, audio_system, ui_system, engine_handle, delta);
                    }
                    if let Some(device) = &mut self.hands_slot_3 {
                        device.process_while_deactive(my_id, &mut self.inner_state, &input, physic_system, audio_system, ui_system, engine_handle, delta);
                    }
                },
                ActiveHandsSlot::First => {
                    if let Some(device) = self.hands_slot_1.as_mut() {
                        device.process_input(my_id, &mut self.inner_state, &input, physic_system, audio_system, ui_system, engine_handle, delta);
                    }

                    self.hands_slot_0.process_while_deactive(my_id, &mut self.inner_state, &input, physic_system, audio_system, ui_system, engine_handle, delta);
                    if let Some(device) = &mut self.hands_slot_2 {
                        device.process_while_deactive(my_id, &mut self.inner_state, &input, physic_system, audio_system, ui_system, engine_handle, delta);
                    }
                    if let Some(device) = &mut self.hands_slot_3 {
                        device.process_while_deactive(my_id, &mut self.inner_state, &input, physic_system, audio_system, ui_system, engine_handle, delta);
                    }
                },
                ActiveHandsSlot::Second => {
                    if let Some(device) = self.hands_slot_2.as_mut() {
                        device.process_input(my_id, &mut self.inner_state, &input, physic_system, audio_system, ui_system, engine_handle, delta);
                    }

                    self.hands_slot_0.process_while_deactive(my_id, &mut self.inner_state, &input, physic_system, audio_system, ui_system, engine_handle, delta);
                    if let Some(device) = &mut self.hands_slot_1 {
                        device.process_while_deactive(my_id, &mut self.inner_state, &input, physic_system, audio_system, ui_system, engine_handle, delta);
                    }
                    if let Some(device) = &mut self.hands_slot_3 {
                        device.process_while_deactive(my_id, &mut self.inner_state, &input, physic_system, audio_system, ui_system, engine_handle, delta);
                    }
                },
                ActiveHandsSlot::Third => {
                    if let Some(device) = self.hands_slot_3.as_mut() {
                        device.process_input(my_id, &mut self.inner_state, &input, physic_system, audio_system, ui_system, engine_handle, delta);
                    }

                    self.hands_slot_0.process_while_deactive(my_id, &mut self.inner_state, &input, physic_system, audio_system, ui_system, engine_handle, delta);
                    if let Some(device) = &mut self.hands_slot_1 {
                        device.process_while_deactive(my_id, &mut self.inner_state, &input, physic_system, audio_system, ui_system, engine_handle, delta);
                    }
                    if let Some(device) = &mut self.hands_slot_2 {
                        device.process_while_deactive(my_id, &mut self.inner_state, &input, physic_system, audio_system, ui_system, engine_handle, delta);
                    }
                }
            }
    
            for device in self.devices.iter_mut() {
                if let Some(device) = device {
                    device.process_input(my_id, &mut self.inner_state, &input, physic_system, audio_system, ui_system, engine_handle, delta);
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
                    ui_system,
                    engine_handle,
                );
                self.active_hands_slot = ActiveHandsSlot::Zero;

                self.hands_slot_0.activate(
                    self.get_id().expect("Player have not ActorID"),
                    &mut self.inner_state,
                    physic_system,
                    audio_system,
                    ui_system,
                    engine_handle,
                );
            }
    
            if input.activate_hand_slot_1.is_action_just_pressed() {
                if self.hands_slot_1.is_some() {
                    self.deavctivate_previous_device(
                        ActiveHandsSlot::First,
                        physic_system,
                        audio_system,
                        ui_system,
                        engine_handle,
                    );
                    self.active_hands_slot = ActiveHandsSlot::First;

                    self.hands_slot_1.as_mut().unwrap().activate(
                        self.id.expect("Player have not ActorID"),
                        &mut self.inner_state,
                        physic_system,
                        audio_system,
                        ui_system,
                        engine_handle,
                    );
                }
            }
    
            if input.activate_hand_slot_2.is_action_just_pressed() {
                if self.hands_slot_2.is_some() {
                    self.deavctivate_previous_device(
                        ActiveHandsSlot::Second,
                        physic_system,
                        audio_system,
                        ui_system,
                        engine_handle,
                    );
                    self.active_hands_slot = ActiveHandsSlot::Second;

                    self.hands_slot_2.as_mut().unwrap().activate(
                        self.id.expect("Player have not ActorID"),
                        &mut self.inner_state,
                        physic_system,
                        audio_system,
                        ui_system,
                        engine_handle,
                    );
                }
            }
    
            if input.activate_hand_slot_3.is_action_just_pressed() {
                if self.hands_slot_3.is_some() {
                    self.deavctivate_previous_device(
                        ActiveHandsSlot::Third,
                        physic_system,
                        audio_system,
                        ui_system,
                        engine_handle,
                    );
                    self.active_hands_slot = ActiveHandsSlot::Third;

                    self.hands_slot_3.as_mut().unwrap().activate(
                        self.id.expect("Player have not ActorID"),
                        &mut self.inner_state,
                        physic_system,
                        audio_system,
                        ui_system,
                        engine_handle,
                    );
                }
            }
    
            let mut movement_vec = Vec4::ZERO;
    
            if input.move_forward.is_action_pressed() {
                movement_vec += Vec4::NEG_Z;

                player_doll_input_state.move_forward = true;
            }
    
            if input.move_backward.is_action_pressed() {
                movement_vec += Vec4::Z;

                player_doll_input_state.move_backward = true;
            }
    
            if input.move_right.is_action_pressed() {
                movement_vec += Vec4::X;

                player_doll_input_state.move_right = true;
            }
    
            if input.move_left.is_action_pressed() {
                movement_vec += Vec4::NEG_X;
                
                player_doll_input_state.move_left = true;
            }
    
            if let Some(vec) = movement_vec.try_normalize() {
                movement_vec = vec;
            }
    
            if input.jump.is_action_just_pressed() {

                player_doll_input_state.will_jump = true;

                self.jumped_to_y_on_current_action = false;
    
                if self.inner_state.collider.is_on_y_ground {
                    self.inner_state.collider.add_force(Vec4::Y * self.player_settings.jump_y_speed);

                    self.jumped_to_y_on_current_action = true;
                    
                    player_doll_input_state.will_jump = false;
                }
            }

            if input.jump.is_action_pressed() {
                if !self.jumped_to_y_on_current_action {
                    if self.inner_state.collider.is_on_y_ground {
                        self.inner_state.collider.add_force(Vec4::Y * self.player_settings.jump_y_speed);

                        self.jumped_to_y_on_current_action = true;
                        
                        player_doll_input_state.will_jump = false;
                    }
                }
            } else {
                player_doll_input_state.will_jump = false;
            }

            self.w_jump_reloading_time += delta;

            if input.jump_wy.is_action_just_pressed() {

                let next_w_level = self.current_w_level + 1;

                if next_w_level < self.w_levels_of_map.len() {
                    self.current_w_level = next_w_level;

                    player_doll_input_state.current_w_level = next_w_level as u32;
                }
            }

            if input.jump_w.is_action_just_pressed() {

                if self.current_w_level > 0 {

                    let next_w_level = self.current_w_level - 1;
    
                    if next_w_level < self.w_levels_of_map.len() {
                        self.current_w_level = next_w_level;

                        player_doll_input_state.current_w_level = next_w_level as u32;
                    }
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
    
            if input.w_scanner.is_action_just_pressed() {
                if !self.w_scanner_enable {
                    if self.w_scanner_reloading_time >= self.player_settings.scanner_reloading_time {
                        self.w_scanner_enable = true;

                        audio_system.spawn_non_spatial_sound(
                            Sound::ScannerSound,
                            1.0,
                            1.0,
                            false,
                            true,
                            fyrox_sound::source::Status::Playing
                        );

                        self.w_scanner_enemies_show_time = 0.0;
    
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

            self.w_scanner_enemies_show_time += delta;
    
            if !self.w_scanner_enable {
    
                if self.w_scanner_reloading_time < self.player_settings.scanner_reloading_time {
                    self.w_scanner_reloading_time += delta;
                }

                
            }
    
            
    
            if self.inner_state.collider.is_enable {
    
                if self.is_gravity_y_enabled {
                    movement_vec = self.get_zx_rotation_matrix().inverse() * movement_vec;
    
                    if self.inner_state.collider.is_on_y_ground {
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

                    let target_w_pos = self.w_levels_of_map
                        .get(self.current_w_level)
                        .expect("Player's carrent_w_level is not exist in w_levels_of_map")
                        .clone();

                    let w_dif = target_w_pos - self.get_position().w;

                    self.inner_state.collider.current_velocity.w +=
                        self.player_settings.gravity_w_speed*w_dif.clamp(-1.0, 1.0);

                    self.inner_state.collider.current_velocity.w *=
                        (w_dif * 5.0_f32)
                        .abs()
                        .clamp(0.0, 1.0);
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
    
            self.screen_effects.w_scanner_is_active = self.w_scanner_enable;
            self.screen_effects.w_scanner_radius = self.w_scanner_radius;
            self.screen_effects.w_scanner_ring_intesity = {
                let mut intensity = W_SCANNER_MAX_RADIUS - self.w_scanner_radius;
    
                intensity /= W_SCANNER_MAX_RADIUS/3.0;
    
                intensity.clamp(0.0, 1.0)
            };
            self.screen_effects.w_scanner_enemies_intesity = {
                let intensity = self.player_settings.scanner_show_enemies_time - self.w_scanner_enemies_show_time;
    
                intensity.clamp(0.0, 1.0)
            };

            self.player_previous_w_position = self.get_position().w;

            // ---------------------------------------------------
            // temp!
            // y death plane
            if self.get_position().y < -20.0 {
                self.die(
                    true,
                    engine_handle,
                    physic_system,
                    audio_system,
                    ui_system
                );
            }
            // ---------------------------------------------------

        } else {
            //while player is not alive

            self.after_death_timer += delta;

            self.screen_effects.death_screen_effect += delta*DEATH_EFFECT_COEF_INCREASE_SPEED;
            self.screen_effects.death_screen_effect = self.screen_effects.death_screen_effect.clamp(0.0, 1.0);

            match self.active_hands_slot {
                ActiveHandsSlot::Zero => {
                    self.hands_slot_0.process_while_player_is_not_alive(my_id, &mut self.inner_state, &input, physic_system, audio_system, ui_system, engine_handle, delta);

                },
                ActiveHandsSlot::First => {
                    if let Some(device) = self.hands_slot_1.as_mut() {
                        device.process_while_player_is_not_alive(my_id, &mut self.inner_state, &input, physic_system, audio_system, ui_system, engine_handle, delta);
                    }

                },
                ActiveHandsSlot::Second => {
                    if let Some(device) = self.hands_slot_2.as_mut() {
                        device.process_while_player_is_not_alive(my_id, &mut self.inner_state, &input, physic_system, audio_system, ui_system, engine_handle, delta);
                    }

                },
                ActiveHandsSlot::Third => {
                    if let Some(device) = self.hands_slot_3.as_mut() {
                        device.process_while_player_is_not_alive(my_id, &mut self.inner_state, &input, physic_system, audio_system, ui_system, engine_handle, delta);
                    }

                }
            }
    
            for device in self.devices.iter_mut() {
                if let Some(device) = device {
                    device.process_while_player_is_not_alive(my_id, &mut self.inner_state, &input, physic_system, audio_system, ui_system, engine_handle, delta);
                }
            }

            if self.need_to_die_slowly {
                if self.after_death_timer >= TIME_TO_DIE_SLOWLY {
                    self.need_to_die_slowly = false;
                    self.inner_state.is_enable = false;
                    
                    self.play_die_effects(audio_system, engine_handle);
                }
            }

            if self.is_gravity_w_enabled {
                self.inner_state.collider.add_force(Vec4::NEG_W * self.player_settings.gravity_w_speed);
            }

            if self.is_gravity_y_enabled {
                self.inner_state.collider.add_force(Vec4::NEG_Y * self.player_settings.gravity_y_speed);
            }

            if self.after_death_timer >= self.player_settings.max_respawn_timer {
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
                if self.after_death_timer >= self.player_settings.min_respawn_timer {
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

            self.screen_effects.w_scanner_is_active = self.w_scanner_enable;
            self.screen_effects.w_scanner_radius = self.w_scanner_radius;
            self.screen_effects.w_scanner_ring_intesity = {
                let mut intensity = W_SCANNER_MAX_RADIUS - self.w_scanner_radius;
    
                intensity /= W_SCANNER_MAX_RADIUS/3.0;
    
                intensity = intensity.clamp(0.0, 1.0);

                intensity -= self.after_death_timer * 2.0;
                    
                intensity.clamp(0.0, 1.0)
            };
            self.screen_effects.w_scanner_enemies_intesity = {

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

        let remote_velocity = {
            let mut v = self.inner_state.collider.current_velocity;

            for force in self.inner_state.collider.forces.iter() {
                v += *force;
            }

            v.to_array()
        };

        engine_handle.send_command(Command{
            sender: my_id,
            command_type: CommandType::NetCommand(
                NetCommand::SendBoardcastNetMessageUnreliable(
                    NetMessageToPlayer::RemoteDirectMessage(
                        my_id,
                        RemoteMessage::SetPlayerDollState(
                            self.inner_state.transform.to_serializable_transform(),
                            player_doll_input_state.serialize(),
                            remote_velocity,
                            time_system.get_server_time()
                        )
                    )
                )
            )
        });
    }
}



impl Player {

    pub fn new(
        master: InputMaster,
        player_settings: PlayerSettings,
        audio_system: &mut AudioSystem,
        w_levels_of_map: Vec<f32>,
    ) -> Self {
        
        let screen_effects = PlayerScreenEffects {
            w_scanner_is_active: false,
            w_scanner_radius: 0.0,
            w_scanner_ring_intesity: 0.0,
            w_scanner_enemies_intesity: 0.0,
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

        let shifting_along_w_sound_handle = audio_system.spawn_non_spatial_sound(
            Sound::ShiftingAlongW,
            0.0,
            1.0,
            true,
            false,
            fyrox_sound::source::Status::Playing
        );

        let w_scanner_reloading_time =  player_settings.scanner_reloading_time;
        let w_scanner_enemies_show_time =  player_settings.scanner_show_enemies_time;
        let after_death_timer =  player_settings.min_respawn_timer;
        
        Player {
            id: None,

            inner_state: PlayerInnerState::new(Transform::new(), &player_settings, false, false),
            active_hands_slot: ActiveHandsSlot::Zero,

            hands_slot_0: Box::new(HoleGun::new(
                player_settings.energy_gun_hole_size_mult, 
                player_settings.energy_gun_add_force_mult, 
                player_settings.energy_gun_damage_mult, 
                player_settings.energy_gun_restoring_speed,
            )),
            hands_slot_1: Some(Box::new(MachineGun::new(
                player_settings.machinegun_damage,
                player_settings.machinegun_add_force, 
                player_settings.machinegun_heat_add_on_shot, 
                player_settings.machinegun_cooling_speed
            ))),
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
            w_scanner_reloading_time,
            w_scanner_enemies_show_time,

            after_death_timer,
            need_to_die_slowly: false,

            rotating_around_w_sound_handle,
            rotating_around_w_sound_pitch: 1.0,
            rotating_around_w_sound_gain: 0.0,

            shifting_along_w_sound_handle,
            shifting_along_w_sound_pitch: 1.0,
            shifting_along_w_sound_gain: 0.0,
            player_previous_w_position: 0.0,

            jumped_to_y_on_current_action: false,
            jumped_to_w_on_current_action: false,
            jumped_to_wy_on_current_action: false,

            w_jump_reloading_time: 0.0,

            need_to_rotate_w_to_zero: true,
            time_from_previos_second_mouse_click: 0.0,

            w_levels_of_map,
            current_w_level: 0,
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

    pub fn get_zw_rotation_matrix(&self) -> Mat4 {
        self.inner_state.zw_rotation
    }

    pub fn get_zy_rotation_matrix(&self) -> Mat4 {
        self.inner_state.zy_rotation
    }

    pub fn get_zx_rotation_matrix(&self) -> Mat4 {
        self.inner_state.zx_rotation
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
        physic_system: &PhysicsSystem,
        audio_system: &mut AudioSystem,
        ui_system: &mut UISystem,
        engine_handle: &mut EngineHandle,
    ) {

        audio_system.spawn_non_spatial_sound(
            Sound::PlayerHited,
            0.6.lerp(1.0, (damage as f32/PLAYER_MAX_HP as f32).clamp(0.0, 1.0)),
            1.0,
            false,
            true,
            fyrox_sound::source::Status::Playing
        );

        self.screen_effects.getting_damage_screen_effect = 1.0;

        self.inner_state.hp -= damage;
        self.inner_state.collider.add_force(force);

        let health_bar = ui_system.get_mut_ui_element(&UIElementType::HeathBar);

        if let UIElement::ProgressBar(bar) = health_bar {
            let bar_value = {
                (self.inner_state.hp as f32 / PLAYER_MAX_HP as f32)
                    .clamp(0.0, 1.0)
            };

            bar.set_bar_value(bar_value)

        } else {
            panic!("Health Bar is not Progress Bar")
        }

        if self.inner_state.hp <= 0 {
            if damage >= PLAYER_MAX_HP {
                self.die(
                    true,
                    engine_handle,
                    physic_system,
                    audio_system,
                    ui_system,
                );
            } else {

                // self.die(false, engine_handle, physic_system, audio_system);
                
                // temproral solution
                self.die(
                    true,
                    engine_handle,
                    physic_system,
                    audio_system,
                    ui_system,
                );
            }
        }
    }

    fn deavctivate_previous_device(&mut self,
        new_active_slot: ActiveHandsSlot,
        physic_system: &PhysicsSystem,
        audio_system: &mut AudioSystem,
        ui_system: &mut UISystem,
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
                            ui_system,
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
                            ui_system,
                            engine_handle,
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
                            ui_system,
                            engine_handle,
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
                            ui_system,
                            engine_handle,
                        );
                }
            }
        }
    }

    fn telefrag(&mut self, audio_system: &mut AudioSystem, engine_handle: &mut EngineHandle) {
        self.die_immediately(audio_system, engine_handle);
    }

    fn die_immediately(
        &mut self,
        audio_system: &mut AudioSystem,
        engine_handle: &mut EngineHandle,
    ) {
        if self.inner_state.is_alive {

            self.inner_state.is_alive = false;
            self.inner_state.is_enable = false;
            self.need_to_die_slowly = false;
            self.after_death_timer = 0.0;

            self.play_die_effects(audio_system, engine_handle);

            engine_handle.send_command(
                Command {
                    sender: self.get_id().expect("Player have not ActorID"),
                    command_type: CommandType::NetCommand(
                        NetCommand::SendBoardcastNetMessageReliable(
                            NetMessageToPlayer::RemoteDirectMessage(
                                self.get_id().expect("Player have not ActorID"),
                                RemoteMessage::DieImmediately
                            )
                        )
                    )
                }
            );
        }
    }

    fn make_hud_transparency_as_death_screen_effect(&mut self, ui: &mut UISystem) {
        let a = 1.0 - self.screen_effects.death_screen_effect.clamp(0.0, 1.0);

        let hud_elem = ui.get_mut_ui_element(&UIElementType::Crosshair);

        if let UIElement::Image(elem) = hud_elem {
            elem.ui_data.rect.transparency = a;
        }

        let hud_elem = ui.get_mut_ui_element(&UIElementType::Scanner);

        if let UIElement::Image(elem) = hud_elem {
            elem.ui_data.rect.transparency = a;
        }

        let hud_elem = ui.get_mut_ui_element(&UIElementType::ScannerHPointer);

        if let UIElement::Image(elem) = hud_elem {
            elem.ui_data.rect.transparency = a;
        }

        let hud_elem = ui.get_mut_ui_element(&UIElementType::ZWScannerArrow);

        if let UIElement::Image(elem) = hud_elem {
            elem.ui_data.rect.transparency = a;
        }

        let hud_elem = ui.get_mut_ui_element(&UIElementType::ZXScannerArrow);

        if let UIElement::Image(elem) = hud_elem {
            elem.ui_data.rect.transparency = a;
        }

        let hud_elem = ui.get_mut_ui_element(&UIElementType::HeathBar);

        if let UIElement::ProgressBar(elem) = hud_elem {
            elem.ui_data.rect.transparency = a;
        }

        let hud_elem = ui.get_mut_ui_element(&UIElementType::EnergyGunBar);

        if let UIElement::ProgressBar(elem) = hud_elem {
            elem.ui_data.rect.transparency = a;
        }

        let hud_elem = ui.get_mut_ui_element(&UIElementType::MachinegunBar);

        if let UIElement::ProgressBar(elem) = hud_elem {
            elem.ui_data.rect.transparency = a;
        }

        let hud_elem = ui.get_mut_ui_element(&UIElementType::LeftScannerDsiplay);

        if let UIElement::ScannerDisplay(elem) = hud_elem {
            elem.ui_data.rect.transparency = a;
        }

        let hud_elem = ui.get_mut_ui_element(&UIElementType::RightScannerDsiplay);

        if let UIElement::ScannerDisplay(elem) = hud_elem {
            elem.ui_data.rect.transparency = a;
        }

    }

    fn play_die_effects(&mut self, audio_system: &mut AudioSystem, engine_handle: &mut EngineHandle) {
        
        audio_system.spawn_non_spatial_sound(
            Sound::PlayerDied,
            0.37,
            1.0,
            false,
            true,
            fyrox_sound::source::Status::Playing
        );

        let players_death_explode = PlayersDeathExplosion::new(
            self.get_transform().get_position()
        );

        self.screen_effects.death_screen_effect = 0.0;
        self.screen_effects.getting_damage_screen_effect = 1.0;

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
                            NetMessageToPlayer::RemoteDirectMessage(
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
        ui_system: &mut UISystem,

    ) {
        let my_id = self.get_id().expect("Player have not ActorID");

        match self.active_hands_slot {
            ActiveHandsSlot::Zero => {
                self.hands_slot_0.deactivate(
                    my_id,
                    &mut self.inner_state,
                    physic_system,
                    audio_system,
                    ui_system,
                    engine_handle,
                );

            },
            ActiveHandsSlot::First => {
                if let Some(device) = self.hands_slot_1.as_mut() {
                    device.deactivate(
                        my_id,
                        &mut self.inner_state,
                        physic_system,
                        audio_system,
                        ui_system,
                        engine_handle,
                    );
                }

            },
            ActiveHandsSlot::Second => {
                if let Some(device) = self.hands_slot_2.as_mut() {
                    device.deactivate(
                        my_id,
                        &mut self.inner_state,
                        physic_system,
                        audio_system,
                        ui_system,
                        engine_handle,
                    );
                }

            },
            ActiveHandsSlot::Third => {
                if let Some(device) = self.hands_slot_3.as_mut() {
                    device.deactivate(
                        my_id,
                        &mut self.inner_state,
                        physic_system,
                        audio_system,
                        ui_system,
                        engine_handle,
                    );
                }

            }
        }

        for device in self.devices.iter_mut() {
            if let Some(device) = device {
                device.deactivate(
                    my_id,
                    &mut self.inner_state,
                    physic_system,
                    audio_system,
                    ui_system,
                    engine_handle,
                );
            }
        }

        self.restore_scanner_values();

        self.restore_w_shift_and_rotate_values();

        if die_immediately {
            self.die_immediately(audio_system, engine_handle);
        } else {
            self.die_slowly(engine_handle);
        }
    }



    pub fn respawn(
        &mut self,
        spawn: Spawn,
        physics_system: &PhysicsSystem,
        ui_system: &mut UISystem,
        audio_system: &mut AudioSystem,
        engine_handle: &mut EngineHandle,
    ) {
        self.inner_state.is_alive = true;
        self.inner_state.is_enable = true;
        self.inner_state.hp = PLAYER_MAX_HP;

        self.view_angle = Vec4::ZERO;

        self.restore_scanner_values();

        self.restore_w_shift_and_rotate_values();

        audio_system.spawn_non_spatial_sound(
            Sound::PlayerRespawned,
            1.0,
            1.0,
            false,
            true,
            fyrox_sound::source::Status::Playing,
        );

        let health_bar = ui_system.get_mut_ui_element(&UIElementType::HeathBar);

        if let UIElement::ProgressBar(bar) = health_bar {
            let bar_value = {
                (self.inner_state.hp as f32 / PLAYER_MAX_HP as f32)
                    .clamp(0.0, 1.0)
            };

            bar.set_bar_value(bar_value)
            
        } else {
            panic!("Health Bar is not UIProgressBar")
        }

        let my_id = self.get_id().expect("Player have not ActorID");

        match self.active_hands_slot {
            ActiveHandsSlot::Zero => {
                self.hands_slot_0.activate(
                    my_id,
                    &mut self.inner_state,
                    physics_system,
                    audio_system,
                    ui_system,
                    engine_handle,
                );

            },
            ActiveHandsSlot::First => {
                if let Some(device) = self.hands_slot_1.as_mut() {
                    device.activate(
                        my_id,
                        &mut self.inner_state,
                        physics_system,
                        audio_system,
                        ui_system,
                        engine_handle,
                    );
                }

            },
            ActiveHandsSlot::Second => {
                if let Some(device) = self.hands_slot_2.as_mut() {
                    device.activate(
                        my_id,
                        &mut self.inner_state,
                        physics_system,
                        audio_system,
                        ui_system,
                        engine_handle,
                    );
                }

            },
            ActiveHandsSlot::Third => {
                if let Some(device) = self.hands_slot_3.as_mut() {
                    device.activate(
                        my_id,
                        &mut self.inner_state,
                        physics_system,
                        audio_system,
                        ui_system,
                        engine_handle,
                    );
                }

            }
        }

        for device in self.devices.iter_mut() {
            if let Some(device) = device {
                device.activate(
                    my_id,
                    &mut self.inner_state,
                    physics_system,
                    audio_system,
                    ui_system,
                    engine_handle,
                );
            }
        }

        self.screen_effects.w_scanner_ring_intesity = 0.0;
        self.screen_effects.w_scanner_radius = 0.0;
        self.screen_effects.w_scanner_is_active = false;
        self.w_scanner_reloading_time = self.player_settings.scanner_reloading_time;

        self.inner_state.collider.reset_forces_and_velocity();

        self.inner_state.transform = Transform::from_position(spawn.spawn_position);

        self.current_w_level = spawn.w_level;

        self.player_previous_w_position = spawn.spawn_position.w;

        let hits = physics_system.sphere_cast_on_dynamic_colliders(spawn.spawn_position, self.get_collider_radius());

        for hit in hits {
            engine_handle.send_direct_message(
                hit.hited_actors_id.expect("In respawn func in death on respawn hit have not ActorID"),
                Message {
                    from: self.get_id().expect("Player have not ID in respawn func"),
                    message: MessageType::SpecificActorMessage(
                        SpecificActorMessage::PLayerMessage(
                            PlayerMessage::Telefrag
                        )
                    )
                }
            )
        }

        let player_doll_input_state = PlayerDollInputState {
            move_forward: false,
            move_backward: false,
            move_right: false,
            move_left: false,
            will_jump: false,
            current_w_level: self.current_w_level as u32,
        };

        engine_handle.send_command(
            Command {
                sender: self.get_id().expect("Player have not ActorID"),
                command_type: CommandType::NetCommand(
                    NetCommand::SendBoardcastNetMessageReliable(
                        NetMessageToPlayer::RemoteDirectMessage(
                            self.get_id().expect("Player have not ActorID"),
                            RemoteMessage::PlayerRespawn(
                                self.inner_state.transform.to_serializable_transform(),
                                player_doll_input_state.serialize(),
                                Vec4::ZERO.to_array(),
                                
                            )
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

    fn restore_scanner_values(&mut self) {
        self.w_scanner_enable = false;
        self.w_scanner_radius = 0.0;
        self.w_scanner_reloading_time = self.player_settings.scanner_reloading_time;
        self.w_scanner_enemies_show_time = self.player_settings.scanner_show_enemies_time;
    }

    fn restore_w_shift_and_rotate_values(&mut self) {
        self.rotating_around_w_sound_pitch = 1.0;
        self.rotating_around_w_sound_gain = 0.0;
        self.shifting_along_w_sound_pitch = 1.0;
        self.shifting_along_w_sound_gain = 0.0;
        self.player_previous_w_position = 0.0;
    }
}