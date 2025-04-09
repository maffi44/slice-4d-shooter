pub mod player_input_master;
pub mod player_inner_state;
pub mod player_settings;

use bincode::de;
use client_server_protocol::{
    NetCommand,
    NetMessageToPlayer,
    NetMessageToServer,
    RemoteCommand,
    RemoteMessage,
    Team
};
use rand::{seq::SliceRandom, thread_rng};

use crate::{
    actor::{
        device::{
            holegun::HoleGun,
            Device,
            DeviceType
        },
        players_doll::PlayerDollInputState,
        Actor,
        ActorID,
        CommonActorsMessage,
        Component,
        Message,
        MessageType,
        SpecificActorMessage
    },
    engine::{
        audio::{
            AudioSystem,
            Sound
        },
        effects::EffectsSystem,
        engine_handle::{
            Command,
            CommandType,
            EngineHandle
        },
        input::ActionsFrameState, physics::{
            colliders_container::PhysicalElement,
            dynamic_collider::PlayersDollCollider,
            kinematic_collider::{
                KinematicCollider,
                KinematicColliderMessage
            },
            PhysicsSystem
        }, render::{camera::Camera, VisualElement}, time::TimeSystem, ui::{
            self, RectSize, UIElement, UIElementType, UISystem
        }, world::level::Spawn
    },
    transform::Transform,
};

use self::{
    player_input_master::InputMaster,
    player_settings::PlayerSettings,
    player_inner_state::PlayerInnerState,
};

use core::panic;
use std::{collections::btree_set::Difference, f32::consts::PI, usize};
use fyrox_core::pool::Handle;
use fyrox_sound::source::{SoundSource, Status};
use glam::{
    FloatExt, Mat4, Vec2, Vec3, Vec4
};

use super::{
    device::machinegun::MachineGun, flag::{FlagMessage, FlagStatus}, move_w_bonus::{BonusSpotStatus, MoveWBonusSpotMessage}, mover_w::MoverWMessage, players_death_explosion::PlayersDeathExplosion, players_doll::PlayersDollMessage, session_controller::{SessionControllerMessage, DEFAULT_TEAM}, ControlledActor, PhysicsMessages
};

#[derive(Clone)]
pub enum PlayerMovingState
{
    // f32 - lock position on w axis
    MovingPerpendicularW(f32),
    // f32 - lock position on z axis
    MovingParallelW(f32),
    // f32 - how much time moving free player have
    MovingFree(f32)
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

impl Default for PlayerScreenEffects
{
    fn default() -> Self {
        PlayerScreenEffects {
            w_scanner_is_active: false,
            w_scanner_radius: 0.0,
            w_scanner_ring_intesity: 0.0,
            w_scanner_enemies_intesity: 0.0,
            death_screen_effect: 0.0,
            getting_damage_screen_effect: 0.0,
        }
    }
}


pub struct Player {
    id: Option<ActorID>,

    inner_state: PlayerInnerState,

    saved_angle_of_rotation: Vec4,

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

    // explore_w_position: f32,
    // explore_w_coefficient: f32,

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

    jumped_to_y_on_current_action: bool,
    jumped_to_w_on_current_action: bool,
    jumped_to_wy_on_current_action: bool,

    // w_jump_reloading_time: f32,

    // need_to_rotate_w_to_zero: bool,
    // time_from_previos_second_mouse_click: f32,

    // w_levels_of_map: Vec<f32>,
    // current_w_level: usize,

    

    // fisrt_move_w_bonus_transparency_level: f32,
    // second_move_w_bonus_transparency_level: f32,

    flag_pivot_offset: Vec4,

    base_effect_tick_timer: f32,

    holding_player_rotation_along_w: bool,
}
pub const Y_DEATH_PLANE_LEVEL: f32 = -20.0;

pub const PLAYER_MAX_HP: f32 = 100.0;

const MIN_TIME_BEFORE_RESPAWN: f32 = 1.5;
const MAX_TIME_BEFORE_RESPAWN: f32 = 5.0;

// const self.player_settings.scanner_reloading_time: f32 = 6.5;
// const self.player_settings.scanner_show_enemies_time: f32 = 5.5;
const W_SCANNER_MAX_RADIUS: f32 = 21.0;
const W_SCANNER_EXPANDING_SPEED: f32 = 17.0;

pub const TIME_TO_DIE_SLOWLY: f32 = 0.5;

const CROSSHAIR_INCREASING_SPEED: f32 = 0.35f32;
const CROSSHAIR_DECREASING_SPEED: f32 = 0.04f32;
const CROSSHAIR_MAX_SIZE: f32 = 0.038;
const CROSSHAIR_MIN_SIZE: f32 = 0.028;

const GETTING_DAMAGE_EFFECT_COEF_DECREASE_SPEED: f32 = 5.0;
const DEATH_EFFECT_COEF_INCREASE_SPEED: f32 = 10.0;
const DEATH_EFFECT_COEF_DECREASE_SPEED: f32 = 3.0;

const SHOW_CROSSHAIER_HIT_MARK_TIME: f32 = 0.3;

pub const RED_TEAM_COLOR: Vec3 = Vec3::new(3.5, 0.7, 0.08);
pub const BLUE_TEAM_COLOR: Vec3 = Vec3::new(0.08, 0.7, 3.5);

pub const MAX_MOVE_W_BONUSES_I_CAN_HAVE: u32 = 1;

const HAVE_NOT_MOVE_W_BONUS_TRANSPARENCY_LEVEL: f32 = 0.2;

const BASE_EFFECT_HP_IMPACT_SPEED: f32 = 2.6;

const DURATION_OF_MOVING_FREE_BY_BONUS: f32 = 8.0;

pub const PLAYER_FREE_MOVING_SPEED_MULT: f32 = 0.6;

#[derive(Clone)]
pub enum PlayerMessage {
    DealDamageAndAddForce(
        // damage
        u32,
        //force
        Vec4,
        // pos of impact (for spawn get damage effect)
        Vec4,
        // team damage from
        Team,
    ),
    NewPeerConnected(u128),
    Telefrag,
    DieImmediately,
    DieSlowly,
    SetNewTeam(
        // new team you have joined
        Team, 
    )
}


impl Actor for Player {

    fn get_actor_as_controlled(&self) -> Option<&dyn ControlledActor> {
        Some(self)
    }

    fn get_actor_as_controlled_mut(&mut self) -> Option<&mut dyn ControlledActor> {
        Some(self)
    }

