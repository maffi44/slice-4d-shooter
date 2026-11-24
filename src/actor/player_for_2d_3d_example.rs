// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2026  Timofei Molokov

use crate::engine::shared_systems::audio::AudioSystem;
use crate::engine::shared_systems::resources::{ResourceLoader, Retention};
use fyrox_sound::buffer::SoundBufferResource;
use crate::actor::main_player::constants::*;

use crate::engine::world::WorldContext;
use crate::engine::shared_systems::EngineSharedSystems;
use crate::actor::session_controller::send_to_session_controller;
use crate::engine::shared_systems::net::Reliability;

use net_protocol::gameplay::{
    NetMessageToPlayer,
    RemoteCommand,
    RemoteMessage,
    Team
};
use rand::RngExt;

use crate::{
    actor::{
        device::{
            holegun::HoleGun,
            Device,
        },
        main_player::{player_inner_state::PlayerInnerState, PlayerMessage, PlayerScreenEffects, PlayersProjections},
        players_doll::PlayerDollInputState,
        Actor,
        ActorID,
        CommonActorsMessage,
        Message,
        MessageType,
        SpecificActorMessage
    },
    engine::{
        shared_systems::input::ActionsFrameState, world::physics::{
            colliders_container::PhysicalElement,
            kinematic_collider::KinematicColliderMessage,
            PhysicsSystem
        }, shared_systems::render::{camera::Camera, VisualElement}, shared_systems::ui::{
            UIElement, UIElementType
        }, world::static_object::{BeamVolumeArea, SphericalVolumeArea, VolumeArea}
    },
    engine::transform::{Transform, BACKWARD, DOWN, FORWARD, UP},
};


use core::panic;
use std::f32::consts::PI;
use fyrox_sound::source::Status;
use glam::{
    Mat2, Mat4, Vec2, Vec3, Vec4
};

use super::{
    device::{machinegun::MachineGun, shotgun::Shotgun}, flag::{FlagMessage, FlagStatus}, main_player::{self, ActiveHandsSlot, WScanner, GET_DAMAGE_PROJECTION_INTENSITY, MAX_MOVE_W_BONUSES_I_CAN_HAVE, PLAYER_MAX_HP, PLAYER_PROJECTION_DISPLAY_TIME, SHOW_CROSSHAIER_HIT_MARK_TIME}, move_w_bonus::{BonusSpotStatus, MoveWBonusSpotMessage}, players_doll::PlayersDollMessage, session_controller::SessionControllerMessage, ControlledActor, PhysicsMessages
};


pub struct PlayerFor2d3dExample {
    rotating_sound: Option<SoundBufferResource>,
    shifting_sound: Option<SoundBufferResource>,
    player_get_scanned_sound: Option<SoundBufferResource>,
    player_respawned_sound: Option<SoundBufferResource>,


    id: Option<ActorID>,

    inner_state: PlayerInnerState,

    active_hands_slot: ActiveHandsSlot, 

    hands_slot_0: Box<dyn Device + Send>,
    hands_slot_1: Option<Box<dyn Device + Send>>,
    hands_slot_2: Option<Box<dyn Device + Send>>,
    hands_slot_3: Option<Box<dyn Device + Send>>,

    devices: [Option<Box<dyn Device + Send>>; 4],

    screen_effects: PlayerScreenEffects,

    w_scanner: WScanner,

    camera3d_rotation: Mat4,
    camera3d_rotation_zy: Mat4,
    camera3d_rotation_zx: Mat4,
    camera3d_rotation_zw: Mat4,
    camera3d_offset: Vec4,

    pub show_3d_example_current_value: f32,
    show_3d_example_target_value: f32,

    current_w_position_target: f32,

    volume_beam_pointer: Vec<VolumeArea>,

    model_has_left_orientation: bool,

    camera_dynamic_offset: Vec4,
    camera_dynamic_offset_target: Vec4,

    red_spawn_points: Vec<Vec4>,
    blue_spawn_points: Vec<Vec4>,
}

impl Actor for PlayerFor2d3dExample {
    fn on_added_to_world(&mut self, _: &mut WorldContext, shared: &mut EngineSharedSystems) {
        self.inner_state.rotating_around_w_sound_handle = shared.audio.spawn_non_spatial_sound(
            &self.rotating_sound,
            0.0,
            1.0,
            true,
            false,
            fyrox_sound::source::Status::Playing
        );

        self.inner_state.shifting_along_w_sound_handle = shared.audio.spawn_non_spatial_sound(
            &self.shifting_sound,
            0.0,
            1.0,
            true,
            false,
            fyrox_sound::source::Status::Playing
        );
    }


