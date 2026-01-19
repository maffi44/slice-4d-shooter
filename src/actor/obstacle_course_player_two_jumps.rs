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

use std::f32::consts::PI;

use client_server_protocol::{NetCommand, NetMessageToPlayer, RemoteMessage, Team};

use rand::{seq::SliceRandom, thread_rng};

use crate::{
    actor::{
        Actor, ActorID, CommonActorsMessage, Message, MessageType, SpecificActorMessage, device::{Device, holegun::HoleGun, obstaclesgun::ObstaclesGun}, main_player::{ActiveHandsSlot, PlayerMessage, PlayerScreenEffects, PlayersProjections, player_inner_state::PlayerInnerState, player_input_master, player_settings}, players_doll::PlayerDollInputState
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
            PhysicsSystem, colliders_container::PhysicalElement, kinematic_collider::KinematicColliderMessage
        }, render::{VisualElement, camera::Camera}, time::TimeSystem, ui::{UIElement, UIElementType, UISystem}, world::level::Spawn
    },
    transform::{BACKWARD, DOWN, FORWARD, LEFT, RIGHT, Transform, UP, W_DOWN, W_UP},
};

use self::{
    player_input_master::InputMaster,
    player_settings::PlayerSettings,
};

use glam::{Vec2, Vec4};

use super::{
    main_player::{self, Y_DEATH_PLANE_LEVEL}, move_w_bonus::MoveWBonusSpotMessage, session_controller::SessionControllerMessage, ControlledActor, PhysicsMessages
};


pub struct ObstacleCoursePlayerTwoJumps {
    id: Option<ActorID>,
    inner_state: PlayerInnerState,
    pub player_settings: PlayerSettings,
    pub master: InputMaster,
    screen_effects: PlayerScreenEffects,

    active_hands_slot: ActiveHandsSlot, 

    hands_slot_0: Box<dyn Device + Send>,
    hands_slot_1: Option<Box<dyn Device + Send>>,
    hands_slot_2: Option<Box<dyn Device + Send>>,
    hands_slot_3: Option<Box<dyn Device + Send>>,

    devices: [Option<Box<dyn Device + Send>>; 4],
}

impl Actor for ObstacleCoursePlayerTwoJumps {

    fn get_actor_as_controlled(&self) -> Option<&dyn ControlledActor> {
        Some(self)
    }

    fn get_actor_as_controlled_mut(&mut self) -> Option<&mut dyn ControlledActor> {
        Some(self)
    }

    fn get_visual_element(&self) -> Option<VisualElement> {
        if self.inner_state.is_enable {
            let device_visual_elem = match self.active_hands_slot {
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

            Some(VisualElement {
                transform: self.get_transform(),
                static_objects: None,
                coloring_areas: None,
                volume_areas: None,
                waves: None,//Some(&self.w_scanner.visual_wave),
                player: None,
                child_visual_elem: device_visual_elem,
            })
        }
        else
        {
            None
        }
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
                    CommonActorsMessage::ClientDisconnectedFromGameServer => {}
                }
            }