    fn recieve_message(
        &mut self,
        message: Message,
        engine_handle: &mut EngineHandle,
        physic_system: &PhysicsSystem,
        audio_system: &mut AudioSystem,
        ui_system: &mut UISystem,
        time_system: &TimeSystem,
        effects_system: &mut EffectsSystem,
    ) {
        let from = message.from;

        let message = message.message;
        
        match message
        {
            MessageType::CommonActorsMessages(message) =>
            {
                match message
                {
                    CommonActorsMessage::SetTransform(transform) =>
                    {
                        self.inner_state.transform = transform;
                    },

                    CommonActorsMessage::Enable(switch) =>
                    {
                        self.inner_state.is_enable = switch;
                    },

                    CommonActorsMessage::IncrementPosition(increment) =>
                    {
                        self.inner_state.transform.increment_position(increment);
                    },
                    CommonActorsMessage::IWasChangedMyId(new_id) => {}
                }
            }

            MessageType::PhysicsMessages(message) =>
            {
                match message {
                    PhysicsMessages::KinematicColliderMessage(message) => {
                        match message {
                            KinematicColliderMessage::ColliderIsStuckInsideObject =>
                            {    
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

            MessageType::SpecificActorMessage(message) =>
            {
                match message
                {
                    // SpecificActorMessage::MoverW(message) =>
                    // {
                    //     match message {
                    //         MoverWMessage::Rotate(lock_z, lock_w, dir) =>
                    //         {
                    //             audio_system.spawn_non_spatial_sound(
                    //                 Sound::WShiftEnd,
                    //                 0.5,
                    //                 1.0,
                    //                 false,
                    //                 true,
                    //                 fyrox_sound::source::Status::Playing
                    //             );

                    //             match self.inner_state.player_moving_state {
                    //                 PlayerMovingState::MovingPerpendicularW(_) =>
                    //                 {
                    //                     self.inner_state.player_moving_state = PlayerMovingState::MovingParallelW(lock_z);
                    //                     self.holding_player_rotation_along_w = false;
                    //                 }
                    //                 PlayerMovingState::MovingParallelW(_) =>
                    //                 {
                    //                     self.inner_state.player_moving_state = PlayerMovingState::MovingPerpendicularW(lock_w);
                    //                     self.holding_player_rotation_along_w = false;
                    //                 }
                    //                 PlayerMovingState::MovingFree(_) => {}
                    //             }
                    //         }
                    //     }
                    // }

                    SpecificActorMessage::PLayerMessage(message) =>
                    {
                        match message {
                            PlayerMessage::Telefrag =>
                            {
                                self.die(
                                    true,
                                    engine_handle,
                                    physic_system,
                                    audio_system,
                                    ui_system,
                                );
                            }

                            PlayerMessage::DieImmediately =>
                            {
                                self.die(
                                    true,
                                    engine_handle,
                                    physic_system,
                                    audio_system,
                                    ui_system,
                                );
                            }

                            PlayerMessage::DieSlowly =>
                            {
                                self.die(
                                    true,
                                    engine_handle,
                                    physic_system,
                                    audio_system,
                                    ui_system,
                                );
                            }

                            PlayerMessage::DealDamageAndAddForce(
                                damage,
                                force,
                                _,
                                team
                            ) =>
                            {
                                if team != self.inner_state.team
                                {
                                    self.get_damage_and_add_force(
                                        damage as i32,
                                        force,
                                        physic_system,
                                        audio_system,
                                        ui_system,
                                        engine_handle,
                                    );
                                }
                            }

                            PlayerMessage::NewPeerConnected(peer_id) =>
                            {
                                engine_handle.send_command(
                                    Command {
                                        sender: self.id.unwrap(),
                                        command_type: CommandType::NetCommand(
                                            NetCommand::SendDirectNetMessageReliable(
                                                NetMessageToPlayer::RemoteCommand(
                                                    RemoteCommand::SpawnPlayersDollActor(
                                                        self.get_transform().to_serializable_transform(),
                                                        self.inner_state.collider.get_collider_radius(),
                                                        self.inner_state.is_alive,
                                                        self.inner_state.team
                                                    )
                                                ),
                                                peer_id,
                                            )
                                        )
                                    }
                                )
                            }

                            PlayerMessage::SetNewTeam(team) =>
                            {
                                self.inner_state.team = team;
                                self.inner_state.amount_of_move_w_bonuses_do_i_have = 0u32;

                                self.set_right_team_hud(ui_system);

                                engine_handle.send_command(
                                    Command {
                                        sender: self.get_id().expect("Player have not ActorID"),
                                        command_type: CommandType::RespawnPlayer(
                                            self.get_id().expect("Player have not ActorID")
                                        )
                                    }
                                );
                            }
                        }
                    },
                    SpecificActorMessage::SessionControllerMessage(message) =>
                    {
                        match message
                        {
                            SessionControllerMessage::NewSessionStarted(team) =>
                            {
                                self.inner_state.team = team;
                                self.inner_state.is_time_after_some_team_win = false;
                                self.inner_state.amount_of_move_w_bonuses_do_i_have = 0u32;

                                self.set_right_team_hud(ui_system);

                                engine_handle.send_command(
                                    Command {
                                        sender: self.get_id().expect("Player have not ActorID"),
                                        command_type: CommandType::RespawnPlayer(
                                            self.get_id().expect("Player have not ActorID")
                                        )
                                    }
                                );

                            }

                            SessionControllerMessage::JoinedToSession(
                                your_team, _, _, _, _, _,
                            ) =>
                            {
                                println!("Joined to game session");
                                
                                self.inner_state.team = your_team;
                                self.inner_state.is_time_after_some_team_win = false;
                                self.inner_state.amount_of_move_w_bonuses_do_i_have = 0u32;

                                self.set_right_team_hud(ui_system);

                                engine_handle.send_command(
                                    Command {
                                        sender: self.get_id().expect("Player have not ActorID"),
                                        command_type: CommandType::RespawnPlayer(
                                            self.get_id().expect("Player have not ActorID")
                                        )
                                    }
                                );
                            }

                            SessionControllerMessage::TeamWin(team) =>
                            {
                                self.inner_state.is_time_after_some_team_win = true;
                            }

                            _ => {}
                        }
                    }

                    SpecificActorMessage::FlagMessage(message) =>
                    {
                        match message
                        {
                            FlagMessage::GiveMeTargetPosition =>
                            {
                                match self.inner_state.team {
                                    Team::Red =>
                                    {
                                        let ui_elem = ui_system.get_mut_ui_element(&UIElementType::BlueFlagBacklight);
                                        *ui_elem.get_ui_data().get_is_visible_cloned_arc().lock().unwrap() = true;
                                    }
                                    Team::Blue =>
                                    {
                                        let ui_elem = ui_system.get_mut_ui_element(&UIElementType::RedFlagBacklight);
                                        *ui_elem.get_ui_data().get_is_visible_cloned_arc().lock().unwrap() = true;
                                    }
                                }

                                engine_handle.send_direct_message(
                                    from,
                                    Message {
                                        from: self.get_id().expect("Player have not ActorID"),
                                        message:                                     MessageType::SpecificActorMessage(
                                            SpecificActorMessage::FlagMessage(
                                                FlagMessage::SetTargetPosition(
                                                    self.get_transform().get_position() + self.flag_pivot_offset
                                                )
                                            )
                                        )
                                    }
                                );
                            }

                            FlagMessage::YouInteractingWithFlag(
                                team_that_owns_flag,
                                flag_status,
                            ) =>
                            {
                                if team_that_owns_flag == self.inner_state.team
                                {
                                    match flag_status {
                                        FlagStatus::Missed(_) =>
                                        {
                                            engine_handle.send_command(
                                                Command {
                                                    sender: self.get_id().expect("Player have not ActorID"),
                                                    command_type: CommandType::NetCommand(
                                                        NetCommand::SendMessageToServer(
                                                            NetMessageToServer::TryToReturnMyFlag(
                                                                time_system.get_server_time()
                                                            )
                                                        )
                                                    )
                                                }
                                            );   
                                        }

                                        FlagStatus::OnTheBase =>
                                        {
                                            engine_handle.send_command(
                                                Command {
                                                    sender: self.get_id().expect("Player have not ActorID"),
                                                    command_type: CommandType::NetCommand(
                                                        NetCommand::SendMessageToServer(
                                                            NetMessageToServer::TryToGetScore(
                                                                time_system.get_server_time()
                                                            )
                                                        )
                                                    )
                                                }
                                            ); 
                                        }

                                        FlagStatus::Captured(_) => {}
                                    }
                                }
                                else
                                {
                                    engine_handle.send_command(
                                        Command {
                                            sender: self.get_id().expect("Player have not ActorID"),
                                            command_type: CommandType::NetCommand(
                                                NetCommand::SendMessageToServer(
                                                    NetMessageToServer::TryToGetOpponentsFlag(
                                                        time_system.get_server_time()
                                                    )
                                                )
                                            )
                                        }
                                    );
                                }
                            }

                            _ => {}
                        }
                    }

                    SpecificActorMessage::MoveWBonusSpotMessage(message) =>
                    {
                        match message
                        {
                            MoveWBonusSpotMessage::SetBonusStatus(_, status) =>
                            {
                                match status
                                {
                                    BonusSpotStatus::BonusCollected(collected_by) =>
                                    {
                                        if collected_by == self.get_id().expect("Player have not ActorID")
                                        {
                                            if self.inner_state.amount_of_move_w_bonuses_do_i_have <
                                                MAX_MOVE_W_BONUSES_I_CAN_HAVE
                                            {
                                                audio_system.spawn_non_spatial_sound(
                                                    Sound::PickUpBonus,
                                                    1.0,
                                                    1.0,
                                                    false,
                                                    true,
                                                    Status::Playing,
                                                );

                                                self.inner_state.amount_of_move_w_bonuses_do_i_have += 1;
                                            }                                          
                                        }
                                    }
                                    _ => {}
                                }
                            }

                            MoveWBonusSpotMessage::YouTryingToGetMoveWBonus(index) =>
                            {
                                if self.inner_state.amount_of_move_w_bonuses_do_i_have < MAX_MOVE_W_BONUSES_I_CAN_HAVE
                                {
                                    engine_handle.send_command(
                                        Command {
                                            sender: self.get_id().expect("Player have not ActorID"),
                                            command_type: CommandType::NetCommand(
                                                NetCommand::SendMessageToServer(
                                                    NetMessageToServer::TryToGetMoveWBonus(
                                                        time_system.get_server_time(),
                                                        index
                                                    )
                                                )
                                            )
                                        }
                                    );
                                }
                            }
                        }
                    }

                    SpecificActorMessage::PlayersDollMessage(message) =>
                    {
                        match message
                        {
                            PlayersDollMessage::YouHitMe(_) =>
                            {
                                self.inner_state.show_crosshaier_hit_mark_timer = SHOW_CROSSHAIER_HIT_MARK_TIME;
                            }

                            _ => {}
                        }
                    }

                    _ => {}
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
        if self.inner_state.is_enable
        {
            let collider_container = PhysicalElement {
                id: self.get_id().expect("Actor have not ActorID"),
                transform: &mut self.inner_state.transform,
                kinematic_collider: Some((&mut self.inner_state.collider, None)),
                static_colliders: None,
                dynamic_colliders: Some((&mut self.inner_state.collider_for_others, self.inner_state.team)),
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
        effects_system: &mut EffectsSystem,
        delta: f32
    ) {

        let ui_elem = ui_system.get_mut_ui_element(&UIElementType::RedFlagBacklight);
        *ui_elem.get_ui_data().get_is_visible_cloned_arc().lock().unwrap() = false;

        let ui_elem = ui_system.get_mut_ui_element(&UIElementType::BlueFlagBacklight);
        *ui_elem.get_ui_data().get_is_visible_cloned_arc().lock().unwrap() = false;

        let my_id = self.id.expect("Player does not have id");

        let mut input = match &self.master {
            InputMaster::LocalMaster(master) => {
                master.current_input.clone()
            }
            InputMaster::RemoteMaster(master) => {
               master.current_input.clone()
            }   
        };

        self.inner_state.process_crosshair_size_and_ui(ui_system, delta);
    
        self.screen_effects.getting_damage_screen_effect -= delta * GETTING_DAMAGE_EFFECT_COEF_DECREASE_SPEED;

        self.screen_effects.getting_damage_screen_effect = self.screen_effects.getting_damage_screen_effect.clamp(0.0, 1.0);

        self.make_hud_transparency_as_death_screen_effect(ui_system);

        
        let mut player_doll_input_state = PlayerDollInputState {
            move_forward: false,
            move_backward: false,
            move_right: false,
            move_left: false,
            will_jump: false,
            // player_moving_state: self.inner_state.player_moving_state.clone(),
        };


        if self.inner_state.is_alive {

            if input.hold_player_rotation.is_action_just_pressed()
            {
                self.holding_player_rotation_along_w = !self.holding_player_rotation_along_w;
            }

            self.screen_effects.death_screen_effect -= delta*DEATH_EFFECT_COEF_DECREASE_SPEED;
            self.screen_effects.death_screen_effect = self.screen_effects.death_screen_effect.clamp(0.0, 1.0);

            process_player_rotation(
                &input,
                &self.player_settings,
                &mut self.saved_angle_of_rotation,
                &mut self.inner_state,
                delta
            );

            process_w_scanner_ui(
                ui_system,
                &self.inner_state,
                &self.saved_angle_of_rotation,
            );

            procces_w_rotation_sound(
                audio_system,
                self.rotating_around_w_sound_handle,
                &self.saved_angle_of_rotation,
                &mut self.rotating_around_w_sound_pitch,
                &mut self.rotating_around_w_sound_gain,
                self.inner_state.last_frame_zw_rotation,
                delta,
            );

            procces_w_shift_sound(
                audio_system,
                self.shifting_along_w_sound_handle,
                &self.inner_state,
            );

            self.process_active_devices_input(
                &input,
                physic_system,
                audio_system,
                ui_system,
                engine_handle,
                delta,
            );

            self.process_switch_active_hand_slot_input(
                &input,
                physic_system,
                audio_system,
                ui_system,
                engine_handle,
            );

            process_player_movement_input(
                &input,
                &mut player_doll_input_state,
                &mut self.inner_state,
                &self.player_settings,
                delta,
            );

            process_player_y_jump_input(
                &input,
                &mut player_doll_input_state,
                &mut self.inner_state,
                &self.player_settings,
            );

            process_player_w_jump_input(
                &input,
                &mut player_doll_input_state,
                &mut self.inner_state,
                &self.player_settings,
            );


    
            if input.w_scanner.is_action_just_pressed() {
                if !self.w_scanner_enable {
                    if self.w_scanner_reloading_time >= self.player_settings.scanner_reloading_time {
                        
                        self.w_scanner_enable = true;

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

            self.inner_state.player_previous_w_position = self.inner_state.get_position().w;


            self.get_effected_by_base(delta, physic_system, audio_system, ui_system, engine_handle);

            // ---------------------------------------------------
            // temp!
            // y death plane
            if self.inner_state.get_position().y < Y_DEATH_PLANE_LEVEL {
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
            self.inner_state.get_position(),
            self.get_transform().get_direction_for_audio_system()
        );

        let remote_velocity = {
            let mut v = self.inner_state.collider.current_velocity;

            for force in self.inner_state.collider.forces.iter() {
                v += *force;
            }

            v.to_array()
        };

        // player_doll_input_state.player_moving_state = self.inner_state.player_moving_state.clone();

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


pub fn process_player_rotation(
    input: &ActionsFrameState,
    player_settings: &PlayerSettings,
    saved_angle_of_rotation: &mut Vec4,
    inner_state: &mut PlayerInnerState,
    delta: f32,
)
{
    let mut xz = saved_angle_of_rotation.x;
    let mut yz = saved_angle_of_rotation.y;
    let mut zw = saved_angle_of_rotation.w;

    inner_state.last_frame_zw_rotation = zw;

    if input.second_mouse.is_action_pressed() {
        zw = (input.mouse_axis.y * player_settings.mouse_sensivity + zw).clamp(-PI/2.0, PI/2.0);
        xz = input.mouse_axis.x * player_settings.mouse_sensivity + xz;
        
    } else {
        // restore player's rotation along W
        zw *= 1.0 - delta * 2.8;
        if zw.abs() < 0.0001 {
            zw = 0.0;
        }

        xz = input.mouse_axis.x * player_settings.mouse_sensivity + xz;
        yz = (input.mouse_axis.y * player_settings.mouse_sensivity + yz).clamp(-PI/2.0, PI/2.0);
    }

    let zy_rotation = Mat4::from_rotation_x(-yz);

    let zx_rotation = Mat4::from_rotation_y(-xz);

    let zw_rotation = Mat4::from_cols_slice(&[
        1.0,    0.0,    0.0,        0.0,
        0.0,    1.0,    0.0,        0.0,
        0.0,    0.0,    (zw).cos(),   (zw).sin(),
        0.0,    0.0,    -(zw).sin(),   (zw).cos()
    ]);

    saved_angle_of_rotation.x = xz;
    saved_angle_of_rotation.y = yz;
    saved_angle_of_rotation.w = zw;

    inner_state.zw_rotation = zw_rotation;
    inner_state.zy_rotation = zy_rotation;
    inner_state.zx_rotation = zx_rotation;
    inner_state.set_rotation_matrix( zw_rotation * zy_rotation * zx_rotation);
}


pub fn process_w_scanner_ui(
    ui_system: &mut UISystem,
    inner_state: &PlayerInnerState,
    saved_angle_of_rotation: &Vec4,
)
{
    let xz = saved_angle_of_rotation.x;
    let zw = saved_angle_of_rotation.w;

    let zw_arrow = match inner_state.team {
        Team::Red => ui_system.get_mut_ui_element(&UIElementType::ZWScannerArrowRed),
        Team::Blue => ui_system.get_mut_ui_element(&UIElementType::ZWScannerArrowBlue),
    };

    if let UIElement::Image(arrow) = zw_arrow {
        arrow.set_rotation_around_rect_center(-zw+PI/2.0);
    } else {
        panic!("UI Element ZWScannerArrow is not UIImage")
    }

    let zx_arrow = match inner_state.team {
        Team::Red => ui_system.get_mut_ui_element(&UIElementType::ZXScannerArrowRed),   
        Team::Blue => ui_system.get_mut_ui_element(&UIElementType::ZXScannerArrowBlue),   
    };

    if let UIElement::Image(arrow) = zx_arrow {
        arrow.set_rotation_around_rect_center(xz-PI/2.0);
    } else {
        panic!("UI Element ZXScannerArrow is not UIImage")
    }

    let h_pointer = match inner_state.team {
        Team::Red => ui_system.get_mut_ui_element(&UIElementType::ScannerHPointerRed),
        Team::Blue => ui_system.get_mut_ui_element(&UIElementType::ScannerHPointerBlue),
    };

    if let UIElement::Image(h_pointer) = h_pointer {
        let h = {
            (((inner_state.get_position().w + 20.0) / 40.0) - 0.51)
                .clamp(-0.7, 0.8)
        };
        
        h_pointer.set_position(Vec2::new(0.002, h));
    } else {
        panic!("UI Element ScannerHPointer is not UIImage")
    }
}


pub fn procces_w_rotation_sound(
    audio_system: &mut AudioSystem,
    rotating_around_w_sound_handle: Handle<SoundSource>,
    saved_angle_of_rotation: &Vec4,
    rotating_around_w_sound_pitch: &mut f64,
    rotating_around_w_sound_gain: &mut f32,
    last_frame_zw_rotation: f32,
    delta: f32,
)
{
    let zw = saved_angle_of_rotation.w;

    let base_pitch = {
        0.8.lerp(
            1.5,
            (std::f64::consts::PI/2.0 + zw as f64) / std::f64::consts::PI
        )
    };

    let addition_pitch = {
        *rotating_around_w_sound_pitch * (1.0-delta as f64*22.0) +
        ((last_frame_zw_rotation - zw) as f64).abs() * 2.0
    };

    *rotating_around_w_sound_pitch = addition_pitch;

    let gain = {
        *rotating_around_w_sound_gain * (1.0-(delta*42.0)) +
        (last_frame_zw_rotation - zw).abs() * 10.0
    };

    *rotating_around_w_sound_gain = gain;

    audio_system.sound_set_pitch_and_gain(
        rotating_around_w_sound_handle,
        base_pitch,//D + addition_pitch,
        gain
    );
}


pub fn procces_w_shift_sound(
    audio_system: &mut AudioSystem,
    shifting_along_w_sound_handle: Handle<SoundSource>,
    inner_state: &PlayerInnerState,
)
{
    let shift_pitch = {
        1.0.lerp(
            1.5,
            0.5 +
            (
                (inner_state.get_position().w - inner_state.player_previous_w_position) *
                10.0
            ).clamp(-0.5, 0.5)
        )
    };

    let shift_gain = {
        0.0.lerp(
            1.0,
            (
                (inner_state.get_position().w - inner_state.player_previous_w_position).abs() *
                20.0
            ).clamp(0.0, 1.0)
        )
    };

    audio_system.sound_set_pitch_and_gain(
        shifting_along_w_sound_handle,
        shift_pitch as f64,//D + addition_pitch,
        shift_gain
    );
}


pub fn process_player_movement_input(
    input: &ActionsFrameState,
    player_doll_input_state: &mut PlayerDollInputState,
    inner_state: &mut PlayerInnerState,
    player_settings: &PlayerSettings,
    delta: f32,
)
{
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

    movement_vec = inner_state.get_rotation_matrix().inverse() * movement_vec;
    movement_vec.y = 0.0;
    movement_vec.w = 0.0;

    match movement_vec.try_normalize()
    {
        Some(vec) => movement_vec = vec,
        None => movement_vec = Vec4::ZERO,
    }

    // add w gravity
    inner_state.collider.add_force(Vec4::NEG_W * player_settings.gravity_w_speed * delta);
    
    // add y gravity
    inner_state.collider.add_force(Vec4::NEG_Y * player_settings.gravity_y_speed * delta);

    if inner_state.collider.is_on_y_ground {
        inner_state.collider.set_wish_direction(
            movement_vec,
            1.0
        );
    } else {
        inner_state.collider.set_wish_direction(
            movement_vec,
            player_settings.air_speed_mult
        );
    }

    inner_state.collider.set_friction_on_air(
        inner_state.friction_on_air
    );
}


pub fn process_player_y_jump_input(
    input: &ActionsFrameState,
    player_doll_input_state: &mut PlayerDollInputState,
    inner_state: &mut PlayerInnerState,
    player_settings: &PlayerSettings,
)
{
    if input.jump.is_action_just_pressed() {

        player_doll_input_state.will_jump = true;

        inner_state.jumped_to_y_on_current_action = false;

        if inner_state.collider.is_on_y_ground {
            inner_state.collider.add_force(Vec4::Y * player_settings.jump_y_speed);

            inner_state.jumped_to_y_on_current_action = true;
            
            player_doll_input_state.will_jump = false;
        }
    }

    if input.jump.is_action_pressed() {
        if !inner_state.jumped_to_y_on_current_action {
            if inner_state.collider.is_on_y_ground {
                inner_state.collider.add_force(Vec4::Y * player_settings.jump_y_speed);

                inner_state.jumped_to_y_on_current_action = true;
                
                player_doll_input_state.will_jump = false;
            }
        }
    } else {
        player_doll_input_state.will_jump = false;
    }
}


pub fn process_player_w_jump_input(
    input: &ActionsFrameState,
    player_doll_input_state: &mut PlayerDollInputState,
    inner_state: &mut PlayerInnerState,
    player_settings: &PlayerSettings,
)
{
    if input.move_w_up.is_action_just_pressed() {
        inner_state.collider.add_force(Vec4::W * player_settings.jump_w_speed);
    }
}


impl Player {

    pub fn new(
        master: InputMaster,
        player_settings: PlayerSettings,
        audio_system: &mut AudioSystem,
        w_levels_of_map: Vec<f32>
    ) -> Self {

        assert!(w_levels_of_map.len() > 1);

        let blue_map_w_level = w_levels_of_map[0];

        let red_map_w_level = *w_levels_of_map.last().unwrap();
        
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

        let player_radius = player_settings.collider_radius;
        
        Player {
            id: None,

            inner_state: PlayerInnerState::new(
                Transform::new(),
                &player_settings,
                false,
                false,
                blue_map_w_level,
                red_map_w_level,
            ),
            active_hands_slot: ActiveHandsSlot::Zero,

            hands_slot_0: Box::new(HoleGun::new(
                player_settings.energy_gun_hole_size_mult, 
                player_settings.energy_gun_add_force_mult, 
                player_settings.energy_gun_damage_mult, 
                player_settings.energy_gun_restoring_speed,
                Vec4::new(
                    1.0,
                    -0.42,
                    -1.0,
                    0.0
                ),
            )),
            hands_slot_1: Some(Box::new(MachineGun::new(
                player_settings.machinegun_damage,
                player_settings.machinegun_add_force, 
                player_settings.machinegun_heat_add_on_shot, 
                player_settings.machinegun_cooling_speed,
                Vec4::new(
                    1.0,
                    -0.42,
                    -1.0,
                    0.0
                ),
            ))),
            hands_slot_2: None,
            hands_slot_3: None,

            is_gravity_y_enabled: true,
            is_gravity_w_enabled: true,

            devices: [None, None, None, None],
            
            player_settings,

            master,

            // explore_w_position: 0.0,
            // explore_w_coefficient: 0.0,

            no_collider_veclocity: Vec4::ZERO,

            saved_angle_of_rotation: Vec4::ZERO,

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

            jumped_to_y_on_current_action: false,
            jumped_to_w_on_current_action: false,
            jumped_to_wy_on_current_action: false,

            // need_to_rotate_w_to_zero: true,
            // time_from_previos_second_mouse_click: 0.0,

            // w_levels_of_map,
            // current_w_level: 0,

            // fisrt_move_w_bonus_transparency_level: HAVE_NOT_MOVE_W_BONUS_TRANSPARENCY_LEVEL,
            // second_move_w_bonus_transparency_level: HAVE_NOT_MOVE_W_BONUS_TRANSPARENCY_LEVEL,

            flag_pivot_offset: Vec4::new(0.0, player_radius * 2.0, 0.0, 0.0),

            base_effect_tick_timer: 0.0,

            holding_player_rotation_along_w: false,
        }
    }

    // pub fn process_bonuses_ui(&mut self)
    // {
    //     match self.inner_state.amount_of_move_w_bonuses_do_i_have
    //     {
    //         0 =>
    //         {
    //             self.fisrt_move_w_bonus_transparency_level = HAVE_NOT_MOVE_W_BONUS_TRANSPARENCY_LEVEL;
    //             self.second_move_w_bonus_transparency_level = HAVE_NOT_MOVE_W_BONUS_TRANSPARENCY_LEVEL;
    //         }

    //         1 =>
    //         {
    //             self.fisrt_move_w_bonus_transparency_level = 1.0;
    //             self.second_move_w_bonus_transparency_level = HAVE_NOT_MOVE_W_BONUS_TRANSPARENCY_LEVEL;
    //         }

    //         2 =>
    //         {
    //             self.fisrt_move_w_bonus_transparency_level = 1.0;
    //             self.second_move_w_bonus_transparency_level = 1.0;
    //         }

    //         _ => panic!("Player have move w bonuses > 2")
    //     }
    // }


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

    fn telefraged(&mut self, audio_system: &mut AudioSystem, engine_handle: &mut EngineHandle) {
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

        // let hud_elem = ui.get_mut_ui_element(&UIElementType::MoveWBonusMarkFirst);
        // hud_elem.get_ui_data_mut().rect.transparency =
        //     self.fisrt_move_w_bonus_transparency_level * a;
        
        // let hud_elem = ui.get_mut_ui_element(&UIElementType::MoveWBonusMarkSecond);
        // hud_elem.get_ui_data_mut().rect.transparency =
        //     self.second_move_w_bonus_transparency_level * a;


        let hud_elem = ui.get_mut_ui_element(&UIElementType::Crosshair);
        hud_elem.get_ui_data_mut().rect.transparency = a;

        let hud_elem = ui.get_mut_ui_element(&UIElementType::MachinegunImage);
        hud_elem.get_ui_data_mut().rect.transparency = a;

        let hud_elem = ui.get_mut_ui_element(&UIElementType::EnergyGunImage);
        hud_elem.get_ui_data_mut().rect.transparency = a;

        let hud_elem = ui.get_mut_ui_element(&UIElementType::CrosshairHitMark);
        hud_elem.get_ui_data_mut().rect.transparency = a;

        let hud_elem = ui.get_mut_ui_element(&UIElementType::ScoreBar);
        hud_elem.get_ui_data_mut().rect.transparency = a;

        let hud_elem = ui.get_mut_ui_element(&UIElementType::RedFlagMark);
        hud_elem.get_ui_data_mut().rect.transparency = a;

        let hud_elem = ui.get_mut_ui_element(&UIElementType::FirstScoreMarkRed);
        hud_elem.get_ui_data_mut().rect.transparency = a;

        let hud_elem = ui.get_mut_ui_element(&UIElementType::SecondScoreMarkRed);
        hud_elem.get_ui_data_mut().rect.transparency = a;

        let hud_elem = ui.get_mut_ui_element(&UIElementType::ThirdScoreMarkRed);
        hud_elem.get_ui_data_mut().rect.transparency = a;

        let hud_elem = ui.get_mut_ui_element(&UIElementType::FinalScoreMarkRed);
        hud_elem.get_ui_data_mut().rect.transparency = a;

        let hud_elem = ui.get_mut_ui_element(&UIElementType::BlueFlagMark);
        hud_elem.get_ui_data_mut().rect.transparency = a;

        let hud_elem = ui.get_mut_ui_element(&UIElementType::FirstScoreMarkBlue);
        hud_elem.get_ui_data_mut().rect.transparency = a;

        let hud_elem = ui.get_mut_ui_element(&UIElementType::SecondScoreMarkBlue);
        hud_elem.get_ui_data_mut().rect.transparency = a;

        let hud_elem = ui.get_mut_ui_element(&UIElementType::ThirdScoreMarkBlue);
        hud_elem.get_ui_data_mut().rect.transparency = a;

        let hud_elem = ui.get_mut_ui_element(&UIElementType::FinalScoreMarkBlue);
        hud_elem.get_ui_data_mut().rect.transparency = a;
        
        match self.inner_state.team
        {
            Team::Red =>
            {
                let hud_elem = ui.get_mut_ui_element(&UIElementType::ScannerRed);
                hud_elem.get_ui_data_mut().rect.transparency = a;

                // let hud_elem = ui.get_mut_ui_element(&UIElementType::ScannerRedW);
                // hud_elem.get_ui_data_mut().rect.transparency = a;

                let hud_elem = ui.get_mut_ui_element(&UIElementType::ScannerHPointerRed);
                hud_elem.get_ui_data_mut().rect.transparency = a;

                let hud_elem = ui.get_mut_ui_element(&UIElementType::ZWScannerArrowRed);
                hud_elem.get_ui_data_mut().rect.transparency = a;

                let hud_elem = ui.get_mut_ui_element(&UIElementType::ZXScannerArrowRed);
                hud_elem.get_ui_data_mut().rect.transparency = a;

                let hud_elem = ui.get_mut_ui_element(&UIElementType::HeathBarRed);
                hud_elem.get_ui_data_mut().rect.transparency = a;

                let hud_elem = ui.get_mut_ui_element(&UIElementType::EnergyGunBarRed);
                hud_elem.get_ui_data_mut().rect.transparency = a;
                
                let hud_elem = ui.get_mut_ui_element(&UIElementType::MachinegunBarRed);
                hud_elem.get_ui_data_mut().rect.transparency = a;

                let hud_elem = ui.get_mut_ui_element(&UIElementType::LeftScannerDsiplayRed);
                hud_elem.get_ui_data_mut().rect.transparency = a;

                let hud_elem = ui.get_mut_ui_element(&UIElementType::RightScannerDsiplayRed);
                hud_elem.get_ui_data_mut().rect.transparency = a;
            }

            Team::Blue =>
            {
                let hud_elem = ui.get_mut_ui_element(&UIElementType::ScannerBlue);
                hud_elem.get_ui_data_mut().rect.transparency = a;

                // let hud_elem = ui.get_mut_ui_element(&UIElementType::ScannerBlueW);
                // hud_elem.get_ui_data_mut().rect.transparency = a;

                let hud_elem = ui.get_mut_ui_element(&UIElementType::ScannerHPointerBlue);
                hud_elem.get_ui_data_mut().rect.transparency = a;

                let hud_elem = ui.get_mut_ui_element(&UIElementType::ZWScannerArrowBlue);
                hud_elem.get_ui_data_mut().rect.transparency = a;

                let hud_elem = ui.get_mut_ui_element(&UIElementType::ZXScannerArrowBlue);
                hud_elem.get_ui_data_mut().rect.transparency = a;

                let hud_elem = ui.get_mut_ui_element(&UIElementType::HeathBarBlue);
                hud_elem.get_ui_data_mut().rect.transparency = a;

                let hud_elem = ui.get_mut_ui_element(&UIElementType::EnergyGunBarBlue);
                hud_elem.get_ui_data_mut().rect.transparency = a;
                
                let hud_elem = ui.get_mut_ui_element(&UIElementType::MachinegunBarBlue);
                hud_elem.get_ui_data_mut().rect.transparency = a;

                let hud_elem = ui.get_mut_ui_element(&UIElementType::LeftScannerDsiplayBlue);
                hud_elem.get_ui_data_mut().rect.transparency = a;

                let hud_elem = ui.get_mut_ui_element(&UIElementType::RightScannerDsiplayBlue);
                hud_elem.get_ui_data_mut().rect.transparency = a;
            }
        }
    }

    fn set_right_team_hud(&self, ui: &mut UISystem)
    {
        let hud_elem = ui.get_mut_ui_element(&UIElementType::Crosshair);
        *hud_elem.get_ui_data_mut().get_is_visible_cloned_arc().lock().unwrap() = true;

        let hud_elem = ui.get_mut_ui_element(&UIElementType::ScoreBar);
        *hud_elem.get_ui_data_mut().get_is_visible_cloned_arc().lock().unwrap() = true;

        // let hud_elem = ui.get_mut_ui_element(&UIElementType::MoveWBonusMarkFirst);
        // *hud_elem.get_ui_data_mut().get_is_visible_cloned_arc().lock().unwrap() = true;

        // let hud_elem = ui.get_mut_ui_element(&UIElementType::MoveWBonusMarkSecond);
        // *hud_elem.get_ui_data_mut().get_is_visible_cloned_arc().lock().unwrap() = true;


        match self.inner_state.team
        {
            Team::Red =>
            {
                let hud_elem = ui.get_mut_ui_element(&UIElementType::ScannerRed);
                *hud_elem.get_ui_data_mut().get_is_visible_cloned_arc().lock().unwrap() = true;

                let hud_elem = ui.get_mut_ui_element(&UIElementType::ScannerHPointerRed);
                *hud_elem.get_ui_data_mut().get_is_visible_cloned_arc().lock().unwrap() = true;

                let hud_elem = ui.get_mut_ui_element(&UIElementType::ZWScannerArrowRed);
                *hud_elem.get_ui_data_mut().get_is_visible_cloned_arc().lock().unwrap() = true;

                let hud_elem = ui.get_mut_ui_element(&UIElementType::ZXScannerArrowRed);
                *hud_elem.get_ui_data_mut().get_is_visible_cloned_arc().lock().unwrap() = true;

                let hud_elem = ui.get_mut_ui_element(&UIElementType::HeathBarRed);
                *hud_elem.get_ui_data_mut().get_is_visible_cloned_arc().lock().unwrap() = true;

                let hud_elem = ui.get_mut_ui_element(&UIElementType::LeftScannerDsiplayRed);
                *hud_elem.get_ui_data_mut().get_is_visible_cloned_arc().lock().unwrap() = true;

                let hud_elem = ui.get_mut_ui_element(&UIElementType::RightScannerDsiplayRed);
                *hud_elem.get_ui_data_mut().get_is_visible_cloned_arc().lock().unwrap() = true;


                let hud_elem = ui.get_mut_ui_element(&UIElementType::ScannerBlue);
                *hud_elem.get_ui_data_mut().get_is_visible_cloned_arc().lock().unwrap() = false;

                let hud_elem = ui.get_mut_ui_element(&UIElementType::ScannerHPointerBlue);
                *hud_elem.get_ui_data_mut().get_is_visible_cloned_arc().lock().unwrap() = false;

                let hud_elem = ui.get_mut_ui_element(&UIElementType::ZWScannerArrowBlue);
                *hud_elem.get_ui_data_mut().get_is_visible_cloned_arc().lock().unwrap() = false;

                let hud_elem = ui.get_mut_ui_element(&UIElementType::ZXScannerArrowBlue);
                *hud_elem.get_ui_data_mut().get_is_visible_cloned_arc().lock().unwrap() = false;

                let hud_elem = ui.get_mut_ui_element(&UIElementType::HeathBarBlue);
                *hud_elem.get_ui_data_mut().get_is_visible_cloned_arc().lock().unwrap() = false;

                let hud_elem = ui.get_mut_ui_element(&UIElementType::LeftScannerDsiplayBlue);
                *hud_elem.get_ui_data_mut().get_is_visible_cloned_arc().lock().unwrap() = false;

                let hud_elem = ui.get_mut_ui_element(&UIElementType::RightScannerDsiplayBlue);
                *hud_elem.get_ui_data_mut().get_is_visible_cloned_arc().lock().unwrap() = false;
            }

            Team::Blue =>
            {
                let hud_elem = ui.get_mut_ui_element(&UIElementType::ScannerRed);
                *hud_elem.get_ui_data_mut().get_is_visible_cloned_arc().lock().unwrap() = false;

                let hud_elem = ui.get_mut_ui_element(&UIElementType::ScannerHPointerRed);
                *hud_elem.get_ui_data_mut().get_is_visible_cloned_arc().lock().unwrap() = false;

                let hud_elem = ui.get_mut_ui_element(&UIElementType::ZWScannerArrowRed);
                *hud_elem.get_ui_data_mut().get_is_visible_cloned_arc().lock().unwrap() = false;

                let hud_elem = ui.get_mut_ui_element(&UIElementType::ZXScannerArrowRed);
                *hud_elem.get_ui_data_mut().get_is_visible_cloned_arc().lock().unwrap() = false;

                let hud_elem = ui.get_mut_ui_element(&UIElementType::HeathBarRed);
                *hud_elem.get_ui_data_mut().get_is_visible_cloned_arc().lock().unwrap() = false;

                let hud_elem = ui.get_mut_ui_element(&UIElementType::LeftScannerDsiplayRed);
                *hud_elem.get_ui_data_mut().get_is_visible_cloned_arc().lock().unwrap() = false;

                let hud_elem = ui.get_mut_ui_element(&UIElementType::RightScannerDsiplayRed);
                *hud_elem.get_ui_data_mut().get_is_visible_cloned_arc().lock().unwrap() = false;


                let hud_elem = ui.get_mut_ui_element(&UIElementType::ScannerBlue);
                *hud_elem.get_ui_data_mut().get_is_visible_cloned_arc().lock().unwrap() = true;

                let hud_elem = ui.get_mut_ui_element(&UIElementType::ScannerHPointerBlue);
                *hud_elem.get_ui_data_mut().get_is_visible_cloned_arc().lock().unwrap() = true;

                let hud_elem = ui.get_mut_ui_element(&UIElementType::ZWScannerArrowBlue);
                *hud_elem.get_ui_data_mut().get_is_visible_cloned_arc().lock().unwrap() = true;

                let hud_elem = ui.get_mut_ui_element(&UIElementType::ZXScannerArrowBlue);
                *hud_elem.get_ui_data_mut().get_is_visible_cloned_arc().lock().unwrap() = true;

                let hud_elem = ui.get_mut_ui_element(&UIElementType::HeathBarBlue);
                *hud_elem.get_ui_data_mut().get_is_visible_cloned_arc().lock().unwrap() = true;

                let hud_elem = ui.get_mut_ui_element(&UIElementType::LeftScannerDsiplayBlue);
                *hud_elem.get_ui_data_mut().get_is_visible_cloned_arc().lock().unwrap() = true;

                let hud_elem = ui.get_mut_ui_element(&UIElementType::RightScannerDsiplayBlue);
                *hud_elem.get_ui_data_mut().get_is_visible_cloned_arc().lock().unwrap() = true;
            }
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

        let in_space = {
            self.get_transform().get_position().y < Y_DEATH_PLANE_LEVEL+1.0
        };

        engine_handle.send_boardcast_message(
            Message {
                from: self.get_id().expect("Player have not ActorID"),
                message: MessageType::SpecificActorMessage(
                    SpecificActorMessage::FlagMessage(
                        FlagMessage::PlayerDied(in_space)
                    )
                )
            }
        );

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

        self.inner_state.hp -= damage as f32;
        self.inner_state.collider.add_force(force);

        let health_bar = match self.inner_state.team {
            Team::Red => ui_system.get_mut_ui_element(&UIElementType::HeathBarRed), 
            Team::Blue => ui_system.get_mut_ui_element(&UIElementType::HeathBarBlue), 
        };

        if let UIElement::ProgressBar(bar) = health_bar {
            let bar_value = {
                (self.inner_state.hp as f32 / PLAYER_MAX_HP as f32)
                    .clamp(0.0, 1.0)
            };

            bar.set_bar_value(bar_value)

        } else {
            panic!("Health Bar is not Progress Bar")
        }

        if self.inner_state.hp <= 0.0 {
            if damage as f32 >= PLAYER_MAX_HP {
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

    fn get_effected_by_base(
        &mut self,
        delta: f32,
        physic_system: &PhysicsSystem,
        audio_system: &mut AudioSystem,
        ui_system: &mut UISystem,
        engine_handle: &mut EngineHandle,
    ) {
        let base_coef = 
        {
            let w_pos = self.inner_state.get_position().w;

            let mut coef = f32::clamp(
                (w_pos - self.inner_state.blue_map_w_level) /
                (self.inner_state.red_map_w_level - self.inner_state.blue_map_w_level),
                    0.0,
                    1.0
            );

            if self.inner_state.team == Team::Blue
            {
                coef = 1.0 - coef;
            }

            coef = (coef * 2.0) - 1.0;

            coef.max(0.0)
        };

        self.inner_state.hp += BASE_EFFECT_HP_IMPACT_SPEED * delta * base_coef;

        if self.inner_state.hp > PLAYER_MAX_HP
        {
            self.inner_state.hp = PLAYER_MAX_HP;
        }

        let health_bar = match self.inner_state.team {
            Team::Red => ui_system.get_mut_ui_element(&UIElementType::HeathBarRed), 
            Team::Blue => ui_system.get_mut_ui_element(&UIElementType::HeathBarBlue), 
        };

        if let UIElement::ProgressBar(bar) = health_bar {
            let bar_value = {
                (self.inner_state.hp as f32 / PLAYER_MAX_HP as f32)
                    .clamp(0.0, 1.0)
            };

            bar.set_bar_value(bar_value)

        } else {
            panic!("Health Bar is not Progress Bar")
        }

        if self.inner_state.hp <= 0.0
        {
            self.die(true, engine_handle, physic_system, audio_system, ui_system);
        }
    }

    // pub fn set_current_w_level(&mut self, w_level: usize)
    // {
    //     self.current_w_level = w_level;
    // }

    // pub fn get_current_w_level(&self) -> usize
    // {
    //     self.current_w_level
    // }

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


    fn put_device_into_device_slot(
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


    fn process_switch_active_hand_slot_input(
        &mut self,
        input: &ActionsFrameState,
        physic_system: &PhysicsSystem,
        audio_system: &mut AudioSystem,
        ui_system: &mut UISystem,
        engine_handle: &mut EngineHandle,
    )
    {
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
    }

    fn process_active_devices_input(
        &mut self,
        input: &ActionsFrameState,
        physic_system: &PhysicsSystem,
        audio_system: &mut AudioSystem,
        ui_system: &mut UISystem,
        engine_handle: &mut EngineHandle,
        delta: f32,
    )
    {
        let my_id = self.get_id().expect("Player have not ActorID");
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
    }
}

impl ControlledActor for Player
{
    fn get_camera(&self) -> Camera {
        Camera {
            position: self.inner_state.get_eyes_position(),
            rotation_matrix: self.inner_state.get_rotation_matrix(),
            zw_rotation_matrix: self.inner_state.get_zw_rotation_matrix(),
            zx_rotation_matrix: self.inner_state.get_zx_rotation_matrix(),
            zy_rotation_matrix: self.inner_state.get_zy_rotation_matrix(),
        }
    }

    fn get_screen_effects(&self) -> &PlayerScreenEffects {
        &self.screen_effects
    }

    fn get_team(&self) -> Team {
        self.inner_state.team
    }

    fn get_input_master(&mut self) -> &mut InputMaster {
        &mut self.master
    }

    fn spawn(
        &mut self,
        spawns: &mut Vec<Spawn>,
        physics_system: &PhysicsSystem,
        ui_system: &mut UISystem,
        audio_system: &mut AudioSystem,
        engine_handle: &mut EngineHandle,
    ) {
        let mut rng = thread_rng();
        spawns.shuffle(&mut rng);

        let mut current_spawn = spawns.last().expect("spawns in respawn function has zero length");

        for spawn in spawns.iter()
        {
            let hits = physics_system.sphere_cast_on_dynamic_colliders(
                spawn.spawn_position,
                self.inner_state.get_collider_radius(),
                Some(self.get_id().expect("Player hasn't ActorID"))
            );
    
            for hit in &hits {
                if let Some(team) = hit.hited_actors_team
                {
                    if self.get_team() == team
                    {
                        continue;
                    }
                }
            }

            current_spawn = spawn;
            
            break;
        };

        let hits = physics_system.sphere_cast_on_dynamic_colliders(
            current_spawn.spawn_position,
            self.inner_state.get_collider_radius(),
            Some(self.get_id().expect("Player hasn't ActorID"))
        );

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

        self.holding_player_rotation_along_w = false;
        self.inner_state.is_alive = true;
        self.inner_state.is_enable = true;
        self.inner_state.hp = PLAYER_MAX_HP;
        self.inner_state.amount_of_move_w_bonuses_do_i_have = 0u32;
        // self.inner_state.player_moving_state =
        //     PlayerMovingState::MovingPerpendicularW(self.w_levels_of_map[current_spawn.w_level]);

        self.saved_angle_of_rotation = Vec4::ZERO;

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

        let health_bar = match self.inner_state.team {
            Team::Red => ui_system.get_mut_ui_element(&UIElementType::HeathBarRed), 
            Team::Blue => ui_system.get_mut_ui_element(&UIElementType::HeathBarBlue), 
        };

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

        self.inner_state.transform = Transform::from_position(current_spawn.spawn_position);

        // self.current_w_level = current_spawn.w_level;

        self.inner_state.player_previous_w_position = current_spawn.spawn_position.w;

        let player_doll_input_state = PlayerDollInputState {
            move_forward: false,
            move_backward: false,
            move_right: false,
            move_left: false,
            will_jump: false,
            // player_moving_state: self.inner_state.player_moving_state.clone(),
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
                                self.inner_state.team
                            )
                        )
                    )
                )
            }
        )
    }
}