    fn get_actor_as_controlled(&self) -> Option<&dyn ControlledActor> {
        Some(self)
    }

    fn get_actor_as_controlled_mut(&mut self) -> Option<&mut dyn ControlledActor> {
        Some(self)
    }

    fn recieve_message(
        &mut self,
        message: &Message,
        world: &mut WorldContext,
        shared: &mut EngineSharedSystems,
    ) {
        let physic_system = world.physics;
        let messenger = &mut *world.messenger;
        let audio_system = &mut shared.audio;
        let ui_system = &mut shared.ui;
        let _time_system = &shared.time;

        let from = message.from;

        let message = &message.message;
        
        match message
        {
            MessageType::CommonActorsMessages(message) =>
            {
                match message
                {
                    CommonActorsMessage::SetTransform(transform) =>
                    {
                        self.inner_state.transform = *transform;
                    },

                    CommonActorsMessage::Enable(switch) =>
                    {
                        self.inner_state.is_enable = *switch;
                    },

                    CommonActorsMessage::IncrementPosition(increment) =>
                    {
                        self.inner_state.transform.increment_position(*increment);
                    },
                    CommonActorsMessage::IWasChangedMyId(_new_id) => {}
                    CommonActorsMessage::ClientDisconnectedFromGameServer => { self.inner_state.peer_id = None; }
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
                                    &mut self.screen_effects,
                                    &mut self.devices,
                                    my_id,
                                    physic_system,
                                    audio_system,
                                    ui_system,
                                    messenger,
                                );
                            }
                        }
                    }
                    _ => {}
                }
            },

            MessageType::SpecificActorMessage(message) =>
            {
                match message
                {
                    SpecificActorMessage::PlayerMessage(message) =>
                    {
                        match message
                        {
                            PlayerMessage::AntiProjectionModeTurnedOn => {},

                            PlayerMessage::YouWasScanned =>
                            {
                                audio_system.spawn_non_spatial_sound(
                                    &self.player_get_scanned_sound,
                                    0.45,
                                    0.8,
                                    false,
                                    true,
                                    Status::Playing,
                                );
                            }

                            PlayerMessage::DataForProjection(
                                updated_projection_position,
                                updated_projection_radius,
                                anti_projection_mode_enabled,
                                player_is_alive,
                            ) =>
                            {
                                self.screen_effects.player_projections.update_projection_state_for_2d_3d_example(
                                    from,
                                    *updated_projection_position,
                                    *updated_projection_radius,
                                    *anti_projection_mode_enabled,
                                    *player_is_alive,
                                    &self.inner_state
                                );
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
                                    &mut self.screen_effects,
                                    &mut self.devices,
                                    my_id,
                                    physic_system,
                                    audio_system,
                                    ui_system,
                                    messenger,
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
                                    &mut self.screen_effects,
                                    &mut self.devices,
                                    my_id,
                                    physic_system,
                                    audio_system,
                                    ui_system,
                                    messenger,
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
                                    &mut self.screen_effects,
                                    &mut self.devices,
                                    my_id,
                                    physic_system,
                                    audio_system,
                                    ui_system,
                                    messenger,
                                );
                            }

                            PlayerMessage::GetDamageAndForce(
                                damage,
                                force,
                                _,
                                team,
                                _
                            ) =>
                            {
                                if *team != self.inner_state.team
                                {
                                    self.screen_effects.player_projections.update_or_add_projection(
                                        from,
                                        PLAYER_PROJECTION_DISPLAY_TIME,
                                        GET_DAMAGE_PROJECTION_INTENSITY,
                                        self.get_id().expect("Player for Example have not ActorID"),
                                        false,
                                        audio_system,
                                        messenger,
                                    );

                                    let my_id = self.get_id().expect("Player Have not ActorID");

                                    main_player::get_damage_and_add_force(
                                        *damage as i32,
                                        *force,
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
                                        physic_system,
                                        audio_system,
                                        ui_system,
                                        messenger,
                                    );
                                }
                            }

                            PlayerMessage::ChangeYourId(new_id) =>
                            {
                                self.inner_state.peer_id = Some(*new_id);
                                let available = FlagMessage::CarrierAvailable(*new_id).from_actor(self.id.expect("Player has no ActorID"));
                                messenger.send_broadcast_message(available);
                            }


                            PlayerMessage::NewPeerConnected(peer_id) =>
                            {
                                send_to_session_controller(
                                    messenger,
                                    self.id.unwrap(),
                                    SessionControllerMessage::SendToPlayer(
                                        NetMessageToPlayer::RemoteCommand(
                                            RemoteCommand::SpawnPlayersDollActor(
                                                self.get_transform().to_serializable_transform(),
                                                self.inner_state.collider.get_collider_radius(),
                                                self.inner_state.is_alive,
                                                self.inner_state.team
                                            )
                                        ),
                                        *peer_id,
                                        Reliability::Reliable,
                                    ),
                                )
                            }

                            PlayerMessage::SetNewTeam(team) =>
                            {
                                self.inner_state.team = *team;
                                self.inner_state.amount_of_move_w_bonuses_do_i_have = 0u32;

                                main_player::set_right_team_hud(
                                    &self.inner_state,
                                    ui_system
                                );

                                todo!("must impliment respawn for observer");
                                // messenger.send_command(
                                //     Command {
                                //         sender: self.get_id().expect("Player have not ActorID"),
                                //         command_type: CommandType::RespawnPlayer(
                                //             self.get_id().expect("Player have not ActorID")
                                //         )
                                //     }
                                // );
                            }
                        }
                    },
                    SpecificActorMessage::SessionControllerMessage(message) =>
                    {
                        match message
                        {
                            SessionControllerMessage::NewSessionStarted(team) =>
                            {
                                self.inner_state.team = *team;
                                self.inner_state.is_time_after_some_team_win = false;
                                self.inner_state.amount_of_move_w_bonuses_do_i_have = 0u32;

                                main_player::set_right_team_hud(
                                    &self.inner_state,
                                    ui_system
                                );

                                todo!("must impliment respawn for observer");
                                // messenger.send_command(
                                //     Command {
                                //         sender: self.get_id().expect("Player have not ActorID"),
                                //         command_type: CommandType::RespawnPlayer(
                                //             self.get_id().expect("Player have not ActorID")
                                //         )
                                //     }
                                // );

                            }

                            SessionControllerMessage::JoinedToSession(
                                your_team, _, _, _, _, _,
                            ) =>
                            {
                                println!("Joined to game session");
                                
                                self.inner_state.team = *your_team;
                                self.inner_state.is_time_after_some_team_win = false;
                                self.inner_state.amount_of_move_w_bonuses_do_i_have = 0u32;

                                main_player::set_right_team_hud(
                                    &self.inner_state,
                                    ui_system
                                );

                                todo!("must impliment respawn for observer");
                                // messenger.send_command(
                                //     Command {
                                //         sender: self.get_id().expect("Player have not ActorID"),
                                //         command_type: CommandType::RespawnPlayer(
                                //             self.get_id().expect("Player have not ActorID")
                                //         )
                                //     }
                                // );
                            }

                            SessionControllerMessage::TeamWin(_team) =>
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
                            FlagMessage::FindCarrier(peer) if self.inner_state.peer_id == Some(*peer) =>
                            {
                                let available = FlagMessage::CarrierAvailable(*peer).from_actor(self.id.expect("Player has no ActorID"));
                                messenger.send_direct_message(from, available);
                            }
                            FlagMessage::GiveMeTargetPosition =>
                            {
                                match self.inner_state.team {
                                    Team::Red =>
                                    {
                                        let ui_elem = ui_system.get_mut_ui_element(&UIElementType::BlueFlagBacklight);
                                        *ui_elem.get_ui_data_mut().get_is_visible_mut() = true;
                                    }
                                    Team::Blue =>
                                    {
                                        let ui_elem = ui_system.get_mut_ui_element(&UIElementType::RedFlagBacklight);
                                        *ui_elem.get_ui_data_mut().get_is_visible_mut() = true;
                                    }
                                }

                                messenger.send_direct_message(
                                    from,
                                    Message {
                                        from: self.get_id().expect("Player have not ActorID"),
                                        remote_sender: false,
                                        message: MessageType::SpecificActorMessage(
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
                                if *team_that_owns_flag == self.inner_state.team
                                {
                                    match flag_status {
                                        FlagStatus::Missed(_) =>
                                        {
                                            send_to_session_controller(
                                                messenger,
                                                self.get_id().expect("Player have not ActorID"),
                                                SessionControllerMessage::TryToReturnMyFlag,
                                            );   
                                        }

                                        FlagStatus::OnTheBase =>
                                        {
                                            send_to_session_controller(
                                                messenger,
                                                self.get_id().expect("Player have not ActorID"),
                                                SessionControllerMessage::TryToGetScore,
                                            ); 
                                        }

                                        FlagStatus::Captured(_) => {}
                                    }
                                }
                                else
                                {
                                    send_to_session_controller(
                                        messenger,
                                        self.get_id().expect("Player have not ActorID"),
                                        SessionControllerMessage::TryToGetOpponentsFlag,
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
                                        if *collected_by == self.get_id().expect("Player have not ActorID")
                                        {
                                            if self.inner_state.amount_of_move_w_bonuses_do_i_have <
                                                MAX_MOVE_W_BONUSES_I_CAN_HAVE
                                            {

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
                                    send_to_session_controller(
                                        messenger,
                                        self.get_id().expect("Player have not ActorID"),
                                        SessionControllerMessage::TryToGetMoveWBonus(
                                            *index,
                                        ),
                                    );
                                }
                            }
                        }
                    }

                    SpecificActorMessage::PlayersDollMessage(message) =>
                    {
                        match message
                        {
                            PlayersDollMessage::YouHitedMe(_,_,_) =>
                            {
                                self.inner_state.show_crosshaier_hit_mark_timer = SHOW_CROSSHAIER_HIT_MARK_TIME;

                                self.screen_effects.player_projections.update_or_add_projection(
                                    from,
                                    PLAYER_PROJECTION_DISPLAY_TIME,
                                    0.0,
                                    self.get_id().expect("Player for Example have not ActorID"),
                                    false,
                                    audio_system,
                                    messenger,
                                );
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
    }


    fn get_id(&self) -> Option<ActorID> {
        self.id
    }

    fn get_physical_element(&mut self) -> Option<PhysicalElement<'_>> {
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

    fn get_visual_element(&self) -> Option<VisualElement<'_>> {
        if self.inner_state.is_enable {
            let child_visual_elem = match self.active_hands_slot {
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

            let volume_areas = if self.inner_state.w_aim_enabled
            {
                Some(&self.volume_beam_pointer)
            }
            else
            {
                None
            };

            Some(
                VisualElement
                {
                    transform: self.get_transform(),
                    static_objects: None,
                    coloring_areas: None,
                    volume_areas,
                    waves: Some(&self.w_scanner.visual_wave),
                    player: Some((&self.inner_state.collider_for_others[0], self.inner_state.team)),
                    child_visual_elem,
                }
            )

        }
        else 
        {
            None
        }
    }

    fn tick(
        &mut self,
        world: &mut WorldContext,
        shared: &mut EngineSharedSystems,
    ) {
        let delta = world.time.delta();
        let physic_system = world.physics;
        let messenger = &mut *world.messenger;
        let audio_system = &mut shared.audio;
        let ui_system = &mut shared.ui;
        let _time_system = &shared.time;


        let input = shared.input.actions().clone();

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
                &self.inner_state,
                delta,
            );

            process_player_for_example_rotation(
                &input,
                shared.global_settings.user.mouse_sensitivity,
                &mut self.inner_state,
                &self.screen_effects,
                &mut self.volume_beam_pointer,
                self.model_has_left_orientation,
                physic_system,
                my_id,
                delta
            );

            main_player::process_projection_w_aim(
                &input,
                &mut self.inner_state,
                &mut self.screen_effects,
                ui_system,
                audio_system,
            );

            main_player::process_w_scanner_ui(
                ui_system,
                &self.inner_state,
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
                &mut self.screen_effects,
                &input,
                my_id,
                physic_system,
                audio_system,
                ui_system,
                messenger,
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
                &mut self.screen_effects,
                &input,
                my_id,
                physic_system,
                audio_system,
                ui_system,
                messenger,
            );

            // if input.show_hide_controls.is_action_just_pressed()
            // {
            //     if self.current_w_position_target == 0.0
            //     {
            //         self.current_w_position_target = 3.0
            //     }
            //     else
            //     {
            //         self.current_w_position_target = 0.0
            //     }
            // }

            process_player_for_example_movement_input(
                &input,
                &mut player_doll_input_state,
                &mut self.inner_state,
                self.current_w_position_target,
                &mut self.model_has_left_orientation,
                &mut self.screen_effects,
                audio_system,
                delta,
            );

            self.move_camera(&input, delta);
            
            main_player::process_player_primary_jump_input(
                &input,
                &mut player_doll_input_state,
                &mut self.inner_state,
            );

            main_player::process_player_second_jump_input(
                &input,
                &mut self.inner_state,
                audio_system,
                false,
                delta,
            );

            main_player::process_w_scanner(
                &input,
                &mut self.inner_state,
                &mut self.screen_effects,
                &mut self.w_scanner,
                physic_system,
                ui_system,
                audio_system,
                messenger,
                my_id,
                delta,
            );

            self.screen_effects.player_projections.projections_tick(
                my_id,
                messenger,
                audio_system,
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
                &mut self.screen_effects,
                &mut self.devices,
                my_id,
                physic_system,
                audio_system,
                ui_system,
                messenger,
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
                &mut self.screen_effects,
                &mut self.devices,
                my_id,
                physic_system,
                audio_system,
                ui_system,
                messenger,
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
                messenger,
                delta,
            );

            main_player::process_player_respawn(
                messenger,
                &input,
                &self.inner_state,
                my_id,
            );
        }

        // main_player::process_w_rotation_sound(
        //     audio_system,
        //     &mut self.inner_state,
        //     delta,
        // );

        // main_player::process_w_shift_sound(
        //     audio_system,
        //     &mut self.inner_state,
        //     true
        // );

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
            messenger,
        );
    }
}


fn process_player_for_example_rotation(
    input: &ActionsFrameState,
    mouse_sensitivity: f32,
    inner_state: &mut PlayerInnerState,
    screen_effects: &PlayerScreenEffects,
    volume_beam_pointer: &mut Vec<VolumeArea>,
    model_has_left_orientation: bool,
    physic_system: &PhysicsSystem,
    my_id: ActorID,
    _delta: f32,
)
{
    let mut xz = inner_state.saved_angle_of_rotation.x;
    let mut yz = inner_state.saved_angle_of_rotation.y;
    let zw = inner_state.saved_angle_of_rotation.w;

    inner_state.last_frame_zw_rotation = zw;

    if input.second_mouse.is_action_pressed() {
        xz = 
            input.mouse_axis.x *
            mouse_sensitivity +
            xz;
        
        yz = input.mouse_axis.y *
            mouse_sensitivity +
            yz;
        
    } else {

        let (_target_zw_angle, _rotation_speed) = {
            if inner_state.w_aim_enabled
            {
                let active_projection = screen_effects
                    .player_projections
                    .get_active_projection();

                if let Some(projection) = active_projection {
                    if let Some(projection_body) = projection.body.as_ref()
                    {
                        inner_state.w_aim_ui_frame_intensity = 0.20 +
                            (projection.is_active_intensity*4.0).clamp(0.0, 0.5);

                        (
                            match model_has_left_orientation
                            {
                                true => -projection_body.abs_zw_rotation_offset,
                                false => PI+projection_body.abs_zw_rotation_offset,
                            },
                            2.1
                        )
                    }
                    else
                    {
                        (
                            match model_has_left_orientation
                            {
                                true => 0.0,
                                false => PI,
                            },
                            1.0
                        )
                    }
                }
                else
                {
                    (
                        match model_has_left_orientation
                        {
                            true => 0.0,
                            false => PI,
                        },
                        1.0
                    )
                }
            }
            else
            {
                (
                    match model_has_left_orientation
                    {
                        true => 0.0,
                        false => PI,
                    },
                    1.0
                )
            }
        };

        // xz = lerpf(
        //     xz,
        //     target_zw_angle,
        //     delta * 4.8 * rotation_speed
        // );
        // if (xz - target_zw_angle).abs() < 0.001 {
        //     xz = target_zw_angle;
        // }

        yz = (
            input.mouse_axis.y *
            mouse_sensitivity +
            yz
        ).clamp(-PI/2.0, PI/2.0);
    }

    let zy_rotation = Mat4::from_rotation_x(yz);

    let zx_rotation = Mat4::from_rotation_y(xz);

    let zw_rotation = Mat4::from_cols_slice(&[
        1.0,    0.0,    0.0,        0.0,
        0.0,    1.0,    0.0,        0.0,
        0.0,    0.0,    (-zw).cos(),   (-zw).sin(),
        0.0,    0.0,    -(-zw).sin(),   (-zw).cos()
    ]);

    let mut rotation = zx_rotation;
    rotation *= zy_rotation;
    rotation *= zw_rotation;

    let hit = physic_system.ray_cast(
        inner_state.get_position(),
        rotation*FORWARD,
        100.0,
        Some(my_id)
    );

    let hit_position = if let Some(hit) = hit
    {
        hit.hit_point - inner_state.get_position()
    }
    else
    {
         rotation * (FORWARD*100.0)
    };

    if let VolumeArea::BeamVolumeArea(beam) = &mut volume_beam_pointer[0]
    {
        beam.translation_pos_2 = hit_position
    }
    if let VolumeArea::SphericalVolumeArea(sphere) = &mut volume_beam_pointer[1]
    {
        sphere.translation = hit_position
    }

    inner_state.saved_angle_of_rotation.x = xz;
    inner_state.saved_angle_of_rotation.y = yz;
    inner_state.saved_angle_of_rotation.w = zw;

    inner_state.zw_rotation = zw_rotation;
    inner_state.zy_rotation = zy_rotation;
    inner_state.zx_rotation = zx_rotation;
    inner_state.set_rotation_matrix(rotation);

}


fn process_player_for_example_movement_input(
    input: &ActionsFrameState,
    _player_doll_input_state: &mut PlayerDollInputState,
    inner_state: &mut PlayerInnerState,
    current_w_position_target: f32,
    model_has_left_orientation: &mut bool,
    screen_effects: &mut PlayerScreenEffects,
    audio_system: &mut AudioSystem,
    delta: f32,
)
{
    let model_has_left_orientation_current = *model_has_left_orientation;
    let mut movement_vec = Vec4::ZERO;

    if input.move_right.is_action_pressed() {
        movement_vec += FORWARD;
        // player_doll_input_state.move_forward = true;
        // model_has_left_orientation_current = true;

    }

    if input.move_left.is_action_pressed() {
        movement_vec += BACKWARD;
        // player_doll_input_state.move_backward = true;
        // model_has_left_orientation_current = false;

    }

    if model_has_left_orientation_current != *model_has_left_orientation
    {
        match model_has_left_orientation_current
        {
            true => inner_state.saved_angle_of_rotation.x -= PI,
            false => inner_state.saved_angle_of_rotation.x += PI,
        }

        *model_has_left_orientation = model_has_left_orientation_current;

        screen_effects.player_projections.deactivate_projections(
            audio_system
        );
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

    movement_vec = inner_state.get_rotation_matrix() * movement_vec;

    // lock player on w axis 
    let w_dif = current_w_position_target - inner_state.get_position().w;
    inner_state.collider.current_velocity.w = (w_dif*1.5).clamp(
        -GRAVITY_W_SPEED*25.0,
        GRAVITY_W_SPEED*25.0
    );

    // inner_state.collider.add_force(Vec4::NEG_X * GRAVITY_W_SPEED * delta);

    inner_state.collider.add_force(DOWN * GRAVITY_Y_SPEED * delta);

    if inner_state.collider.is_on_y_ground {
        inner_state.collider.set_wish_direction(
            movement_vec,
            1.0
        );
    } else {
        inner_state.collider.set_wish_direction(
            movement_vec,
            AIR_SPEED_MULT
        );
    }

    inner_state.collider.set_friction_on_air(
        inner_state.friction_on_air
    );
}


impl PlayerFor2d3dExample {

    #[allow(dead_code)] // game content that is not wired into the current maps/modes; kept for later
    pub fn new(
        resources: &mut ResourceLoader,
        _blue_base_position: Vec4,
        _red_base_position: Vec4,
        red_spawn_points: Vec<Vec4>,
        blue_spawn_points: Vec<Vec4>,
    ) -> Self {
        
        let screen_effects = PlayerScreenEffects {
            w_scanner_is_active: false,
            w_scanner_radius: 0.0,
            w_scanner_ring_intensity: 0.0,
            w_scanner_enemies_intesity: 0.0,
            death_screen_effect: 0.0,
            getting_damage_screen_effect: 0.0,
            w_shift_coef: 0.0,
            w_shift_intensity: 0.0,
            player_projections: PlayersProjections::new(resources),
            player_projections_is_visible: false
        };

        let w_scanner = WScanner::new(resources);

        // let camera3d_rotation_zy = Mat4::from_rotation_x(-PI*0.1);
        // let camera3d_rotation_zx = Mat4::from_rotation_y(PI*0.6);
        // let camera3d_rotation_zw = Mat4::IDENTITY;

        let camera3d_rotation_zy = Mat4::from_rotation_x(-PI*0.085);
        let camera3d_rotation_zx = Mat4::from_rotation_y(PI*0.5);
        let camera3d_rotation_zw = Mat4::IDENTITY;

        let mut camera3d_rotation = camera3d_rotation_zx;
        camera3d_rotation *= camera3d_rotation_zy;
        camera3d_rotation *= camera3d_rotation_zw;

        let camera3d_offset = Vec4::new(15.4, 4.35, 0.0, 0.0);

        let volume_beam_pointer = vec![
            VolumeArea::BeamVolumeArea(
                BeamVolumeArea {
                    translation_pos_1: Vec4::ZERO,
                    translation_pos_2: FORWARD*100.0,
                    radius: 0.02,
                    color: Vec3::new(3.5, 1.9, 1.68),
                }
            ),
            VolumeArea::SphericalVolumeArea(
                SphericalVolumeArea {
                    translation: FORWARD*100.0,
                    radius: 0.13,
                    color: Vec3::new(3.5, 1.9, 1.68),
                }
            )
        ];

        PlayerFor2d3dExample {
            rotating_sound: resources.load_media(
                crate::embedded_asset!("sounds/rotating_around_w.wav"), Retention::Session,
            ),
            shifting_sound: resources.load_media(
                crate::embedded_asset!("sounds/shifting_along_w.wav"), Retention::Session,
            ),
            player_get_scanned_sound: resources.load_media(
                crate::embedded_asset!("sounds/player_get_scanned.wav"), Retention::Session,
            ),
            player_respawned_sound: resources.load_media(
                crate::embedded_asset!("sounds/player_respawned.wav"), Retention::Session,
            ),

            id: None,

            inner_state: PlayerInnerState::new(
                resources,
                Transform::new(),
                false,
                false,
                UP*0.6,
                Vec4::ZERO,
                None,
                None,
            ),
            active_hands_slot: ActiveHandsSlot::Zero,

            hands_slot_0: Box::new(MachineGun::new(
                resources,
                MACHINEGUN_DAMAGE,
                MACHINEGUN_ADD_FORCE, 
                MACHINEGUN_HEAT_ADD_ON_SHOT, 
                MACHINEGUN_COOLING_SPEED,
                Vec4::new(
                    0.0,
                    1.0,
                    0.0,
                    0.0
                ),
            )),
            hands_slot_1: Some(Box::new(Shotgun::new(
                resources,
                Vec4::new(
                    0.0,
                    1.0,
                    0.0,
                    0.0
                ),
                true,
            ))),
            hands_slot_2: Some(Box::new(HoleGun::new(
                resources,
                ENERGY_GUN_HOLE_SIZE_MULT, 
                ENERGY_GUN_ADD_FORCE_MULT, 
                ENERGY_GUN_DAMAGE_MULT, 
                ENERGY_GUN_RESTORING_SPEED,
                Vec4::new(
                    0.0,
                    1.0,
                    0.0,
                    0.0
                ),
            ))),
            hands_slot_3: None,

            devices: [None, None, None, None],

            screen_effects,

            w_scanner,

            camera3d_rotation_zy,
            camera3d_rotation_zx,
            camera3d_rotation_zw,
            camera3d_rotation,
            camera3d_offset,

            show_3d_example_current_value: 1.0,
            show_3d_example_target_value: 1.0,

            current_w_position_target: 0.0,
            
            volume_beam_pointer,

            model_has_left_orientation: true,

            camera_dynamic_offset: Vec4::ZERO,
            camera_dynamic_offset_target: Vec4::ZERO,

            red_spawn_points,
            blue_spawn_points,
        }
    }


    pub fn get_2d_slice_pos(&self) -> Vec4
    {
        self.inner_state.get_position() + self.camera_dynamic_offset
    }


    pub fn get_2d_slice_xz_rot(&self) -> Mat2
    {
        let mut angle = self.inner_state.saved_angle_of_rotation.x;

        if !self.model_has_left_orientation
        {
            angle -= PI;
        }

        let zx_rotation = Mat4::from_rotation_y(angle);

        let x_axis = zx_rotation.x_axis;
        let z_axis = zx_rotation.z_axis;

        Mat2::from_cols(
            Vec2::new(x_axis.x, x_axis.z),
            Vec2::new(z_axis.x, z_axis.z)
        )
    }


    const CAMERA_TARGET_SPEED: f32 = 23.0;
    const CAMERA_SPEED: f32 = 3.2;

    fn move_camera(&mut self, input: &ActionsFrameState, delta: f32)
    {
        if input.arrow_up.is_action_pressed()
        {
            self.camera_dynamic_offset_target.y += delta*Self::CAMERA_TARGET_SPEED;
        }

        if input.arrow_down.is_action_pressed()
        {
            self.camera_dynamic_offset_target.y -= delta*Self::CAMERA_TARGET_SPEED;
        }

        if input.arrow_right.is_action_pressed()
        {
            self.camera_dynamic_offset_target.z -= delta*Self::CAMERA_TARGET_SPEED;
        }

        if input.arrow_left.is_action_pressed()
        {
            self.camera_dynamic_offset_target.z += delta*Self::CAMERA_TARGET_SPEED;
        }

        if input.move_camera_back_in_example.is_action_just_pressed()
        {
            self.camera_dynamic_offset_target = Vec4::ZERO;
        }

        self.camera_dynamic_offset = self.camera_dynamic_offset
            .lerp(
                self.camera_dynamic_offset_target,
                delta*Self::CAMERA_SPEED
            );
    }


    fn process_show_3d_example
    (
        &mut self,
        input: &ActionsFrameState,
        delta: f32,
    )
    {
        if input.show_hide_controls.is_action_just_pressed()
        {
            if self.show_3d_example_target_value == 1.0
            {
                self.show_3d_example_target_value = 0.00;
            }
            else
            {
                self.show_3d_example_target_value = 1.0;
            }
        }
    
        let example_expand_speed = 2.0 * delta;
    
        let mut diff = self.show_3d_example_target_value - self.show_3d_example_current_value;
    
        diff = diff.clamp(-example_expand_speed, example_expand_speed);
    
        self.show_3d_example_current_value += diff;
    }
}

impl ControlledActor for PlayerFor2d3dExample
{
    fn get_camera(&self) -> Camera {
        Camera {
            position: self.inner_state.get_position() + self.camera3d_offset + self.camera_dynamic_offset,
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

    fn spawn(
        &mut self,
        world: &mut WorldContext,
        shared: &mut EngineSharedSystems,
    ) {
        let physics_system = world.physics;
        let messenger = &mut *world.messenger;
        let ui_system = &mut shared.ui;
        let audio_system = &mut shared.audio;

        let spawns: &Vec<Vec4>;

        match self.get_team()
        {
            Team::Red => 
            {
                spawns = &self.red_spawn_points;
            },
            Team::Blue =>
            {
                spawns = &self.blue_spawn_points;
            },
        }
        let mut rng = rand::rng();

        let rand_index: usize = rng.random_range(0..spawns.len());
        let mut current_spawn = *spawns.get(rand_index).expect("Main Player haven't spawn points");

        for spawn in spawns.iter()
        {
            let hits = physics_system.sphere_cast_on_dynamic_colliders(
                *spawn,
                self.inner_state.get_collider_radius(),
                Some(self.get_id().expect("Player hasn't ActorID"))
            );
    
            for hit in hits {
                if let Some(team) = hit.hit_actors_team
                {
                    if self.get_team() == team
                    {
                        continue;
                    }
                }
            }

            current_spawn = *spawn;
            
            break;
        };

        let hits = physics_system.sphere_cast_on_dynamic_colliders(
            current_spawn,
            self.inner_state.get_collider_radius(),
            Some(self.get_id().expect("Player hasn't ActorID"))
        );

        for hit in hits {
            messenger.send_direct_message(
                hit.hit_actors_id.expect("In respawn func in death on respawn hit have not ActorID"),
                Message {
                    from: self.get_id().expect("Player have not ID in respawn func"),
                    remote_sender: false,
                    message: MessageType::SpecificActorMessage(
                        SpecificActorMessage::PlayerMessage(
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

        self.w_scanner.restore_scanner_values();

        self.inner_state.restore_w_shift_and_rotate_values();

        audio_system.spawn_non_spatial_sound(
            &self.player_respawned_sound,
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
                    messenger,
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
                        messenger,
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
                        messenger,
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
                        messenger,
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
                    messenger,
                );
            }
        }

        self.screen_effects.w_scanner_ring_intensity = 0.0;
        self.screen_effects.w_scanner_radius = 0.0;
        self.screen_effects.w_scanner_is_active = false;
        self.w_scanner.w_scanner_reloading_time = SCANNER_RELOADING_TIME;

        self.inner_state.collider.reset_forces_and_velocity();

        self.inner_state.transform = Transform::from_position(current_spawn);

        self.inner_state.player_previous_w_position = current_spawn.w;

        let player_doll_input_state = PlayerDollInputState {
            move_forward: false,
            move_backward: false,
            move_right: false,
            move_left: false,
            will_jump: false,
        };

        send_to_session_controller(
            messenger,
            self.get_id().expect("Player have not ActorID"),
            SessionControllerMessage::SendToPlayers(
                NetMessageToPlayer::RemoteDirectMessage(
                    self.get_id().expect("Player have not ActorID"),
                    RemoteMessage::PlayerRespawn(
                        self.inner_state.transform.to_serializable_transform(),
                        player_doll_input_state.serialize(),
                        Vec4::ZERO.to_array(),
                        self.inner_state.team
                    )
                ),
                Reliability::Reliable,
            ),
        )
    }
}