            MessageType::PhysicsMessages(message) =>
            {
                match message {
                    PhysicsMessages::KinematicColliderMessage(message) => {
                        match message {
                            KinematicColliderMessage::ColliderIsStuckInsideObject =>
                            {
                                todo!("must impliment respawn for observer");
                                // engine_handle.send_command(
                                //     Command {
                                //         sender: self.get_id().expect("Player have not ActorID"),
                                //         command_type: CommandType::RespawnPlayer(
                                //             self.get_id().expect("Player have not ActorID")
                                //         )
                                //     }
                                // );
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
                    SpecificActorMessage::PlayerMessage(message) =>
                    {
                        match message {

                            PlayerMessage::SetNewTeam(team) =>
                            {
                                self.inner_state.team = team;
                                self.inner_state.amount_of_move_w_bonuses_do_i_have = 0u32;

                                set_right_team_hud(
                                    &self.inner_state,
                                    ui_system
                                );

                                todo!("must impliment respawn for observer");
                                // engine_handle.send_command(
                                //     Command {
                                //         sender: self.get_id().expect("Player have not ActorID"),
                                //         command_type: CommandType::RespawnPlayer(
                                //             self.get_id().expect("Player have not ActorID")
                                //         )
                                //     }
                                // );
                            }

                            PlayerMessage::DataForProjection(
                                updated_projection_position,
                                updated_projection_radius,
                                anti_projection_mode_enabled,
                                player_is_alive,
                            ) =>
                            {
                                self.screen_effects.player_projections.update_projection_state(
                                    from,
                                    updated_projection_position,
                                    updated_projection_radius,
                                    anti_projection_mode_enabled,
                                    player_is_alive,
                                    &self.inner_state
                                );
                            }

                            _ => {}
                        }
                    },
                    SpecificActorMessage::SessionControllerMessage(message) =>
                    {
                        match message
                        {
                            SessionControllerMessage::NewSessionStarted(team) =>
                            {
                                self.inner_state.team = team;

                                todo!("must impliment respawn for observer");
                                // engine_handle.send_command(
                                //     Command {
                                //         sender: self.get_id().expect("Player have not ActorID"),
                                //         command_type: CommandType::RespawnPlayer(
                                //             self.get_id().expect("Player have not ActorID")
                                //         )
                                //     }
                                // );

                                set_right_team_hud(
                                    &self.inner_state,
                                    ui_system
                                );

                            }

                            SessionControllerMessage::JoinedToSession(
                                your_team, _, _, _, _, _,
                            ) =>
                            {
                                self.inner_state.team = your_team;

                                todo!("must impliment respawn for observer");
                                // engine_handle.send_command(
                                //     Command {
                                //         sender: self.get_id().expect("Player have not ActorID"),
                                //         command_type: CommandType::RespawnPlayer(
                                //             self.get_id().expect("Player have not ActorID")
                                //         )
                                //     }
                                // );

                                set_right_team_hud(
                                    &self.inner_state,
                                    ui_system
                                );
                            }

                            _ => {}
                        }
                    }

                    SpecificActorMessage::FlagMessage(message) =>
                    {
                        match message
                        {
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
                                    _ => {}
                                }
                            }

                            _ => {}
                        }
                    }

                    SpecificActorMessage::PlayersDollMessage(message) =>
                    {
                        match message
                        {
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

    fn get_physical_element(&mut self) -> Option<PhysicalElement> {
        if self.inner_state.is_enable
        {
            let collider_container = PhysicalElement {
                id: self.get_id().expect("Actor have not ActorID"),
                transform: &mut self.inner_state.transform,
                kinematic_collider: Some((&mut self.inner_state.collider, None)),
                static_colliders: None,
                dynamic_colliders: None,
                static_objects: None,
                area: None,
            };
    
            return Some(collider_container);
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

        *ui_system.get_mut_ui_element(&UIElementType::WAimFrame)
            .get_ui_data_mut()
            .get_is_visible_mut() = false;

        let my_id = self.get_id().expect("Player have not ActorID");
        
        if self.inner_state.is_alive {

            main_player::process_screen_effects_while_alive
            (
                &mut self.screen_effects,
                &self.inner_state,
                delta,
            );

            main_player::process_projection_w_aim(
                &input,
                &mut self.inner_state,
                &mut self.screen_effects,
                ui_system,
                audio_system,
            );

            main_player::process_player_rotation(
                &input,
                &self.player_settings,
                &mut self.inner_state,
                &self.screen_effects,
                delta
            );

            process_w_scanner_ui(
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
                &mut self.screen_effects,
                &input,
                my_id,
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

            main_player::process_player_primary_jump_input(
                &input,
                &mut player_doll_input_state,
                &mut self.inner_state,
                &self.player_settings,
            );

            process_dash(
                &input,
                &mut self.inner_state,
                &self.player_settings,
                audio_system,
                delta,
            );

            process_ata_keta_jumps(
                &input,
                &mut self.inner_state,
                &self.player_settings,
                audio_system,
                delta,
            );

            self.screen_effects.player_projections.projections_tick(
                my_id,
                engine_handle,
                audio_system,
                delta
            );

            if self.inner_state.get_position().y < Y_DEATH_PLANE_LEVEL
            {
                todo!("must impliment respawn for observer");
                // engine_handle.send_command(
                //     Command {
                //         sender: self.get_id().expect("Player have not ActorID"),
                //         command_type: CommandType::RespawnPlayer(
                //             self.get_id().expect("Player have not ActorID")
                //         )
                //     }
                // );
            }

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

        // main_player::process_ui_tutorial_window_input(
        //     &input,
        //     &mut self.inner_state,
        //     ui_system,
        //     engine_handle,
        //     my_id,
        //     delta,
        // );

        main_player::process_w_rotation_sound(
            audio_system,
            &mut self.inner_state,
            delta,
        );

        main_player::process_w_shift_sound(
            audio_system,
            &mut self.inner_state,
            false
        );

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

fn process_free_movement_input(
    input: &ActionsFrameState,
    inner_state: &mut PlayerInnerState,
    player_settings: &PlayerSettings,
    delta: f32,
)
{
    let mut movement_vec = Vec4::ZERO;
    
    if input.move_forward.is_action_pressed() {
        movement_vec += FORWARD;
    }

    if input.move_backward.is_action_pressed() {
        movement_vec += BACKWARD;
    }

    if input.move_right.is_action_pressed() {
        movement_vec += RIGHT;
    }

    if input.move_left.is_action_pressed() {
        movement_vec += LEFT;
    }

    if input.jump.is_action_pressed() {
        movement_vec += W_DOWN;
    }

    if input.jump_w.is_action_pressed() {
        movement_vec += W_UP;
    }

    if let Some(vec) = movement_vec.try_normalize() {
        movement_vec = vec;
    }

    movement_vec = inner_state.get_rotation_matrix() * movement_vec;

    match movement_vec.try_normalize()
    {
        Some(vec) => movement_vec = vec,
        None => movement_vec = Vec4::ZERO,
    }

    movement_vec.w *= 0.21;


    inner_state.collider.set_wish_direction(
        movement_vec,
        1.0
    );

    inner_state.collider.set_friction_on_air(
        inner_state.friction_on_air*33.0
    );
}


impl ObstacleCoursePlayerTwoJumps {

    pub fn new(
        master: InputMaster,
        player_settings: PlayerSettings,
        audio_system: &mut AudioSystem,
        blue_base_position: Vec4,
        red_base_position: Vec4,
    ) -> Self {
        
        let screen_effects = PlayerScreenEffects {
            w_scanner_is_active: false,
            w_scanner_radius: 0.0,
            w_scanner_ring_intesity: 0.0,
            w_scanner_enemies_intesity: 0.0,
            death_screen_effect: 1.0,
            getting_damage_screen_effect: 0.0,
            w_shift_coef: 0.0,
            w_shift_intensity: 0.0,
            player_projections: PlayersProjections::new(),
            player_projections_is_visible: false
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

        let mut inner_state = PlayerInnerState::new(
            Transform::new(),
            &player_settings,
            false,
            false,
            blue_base_position,
            red_base_position,
            RIGHT*0.6,
            UP * player_settings.collider_radius * 0.2,
            rotating_around_w_sound_handle,
            shifting_along_w_sound_handle,
        );

        inner_state.tutrial_window_was_open = true;
        inner_state.w_aim_enabled = false;
        
        ObstacleCoursePlayerTwoJumps {
            id: None,

            inner_state,

            active_hands_slot: ActiveHandsSlot::Zero,

            hands_slot_0: Box::new(HoleGun::new(
                player_settings.energy_gun_hole_size_mult, 
                player_settings.energy_gun_add_force_mult, 
                player_settings.energy_gun_damage_mult, 
                player_settings.energy_gun_restoring_speed,
                Vec4::new(
                    1.0,
                    -0.3,
                    -1.0,
                    0.0
                ),
            )),
            hands_slot_1: Some(Box::new(ObstaclesGun::new(
                player_settings.energy_gun_hole_size_mult, 
                player_settings.energy_gun_add_force_mult, 
                player_settings.energy_gun_damage_mult, 
                player_settings.energy_gun_restoring_speed,
                Vec4::new(
                    1.0,
                    -0.3,
                    -1.0,
                    0.0
                ),
            ))),
            hands_slot_2: None,
            hands_slot_3: None,

            devices: [None, None, None, None],
            
            player_settings,

            master,

            screen_effects,
        }
    }
}

impl ControlledActor for ObstacleCoursePlayerTwoJumps
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
                    remote_sender: false,
                    message: MessageType::SpecificActorMessage(
                        SpecificActorMessage::PlayerMessage(
                            PlayerMessage::Telefrag
                        )
                    )
                }
            )
        }

        self.inner_state.w_aim_enabled = true;
        self.inner_state.is_alive = true;
        self.inner_state.is_enable = true;
        self.inner_state.hp = main_player::PLAYER_MAX_HP;
        self.inner_state.amount_of_move_w_bonuses_do_i_have = 0u32;
        // self.inner_state.player_moving_state =
        //     PlayerMovingState::MovingPerpendicularW(self.w_levels_of_map[current_spawn.w_level]);

        self.inner_state.saved_angle_of_rotation = Vec4::ZERO;

        // self.w_scanner.restore_scanner_values(&self.player_settings);

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
                (self.inner_state.hp as f32 / main_player::PLAYER_MAX_HP as f32)
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
        // self.w_scanner.w_scanner_reloading_time = self.player_settings.scanner_reloading_time;

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

fn set_right_team_hud(
    inner_state: &PlayerInnerState,
    ui: &mut UISystem
)
{
    let hud_elem = ui.get_mut_ui_element(&UIElementType::Crosshair);
    *hud_elem.get_ui_data_mut().get_is_visible_mut() = true;

    // let hud_elem = ui.get_mut_ui_element(&UIElementType::ScoreBar);
    // *hud_elem.get_ui_data_mut().get_is_visible_mut() = true;

    let hud_elem = ui.get_mut_ui_element(&UIElementType::ScannerHPointer);
    *hud_elem.get_ui_data_mut().get_is_visible_mut() = true;

    let hud_elem = ui.get_mut_ui_element(&UIElementType::ZWScannerArrow);
    *hud_elem.get_ui_data_mut().get_is_visible_mut() = true;

    let hud_elem = ui.get_mut_ui_element(&UIElementType::ZXScannerArrow);
    *hud_elem.get_ui_data_mut().get_is_visible_mut() = true;

    // let hud_elem = ui.get_mut_ui_element(&UIElementType::TitlePressTForTutorial);
    // *hud_elem.get_ui_data_mut().get_is_visible_mut() = true;


    match inner_state.team
    {
        Team::Red =>
        {
            let hud_elem = ui.get_mut_ui_element(&UIElementType::ScannerRed);
            *hud_elem.get_ui_data_mut().get_is_visible_mut() = true;

            let hud_elem = ui.get_mut_ui_element(&UIElementType::HeathBarRed);
            *hud_elem.get_ui_data_mut().get_is_visible_mut() = true;

            let hud_elem = ui.get_mut_ui_element(&UIElementType::LeftScannerDsiplayRed);
            *hud_elem.get_ui_data_mut().get_is_visible_mut() = true;

            let hud_elem = ui.get_mut_ui_element(&UIElementType::RightScannerDsiplayRed);
            *hud_elem.get_ui_data_mut().get_is_visible_mut() = true;

            let hud_elem = ui.get_mut_ui_element(&UIElementType::ScannerBlue);
            *hud_elem.get_ui_data_mut().get_is_visible_mut() = false;

            let hud_elem = ui.get_mut_ui_element(&UIElementType::HeathBarBlue);
            *hud_elem.get_ui_data_mut().get_is_visible_mut() = false;

            let hud_elem = ui.get_mut_ui_element(&UIElementType::LeftScannerDsiplayBlue);
            *hud_elem.get_ui_data_mut().get_is_visible_mut() = false;

            let hud_elem = ui.get_mut_ui_element(&UIElementType::RightScannerDsiplayBlue);
            *hud_elem.get_ui_data_mut().get_is_visible_mut() = false;
        }

        Team::Blue =>
        {
            let hud_elem = ui.get_mut_ui_element(&UIElementType::ScannerRed);
            *hud_elem.get_ui_data_mut().get_is_visible_mut() = false;

            *hud_elem.get_ui_data_mut().get_is_visible_mut() = false;

            let hud_elem = ui.get_mut_ui_element(&UIElementType::HeathBarRed);
            *hud_elem.get_ui_data_mut().get_is_visible_mut() = false;

            let hud_elem = ui.get_mut_ui_element(&UIElementType::LeftScannerDsiplayRed);
            *hud_elem.get_ui_data_mut().get_is_visible_mut() = false;

            let hud_elem = ui.get_mut_ui_element(&UIElementType::RightScannerDsiplayRed);
            *hud_elem.get_ui_data_mut().get_is_visible_mut() = false;

            let hud_elem = ui.get_mut_ui_element(&UIElementType::ScannerBlue);
            *hud_elem.get_ui_data_mut().get_is_visible_mut() = true;

            let hud_elem = ui.get_mut_ui_element(&UIElementType::HeathBarBlue);
            *hud_elem.get_ui_data_mut().get_is_visible_mut() = true;

            let hud_elem = ui.get_mut_ui_element(&UIElementType::LeftScannerDsiplayBlue);
            *hud_elem.get_ui_data_mut().get_is_visible_mut() = true;

            let hud_elem = ui.get_mut_ui_element(&UIElementType::RightScannerDsiplayBlue);
            *hud_elem.get_ui_data_mut().get_is_visible_mut() = true;
        }
    }
}

const W_JUMP_LEVEL: f32 = 0.05;

fn process_ata_keta_jumps(
input: &ActionsFrameState,
    inner_state: &mut PlayerInnerState,
    player_settings: &PlayerSettings,
    audio_system: &mut AudioSystem,
    delta: f32,
)
{
    if input.mouse_wheel_delta.y < 0.0
    {
        if inner_state.collider.is_on_w_ground ||
        (inner_state.get_position().w < W_JUMP_LEVEL && inner_state.get_position().w > -W_JUMP_LEVEL)
        {
            inner_state.collider.current_velocity.w = inner_state.collider.current_velocity.w.max(0.0);

            if inner_state.collider.is_on_y_ground
            {
                inner_state.collider.add_force(W_UP * player_settings.jump_w_speed*2.2);
            }
            else
            {
                inner_state.collider.add_force(W_UP * player_settings.jump_w_speed);
            }
        }
    }
    else if input.mouse_wheel_delta.y > 0.0
    {
        if inner_state.collider.is_on_w_upper_ground ||
        (inner_state.get_position().w < W_JUMP_LEVEL && inner_state.get_position().w > -W_JUMP_LEVEL)
        {
            inner_state.collider.current_velocity.w = inner_state.collider.current_velocity.w.max(0.0);

            if inner_state.collider.is_on_y_ground
            {
                inner_state.collider.add_force(W_DOWN * player_settings.jump_w_speed*2.2);
            }
            else
            {
                inner_state.collider.add_force(W_DOWN * player_settings.jump_w_speed);
            }
        }
    }
}

fn process_dash(
input: &ActionsFrameState,
    inner_state: &mut PlayerInnerState,
    player_settings: &PlayerSettings,
    audio_system: &mut AudioSystem,
    delta: f32,
)
{
    if input.jump_w.is_action_just_pressed()
    {
        inner_state.collider.add_force(inner_state.get_rotation_matrix()*FORWARD * player_settings.jump_y_speed);

        // if inner_state.collider.is_on_y_ground
        // {
        //     inner_state.collider.add_force(inner_state.get_rotation_matrix()*FORWARD * player_settings.jump_w_speed*2.2);
        // }
        // else
        // {
        //     inner_state.collider.add_force(inner_state.get_rotation_matrix()*FORWARD * player_settings.jump_w_speed);
        // }
    }
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

        movement_vec += FORWARD;

        player_doll_input_state.move_forward = true;
    }

    if input.move_backward.is_action_pressed() {
        
        movement_vec += BACKWARD;

        player_doll_input_state.move_backward = true;
    }

    if input.move_right.is_action_pressed() {
        movement_vec += RIGHT;

        player_doll_input_state.move_right = true;
    }

    if input.move_left.is_action_pressed() {
        movement_vec += LEFT;
        
        player_doll_input_state.move_left = true;
    }

    if let Some(vec) = movement_vec.try_normalize() {
        movement_vec = vec;
    }

    movement_vec = inner_state.get_rotation_matrix() * movement_vec;
    movement_vec.y = 0.0;
    movement_vec.w = 0.0;

    match movement_vec.try_normalize()
    {
        Some(vec) => movement_vec = vec,
        None => movement_vec = Vec4::ZERO,
    }

    // add w gravity
    inner_state.collider.add_force(
        W_DOWN *
        (inner_state.get_position().w*25.0).clamp(-player_settings.gravity_w_speed, player_settings.gravity_w_speed)
        * delta
    );

    // println!("added w force {}", W_DOWN *
    //     (inner_state.get_position().w*25.0).clamp(-player_settings.gravity_w_speed, player_settings.gravity_w_speed)
    //     * delta
    // );
    
    // add y gravity
    inner_state.collider.add_force(DOWN * player_settings.gravity_y_speed * delta);

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


pub fn process_w_scanner_ui(
    ui_system: &mut UISystem,
    inner_state: &PlayerInnerState,
)
{
    let xz = inner_state.saved_angle_of_rotation.x;
    let zw = inner_state.saved_angle_of_rotation.w;

    let zw_arrow = ui_system.get_mut_ui_element(&UIElementType::ZWScannerArrow);

    if let UIElement::Image(arrow) = zw_arrow {
        arrow.set_rotation_around_rect_center(-zw+PI/2.0);
    } else {
        panic!("UI Element ZWScannerArrow is not UIImage")
    }

    let zx_arrow = ui_system.get_mut_ui_element(&UIElementType::ZXScannerArrow);

    if let UIElement::Image(arrow) = zx_arrow {
        arrow.set_rotation_around_rect_center(0.0);
    } else {
        panic!("UI Element ZXScannerArrow is not UIImage")
    }

    let h_pointer = ui_system.get_mut_ui_element(&UIElementType::ScannerHPointer);

    if let UIElement::Image(h_pointer) = h_pointer {
        let h = {
            (((inner_state.get_position().w - 1.5) / 4.1) + 0.351)
                .clamp(-0.7, 0.8)
        };
        
        h_pointer.set_position(Vec2::new(0.002, h));
    } else {
        panic!("UI Element ScannerHPointer is not UIImage")
    }
}