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
        main_player::{player_input_master, player_settings, player_inner_state::PlayerInnerState, PlayerMessage, PlayerMovingState, PlayerScreenEffects},
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
        }, effects::EffectsSystem, engine_handle::{
            Command,
            CommandType,
            EngineHandle
        }, input::ActionsFrameState, physics::{
            colliders_container::PhysicalElement,
            dynamic_collider::PlayersDollCollider,
            kinematic_collider::{
                KinematicCollider,
                KinematicColliderMessage
            },
            PhysicsSystem
        }, render::{camera::Camera, VisualElement}, time::TimeSystem, ui::{
            self, RectSize, UIElement, UIElementType, UISystem
        }, world::{level::Spawn, static_object::{BeamVolumeArea, VolumeArea}}
    },
    transform::Transform,
};

use self::{
    player_input_master::InputMaster,
    player_settings::PlayerSettings,
};

use core::panic;
use std::{collections::btree_set::Difference, f32::consts::PI, usize};
use fyrox_core::pool::Handle;
use fyrox_sound::source::{SoundSource, Status};
use glam::{
    FloatExt, Mat2, Mat3, Mat4, Vec2, Vec3, Vec4
};

use super::{
    device::machinegun::MachineGun, flag::{FlagMessage, FlagStatus}, main_player::{self, ActiveHandsSlot, WScanner}, move_w_bonus::{BonusSpotStatus, MoveWBonusSpotMessage}, mover_w::MoverWMessage, players_death_explosion::PlayersDeathExplosion, players_doll::PlayersDollMessage, session_controller::{SessionControllerMessage, DEFAULT_TEAM}, ControlledActor, PhysicsMessages
};


pub struct PlayerFor2d3dExample {
    id: Option<ActorID>,

    inner_state: PlayerInnerState,

    active_hands_slot: ActiveHandsSlot, 

    hands_slot_0: Box<dyn Device>,
    hands_slot_1: Option<Box<dyn Device>>,
    hands_slot_2: Option<Box<dyn Device>>,
    hands_slot_3: Option<Box<dyn Device>>,

    devices: [Option<Box<dyn Device>>; 4],

    pub player_settings: PlayerSettings,

    pub master: InputMaster,

    screen_effects: PlayerScreenEffects,

    w_scanner: WScanner,

    camera3d_rotation: Mat4,
    camera3d_rotation_zy: Mat4,
    camera3d_rotation_zx: Mat4,
    camera3d_rotation_zw: Mat4,
    camera3d_offset: Vec4,

    pub show_3d_example_current_value: f32,
    show_3d_example_target_value: f32,
}
pub const Y_DEATH_PLANE_LEVEL: f32 = -20.0;

pub const PLAYER_MAX_HP: f32 = 100.0;

const MIN_TIME_BEFORE_RESPAWN: f32 = 1.5;
const MAX_TIME_BEFORE_RESPAWN: f32 = 5.0;

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

impl Actor for PlayerFor2d3dExample {

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
                                let my_id = self.get_id().expect("Player Have not ActorID");

                                main_player::die(
                                    &mut self.inner_state,
                                    &mut self.active_hands_slot,
                                    &mut self.hands_slot_0,
                                    &mut self.hands_slot_1,
                                    &mut self.hands_slot_2,
                                    &mut self.hands_slot_3,
                                    &mut self.w_scanner,
                                    &mut self.devices,
                                    my_id,
                                    &mut self.player_settings,
                                    physic_system,
                                    audio_system,
                                    ui_system,
                                    engine_handle,
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
                    SpecificActorMessage::PLayerMessage(message) =>
                    {
                        match message {
                            PlayerMessage::DataForProjection(
                                updated_projection_position,
                                updated_projection_radius
                            ) =>
                            {
                                main_player::update_player_projection(
                                    from,
                                    updated_projection_position,
                                    updated_projection_radius,
                                    &mut self.screen_effects.player_projections,
                                    &self.inner_state
                                )
                            }

                            PlayerMessage::GiveMeDataForProjection => {}

                            PlayerMessage::Telefrag =>
                            {
                                let my_id = self.get_id().expect("Player Have not ActorID");

                                main_player::die(
                                    &mut self.inner_state,
                                    &mut self.active_hands_slot,
                                    &mut self.hands_slot_0,
                                    &mut self.hands_slot_1,
                                    &mut self.hands_slot_2,
                                    &mut self.hands_slot_3,
                                    &mut self.w_scanner,
                                    &mut self.devices,
                                    my_id,
                                    &mut self.player_settings,
                                    physic_system,
                                    audio_system,
                                    ui_system,
                                    engine_handle,
                                );
                            }

                            PlayerMessage::DieImmediately =>
                            {
                                let my_id = self.get_id().expect("Player Have not ActorID");

                                main_player::die(
                                    &mut self.inner_state,
                                    &mut self.active_hands_slot,
                                    &mut self.hands_slot_0,
                                    &mut self.hands_slot_1,
                                    &mut self.hands_slot_2,
                                    &mut self.hands_slot_3,
                                    &mut self.w_scanner,
                                    &mut self.devices,
                                    my_id,
                                    &mut self.player_settings,
                                    physic_system,
                                    audio_system,
                                    ui_system,
                                    engine_handle,
                                );
                            }

                            PlayerMessage::DieSlowly =>
                            {
                                let my_id = self.get_id().expect("Player Have not ActorID");

                                main_player::die(
                                    &mut self.inner_state,
                                    &mut self.active_hands_slot,
                                    &mut self.hands_slot_0,
                                    &mut self.hands_slot_1,
                                    &mut self.hands_slot_2,
                                    &mut self.hands_slot_3,
                                    &mut self.w_scanner,
                                    &mut self.devices,
                                    my_id,
                                    &mut self.player_settings,
                                    physic_system,
                                    audio_system,
                                    ui_system,
                                    engine_handle,
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
                                    let my_id = self.get_id().expect("Player Have not ActorID");

                                    main_player::get_damage_and_add_force(
                                        damage as i32,
                                        force,
                                        &mut self.screen_effects,
                                        &mut self.inner_state,
                                        &mut self.active_hands_slot,
                                        &mut self.hands_slot_0,
                                        &mut self.hands_slot_1,
                                        &mut self.hands_slot_2,
                                        &mut self.hands_slot_3,
                                        &mut self.w_scanner,
                                        &mut self.devices,
                                        my_id,
                                        &mut self.player_settings,
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

                                main_player::set_right_team_hud(
                                    &self.inner_state,
                                    ui_system
                                );

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

                                main_player::set_right_team_hud(
                                    &self.inner_state,
                                    ui_system
                                );

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

                                main_player::set_right_team_hud(
                                    &self.inner_state,
                                    ui_system
                                );

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
                                                    self.get_transform().get_position() + self.inner_state.flag_pivot_offset
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
            let mut visual_elem: Option<VisualElement> = match self.active_hands_slot {
                ActiveHandsSlot::Zero => {
                    self.hands_slot_0.get_visual_element(self.get_transform())
                },
                ActiveHandsSlot::First => {
                    if let Some(device) = &self.hands_slot_1 {
                        device.get_visual_element(self.get_transform())
                    } else {
                        None
                    }
                },
                ActiveHandsSlot::Second => {
                    if let Some(device) = &self.hands_slot_2 {
                        device.get_visual_element(self.get_transform())
                    } else {
                        None
                    }
                },
                ActiveHandsSlot::Third => {
                    if let Some(device) = &self.hands_slot_3 {
                        device.get_visual_element(self.get_transform())
                    } else {
                        None
                    }
                }
            };

            if let Some(vl) = visual_elem.as_mut()
            {
                vl.player = Some((&self.inner_state.collider_for_others[0], self.inner_state.team));

                // vl.volume_areas = Some(&self.view_volume_beams);
            }
            else
            {
                visual_elem = Some(
                    VisualElement {
                        transform: self.get_transform(),
                        static_objects: None,
                        coloring_areas: None,
                        volume_areas: None,//Some(&self.view_volume_beams),
                        waves: None,
                        player: Some((&self.inner_state.collider_for_others[0], self.inner_state.team))
                    }
                )
            }

            return visual_elem;

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

        let input = match &self.master {
            InputMaster::LocalMaster(master) => {
                master.current_input.clone()
            }
            InputMaster::RemoteMaster(master) => {
               master.current_input.clone()
            }   
        };

        let mut player_doll_input_state = PlayerDollInputState {
            move_forward: false,
            move_backward: false,
            move_right: false,
            move_left: false,
            will_jump: false,
        };

        let my_id = self.get_id().expect("Player have not ActorID");

        self.process_show_3d_example(
            &input,
            delta,
        );

        if self.inner_state.is_alive {

            main_player::process_screen_effects_while_alive
            (
                &mut self.screen_effects,
                delta,
            );

            process_player_for_example_rotation(
                &input,
                &self.player_settings,
                &mut self.inner_state,
                delta
            );

            main_player::process_w_scanner_ui(
                ui_system,
                &self.inner_state,
            );

            main_player::procces_w_rotation_sound(
                audio_system,
                &mut self.inner_state,
                delta,
            );

            main_player::procces_w_shift_sound(
                audio_system,
                &mut self.inner_state,
            );

            main_player::process_active_devices_input
            (
                &self.active_hands_slot,
                &mut self.hands_slot_0,
                &mut self.hands_slot_1,
                &mut self.hands_slot_2,
                &mut self.hands_slot_3,
                &mut self.devices,
                &mut self.inner_state,
                &input,
                my_id,
                physic_system,
                audio_system,
                ui_system,
                engine_handle,
                delta,
            );

            main_player::process_switch_active_hand_slot_input
            (
                &mut self.active_hands_slot,
                &mut self.hands_slot_0,
                &mut self.hands_slot_1,
                &mut self.hands_slot_2,
                &mut self.hands_slot_3,
                &mut self.inner_state,
                &input,
                my_id,
                physic_system,
                audio_system,
                ui_system,
                engine_handle,
            );

            process_player_for_example_movement_input(
                &input,
                &mut player_doll_input_state,
                &mut self.inner_state,
                &self.player_settings,
                delta,
            );

            main_player::process_player_y_jump_input(
                &input,
                &mut player_doll_input_state,
                &mut self.inner_state,
                &self.player_settings,
            );

            main_player::process_player_second_jump_input(
                &input,
                &mut self.inner_state,
                &self.player_settings,
                Vec4::X,
            );

            main_player::process_w_scanner(
                &input,
                &self.inner_state,
                &self.player_settings,
                &mut self.screen_effects,
                &mut self.w_scanner,
                physic_system,
                my_id,
                delta,
            );

            main_player::process_player_projections(
                &mut self.screen_effects.player_projections,
                engine_handle,
                my_id,
                delta
            );
            
            
            main_player::get_effected_by_base(
                &mut self.inner_state,
                &self.active_hands_slot,
                &mut self.hands_slot_0,
                &mut self.hands_slot_1,
                &mut self.hands_slot_2,
                &mut self.hands_slot_3,
                &mut self.w_scanner,
                &mut self.devices,
                my_id,
                &self.player_settings,
                physic_system,
                audio_system,
                ui_system,
                engine_handle,
                delta,
            );

            main_player::check_if_touching_death_plane(
                &mut self.inner_state,
                &self.active_hands_slot,
                &mut self.hands_slot_0,
                &mut self.hands_slot_1,
                &mut self.hands_slot_2,
                &mut self.hands_slot_3,
                &mut self.w_scanner,
                &mut self.devices,
                my_id,
                &self.player_settings,
                physic_system,
                audio_system,
                ui_system,
                engine_handle,
            );

        } else {
            //while player is not alive
            main_player::update_after_death_timer(
                &mut self.inner_state,
                delta
            );

            main_player::process_screen_effects_while_dead
            (
                &mut self.screen_effects,
                delta,
            );

            main_player::process_devices_while_player_is_dead
            (
                &self.active_hands_slot,
                &mut self.hands_slot_0,
                &mut self.hands_slot_1,
                &mut self.hands_slot_2,
                &mut self.hands_slot_3,
                &mut self.devices,
                &mut self.inner_state,
                &input,
                my_id,
                physic_system,
                audio_system,
                ui_system,
                engine_handle,
                delta,
            );

            main_player::process_player_respawn(
                engine_handle,
                &self.player_settings,
                &input,
                &self.inner_state,
                my_id,
            );
        }

        self.inner_state.process_crosshair_size_and_ui(ui_system, delta);

        main_player::decrease_getting_damage_screen_effect
        (
            &mut self.screen_effects,
            delta,
        );

        main_player::make_hud_transparency_as_death_screen_effect(
            &self.screen_effects,
            &self.inner_state,
            ui_system
        );

        main_player::set_audio_listener_position
        (
            audio_system,
            &self.inner_state,
        );

        main_player::send_player_state_to_remote_player_doll
        (
            player_doll_input_state,
            &self.inner_state,
            my_id,
            time_system,
            engine_handle,
        );
    }
}


fn process_player_for_example_rotation(
    input: &ActionsFrameState,
    player_settings: &PlayerSettings,
    inner_state: &mut PlayerInnerState,
    delta: f32,
)
{
    let mut xz = inner_state.saved_angle_of_rotation.x;
    let mut yz = inner_state.saved_angle_of_rotation.y;
    let mut zw = inner_state.saved_angle_of_rotation.w;

    inner_state.last_frame_zw_rotation = zw;

    if input.second_mouse.is_action_pressed() {
        xz = input.mouse_axis.x * player_settings.mouse_sensivity + xz;
        yz = (input.mouse_axis.y * player_settings.mouse_sensivity + yz).clamp(-PI/2.0, PI/2.0);
    } else {
        xz *= 1.0 - delta * 2.8;
        if xz.abs() < 0.0001 {
            xz = 0.0;
        }
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

    inner_state.saved_angle_of_rotation.x = xz;
    inner_state.saved_angle_of_rotation.y = yz;
    inner_state.saved_angle_of_rotation.w = zw;

    inner_state.zw_rotation = zw_rotation;
    inner_state.zy_rotation = zy_rotation;
    inner_state.zx_rotation = zx_rotation;
    inner_state.set_rotation_matrix( zw_rotation * zy_rotation * zx_rotation);
}


fn process_player_for_example_movement_input(
    input: &ActionsFrameState,
    player_doll_input_state: &mut PlayerDollInputState,
    inner_state: &mut PlayerInnerState,
    player_settings: &PlayerSettings,
    delta: f32,
)
{
    let mut movement_vec = Vec4::ZERO;

    if input.move_right.is_action_pressed() {
        movement_vec += Vec4::NEG_Z;
        player_doll_input_state.move_forward = true;

    }

    if input.move_left.is_action_pressed() {
        movement_vec += Vec4::Z;
        player_doll_input_state.move_backward = true;
    }

    if let Some(vec) = movement_vec.try_normalize() {
        movement_vec = vec;
    }

    movement_vec.y = 0.0;
    movement_vec.w = 0.0;

    match movement_vec.try_normalize()
    {
        Some(vec) => movement_vec = vec,
        None => movement_vec = Vec4::ZERO,
    }

    // lock player on w axis 
    let w_dif = 0.0 - inner_state.get_position().w;
    inner_state.collider.current_velocity.w = (w_dif*1.5).clamp(
        -player_settings.gravity_w_speed*25.0,
        player_settings.gravity_w_speed*25.0
    );

    inner_state.collider.add_force(Vec4::NEG_X * player_settings.gravity_w_speed * delta);

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


impl PlayerFor2d3dExample {

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
            player_projections: Vec::with_capacity(10),
        };

        let w_scanner = WScanner::new(&player_settings);

        let camera3d_rotation_zy = Mat4::from_rotation_x(PI*0.1);
        let camera3d_rotation_zx = Mat4::from_rotation_y(-PI*0.6);
        let camera3d_rotation_zw = Mat4::IDENTITY;

        let camera3d_rotation = camera3d_rotation_zw * camera3d_rotation_zy * camera3d_rotation_zx;
        let camera3d_offset = Vec4::new(8.0, 3.0, -2.5, 0.0);
        
        PlayerFor2d3dExample {
            id: None,

            inner_state: PlayerInnerState::new(
                Transform::new(),
                &player_settings,
                false,
                false,
                blue_map_w_level,
                red_map_w_level,
                Vec4::Y*0.6,
                audio_system,
            ),
            active_hands_slot: ActiveHandsSlot::Zero,

            hands_slot_0: Box::new(HoleGun::new(
                player_settings.energy_gun_hole_size_mult, 
                player_settings.energy_gun_add_force_mult, 
                player_settings.energy_gun_damage_mult, 
                player_settings.energy_gun_restoring_speed,
                Vec4::new(
                    0.0,
                    1.0,
                    0.0,
                    0.0
                ),
            )),
            hands_slot_1: Some(Box::new(MachineGun::new(
                player_settings.machinegun_damage,
                player_settings.machinegun_add_force, 
                player_settings.machinegun_heat_add_on_shot, 
                player_settings.machinegun_cooling_speed,
                Vec4::new(
                    0.0,
                    1.0,
                    0.0,
                    0.0
                ),
            ))),
            hands_slot_2: None,
            hands_slot_3: None,

            devices: [None, None, None, None],
            
            player_settings,

            master,

            screen_effects,

            w_scanner,

            camera3d_rotation_zy,
            camera3d_rotation_zx,
            camera3d_rotation_zw,
            camera3d_rotation,
            camera3d_offset,

            show_3d_example_current_value: 0.0,
            show_3d_example_target_value: 0.0,
        }
    }

    pub fn get_2d_slice_pos(&self) -> Vec4
    {
        self.inner_state.get_position()
    }

    pub fn get_2d_slice_xz_rot(&self) -> Mat2
    {
        let x_axis = self.inner_state.zx_rotation.x_axis;
        let z_axis = self.inner_state.zx_rotation.z_axis;

        Mat2::from_cols(
            Vec2::new(x_axis.x, x_axis.z),
            Vec2::new(z_axis.x, z_axis.z)
        )
    }

    fn process_show_3d_example
    (
        &mut self,
        input: &ActionsFrameState,
        delta: f32,
    )
    {
        if input.hold_player_rotation.is_action_just_pressed()
        {
            if self.show_3d_example_target_value == 0.0
            {
                self.show_3d_example_target_value = 1.0;
            }
            else
            {
                self.show_3d_example_target_value = 0.0;
            }
        }
    
        let example_expand_speed = 5.0 * delta;
    
        let mut diff = self.show_3d_example_target_value - self.show_3d_example_current_value;
    
        diff = diff.clamp(-example_expand_speed, example_expand_speed);
    
        self.show_3d_example_current_value += diff;
    }
}

impl ControlledActor for PlayerFor2d3dExample
{
    fn get_camera(&self) -> Camera {
        Camera {
            position: self.inner_state.get_position() + self.camera3d_offset,
            rotation_matrix: self.camera3d_rotation,
            zw_rotation_matrix: self.camera3d_rotation_zw,
            zx_rotation_matrix: self.camera3d_rotation_zx,
            zy_rotation_matrix: self.camera3d_rotation_zy,
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

        self.inner_state.is_alive = true;
        self.inner_state.is_enable = true;
        self.inner_state.hp = PLAYER_MAX_HP;
        self.inner_state.amount_of_move_w_bonuses_do_i_have = 0u32;

        self.inner_state.saved_angle_of_rotation = Vec4::ZERO;

        self.w_scanner.restore_scanner_values(&self.player_settings);

        self.inner_state.restore_w_shift_and_rotate_values();

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
        self.w_scanner.w_scanner_reloading_time = self.player_settings.scanner_reloading_time;

        self.inner_state.collider.reset_forces_and_velocity();

        self.inner_state.transform = Transform::from_position(current_spawn.spawn_position);

        self.inner_state.player_previous_w_position = current_spawn.spawn_position.w;

        let player_doll_input_state = PlayerDollInputState {
            move_forward: false,
            move_backward: false,
            move_right: false,
            move_left: false,
            will_jump: false,
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