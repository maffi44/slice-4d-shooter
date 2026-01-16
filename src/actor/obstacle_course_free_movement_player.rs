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

use fyrox_core::math::lerpf;
use rand::{seq::SliceRandom, thread_rng};

use crate::{
    actor::{
        Actor, ActorID, CommonActorsMessage, Message, MessageType, SpecificActorMessage, device::{Device, EmptyHand, holegun::HoleGun, obstaclesgun::ObstaclesGun, rotator::RotatorTool}, droped_rotator_tool::DropedRotatorToolMessage, main_player::{ActiveHandsSlot, PlayerMessage, PlayerScreenEffects, PlayersProjections, player_inner_state::PlayerInnerState, player_input_master, player_settings}, new_spawn_area::NewSpawnAreaMessage, players_doll::PlayerDollInputState, trgger_orb::TriggerOrbMessage
    },
    engine::{
        audio::{
            AudioSystem,
            Sound
        }, effects::EffectsSystem, engine_handle::{
            Command,
            CommandType,
            EngineHandle
        }, input::{self, ActionsFrameState}, physics::{
            PhysicsSystem, colliders_container::PhysicalElement, kinematic_collider::KinematicColliderMessage
        }, render::{VisualElement, camera::Camera}, time::TimeSystem, ui::{UIElement, UIElementType, UISystem}, world::{level::Spawn, static_object::{BeamVolumeArea, SphericalVolumeArea, VolumeArea}}
    },
    transform::{BACKWARD, DOWN, FORWARD, LEFT, RIGHT, Transform, UP, W_DOWN, W_UP},
};

use self::{
    player_input_master::InputMaster,
    player_settings::PlayerSettings,
};

use glam::{FloatExt, Mat4, Vec2, Vec3, Vec4};

use super::{
    main_player::{self, Y_DEATH_PLANE_LEVEL}, move_w_bonus::MoveWBonusSpotMessage, session_controller::SessionControllerMessage, ControlledActor, PhysicsMessages
};

const POINTER_COLOR: Vec3 = Vec3::new(0.7, 0.0, 0.0);
const DITHERING_EFFET_FOV: f32 = 1.1;

pub struct ObstacleCourseFreeMovementPlayer {
    id: Option<ActorID>,
    inner_state: PlayerInnerState,
    pub player_settings: PlayerSettings,
    pub master: InputMaster,
    screen_effects: PlayerScreenEffects,

    active_hands_slot: ActiveHandsSlot, 

    hands_slot_0: Box<dyn Device>,
    hands_slot_1: Option<Box<dyn Device>>,
    hands_slot_2: Option<Box<dyn Device>>,
    hands_slot_3: Option<Box<dyn Device>>,

    devices: [Option<Box<dyn Device>>; 4],

    pub dithering_effect: f32,
    pub dithering_effect_target: f32,
    pub navigation_lines_mode: u32,

    current_spawn: Vec4,

    pub nav_slice_height: f32,
    pub nav_slice_height_target: f32,
    pub nav_slice_is_visible: f32,
    pub nav_slice_is_visible_target: f32,
    first_mouse_was_pressed: bool,
    pub xwz_slice_point: Vec4,

    volume_areas: Vec<VolumeArea>,

    zw_rotation_enabled: bool,
    pub rotator_tool_equiped: bool,
    show_tutorial_widndow_timer: f32,
}

impl Actor for ObstacleCourseFreeMovementPlayer {

    fn get_actor_as_controlled(&self) -> Option<&dyn ControlledActor> {
        Some(self)
    }

    fn get_actor_as_controlled_mut(&mut self) -> Option<&mut dyn ControlledActor> {
        Some(self)
    }

    fn get_visual_element(&self) -> Option<VisualElement<'_>> {
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
                volume_areas: Some(&self.volume_areas),
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
                                self.respawn(
                                    physic_system,
                                    ui_system,
                                    audio_system,
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
                    SpecificActorMessage::NewSpawnArea(message) =>
                    {
                        match message
                        {
                            NewSpawnAreaMessage::SetNewSpawnPosition(new_spawn_position) =>
                            {
                                self.current_spawn = new_spawn_position;
                            }
                        }
                    }
                    SpecificActorMessage::DropedRotatorToolMessage(message) =>
                    {
                        match message
                        {
                            DropedRotatorToolMessage::YouInteractingWithDropedRotatorTool =>
                            {
                                engine_handle.send_direct_message(
                                    from,
                                    Message {
                                        from: self.get_id().expect("Player for obstacle course have not ActorID"),
                                        remote_sender: false,
                                        message: MessageType::SpecificActorMessage(
                                            SpecificActorMessage::DropedRotatorToolMessage(
                                                DropedRotatorToolMessage::DropedRotatorToolCapturedByPlayer
                                            )
                                        )
                                    }

                                );

                                self.hands_slot_0 = Box::new(
                                    RotatorTool::new()
                                );

                                self.hands_slot_0.activate(
                                    self.get_id().expect("Player for obstacle course have not ActorID"),
                                    &mut self.inner_state,
                                    physic_system,
                                    audio_system,
                                    ui_system,
                                    engine_handle,
                                );

                                self.dithering_effect_target = DITHERING_EFFET_FOV;

                                self.navigation_lines_mode = 2u32;

                                self.show_tutorial_widndow_timer = 2.9;
                            }
                            _ => {}
                        }
                    }
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

                                self.respawn(
                                    physic_system,
                                    ui_system,
                                    audio_system,
                                    engine_handle,
                                );
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

                                self.respawn(
                                    physic_system,
                                    ui_system,
                                    audio_system,
                                    engine_handle,
                                );

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

                                self.respawn(
                                    physic_system,
                                    ui_system,
                                    audio_system,
                                    engine_handle,
                                );

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

                    SpecificActorMessage::TriggerOrbMessage(message) =>
                    {
                        match message {
                            TriggerOrbMessage::YouInteractingWithTriggerOrb =>
                            {
                                engine_handle.send_direct_message(
                                    from,
                                    Message {
                                        from: self.get_id().expect("Obstacle course player have not ActorID"),
                                        remote_sender: false,
                                        message: MessageType::SpecificActorMessage(
                                            SpecificActorMessage::TriggerOrbMessage(
                                                TriggerOrbMessage::TriggerOrbCapturedByPlayer(
                                                    self.get_id().expect("Obstacle course player have not ActorID")
                                                )
                                            )
                                        )
                                    }
                                );
                            },

                            TriggerOrbMessage::GiveMeTargetPosition => 
                            {
                                engine_handle.send_direct_message(
                                    from,
                                    Message {
                                        from: self.get_id().expect("Obstacle course player have not ActorID"),
                                        remote_sender: false,
                                        message: MessageType::SpecificActorMessage(
                                            SpecificActorMessage::TriggerOrbMessage(
                                                TriggerOrbMessage::SetTargetPosition(
                                                    self.inner_state.get_position()
                                                    +
                                                    self.inner_state.get_rotation_matrix() * Vec4::new(-0.8, 0.1, -0.9, 0.0).normalize() * 0.6
                                                )
                                            )
                                        )
                                    }
                                );
                            },

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

            // main_player::process_projection_w_aim(
            //     &input,
            //     &mut self.inner_state,
            //     &mut self.screen_effects,
            //     ui_system,
            //     audio_system,
            // );

            self.process_player_rotation(
                &input,
                ui_system,
                delta
            );

            self.process_navigation_input(&input, physic_system, ui_system, delta);

            // process_w_scanner_ui(
            //     ui_system,
            //     &self.inner_state,
            // );

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

            // process_dash(
            //     &input,
            //     &mut self.inner_state,
            //     &self.player_settings,
            //     audio_system,
            //     delta,
            // );

            self.screen_effects.player_projections.projections_tick(
                my_id,
                engine_handle,
                audio_system,
                delta
            );

            if self.inner_state.get_position().y < Y_DEATH_PLANE_LEVEL
            {
                self.respawn(
                    physic_system,
                    ui_system,
                    audio_system,
                    engine_handle,
                );
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

            self.process_player_respawn(
                physic_system,
                ui_system,
                audio_system,
                engine_handle,
                &input,
            );
        }

        self.inner_state.process_crosshair_size_and_ui(ui_system, delta);

        main_player::decrease_getting_damage_screen_effect
        (
            &mut self.screen_effects,
            delta,
        );

        // main_player::make_hud_transparency_as_death_screen_effect(
        //     &self.screen_effects,
        //     &self.inner_state,
        //     ui_system
        // );

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

    fn on_added_to_world(
            &mut self,
            physic_system: &PhysicsSystem,
            engine_handle: &mut EngineHandle,
            audio_system: &mut AudioSystem,
            ui_system: &mut UISystem,
            time_system: &mut TimeSystem,
            effects_system: &mut EffectsSystem,
        ) {
        self.respawn(physic_system, ui_system, audio_system, engine_handle);
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


impl ObstacleCourseFreeMovementPlayer {

    pub fn new(
        master: InputMaster,
        player_settings: PlayerSettings,
        spawn: Vec4,
        with_rotator_tool: bool,
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

        let mut inner_state = PlayerInnerState::new(
            Transform::new(),
            &player_settings,
            false,
            false,
            Vec4::ZERO,
            Vec4::ZERO,
            RIGHT*0.6,
            UP * player_settings.collider_radius * 0.2,
            None,
            None,
        );

        inner_state.tutrial_window_was_open = true;
        inner_state.w_aim_enabled = false;

        let tool = if with_rotator_tool
        {
            Box::new(RotatorTool::new()) as Box<dyn Device>
        }
        else
        {
            Box::new(EmptyHand::default()) as Box<dyn Device>
        };

        let volume_areas = Vec::with_capacity(3);

        let dithering_effect = if with_rotator_tool {1.0} else {0.0};
        let dithering_effect_target = dithering_effect;

        ObstacleCourseFreeMovementPlayer {
            id: None,

            inner_state,

            active_hands_slot: ActiveHandsSlot::Zero,

            hands_slot_0: tool,
            hands_slot_1: None,
            hands_slot_2: None,
            hands_slot_3: None,

            devices: [None, None, None, None],
            
            player_settings,

            master,

            screen_effects,

            dithering_effect: 0.0,
            dithering_effect_target: 0.0,
            navigation_lines_mode: 1u32,
            current_spawn: spawn,
            nav_slice_height: 0.0,
            nav_slice_height_target: 0.0,
            nav_slice_is_visible: 0.0,
            nav_slice_is_visible_target: 0.0,
            first_mouse_was_pressed: false,
            xwz_slice_point: Vec4::new(-9999.0,-9999.0,-9999.0,-9999.0),
            volume_areas,
            zw_rotation_enabled: true,
            rotator_tool_equiped: with_rotator_tool,
            show_tutorial_widndow_timer: 0.0,
        }
    }

    fn process_navigation_input(
        &mut self,
        input: &ActionsFrameState,
        physic_system: &PhysicsSystem,
        ui_system: &mut UISystem,
        delta: f32
    )
    {
        if self.show_tutorial_widndow_timer > 0.0
        {
            self.show_tutorial_widndow_timer -= delta;

            if self.show_tutorial_widndow_timer <= 1.0
            {
                *ui_system.get_mut_ui_element(&UIElementType::RotatorTutorialDraft).get_ui_data_mut().get_is_visible_mut() = true;
                ui_system.get_mut_ui_element(&UIElementType::RotatorTutorialDraft).get_ui_data_mut().set_transparency(
                    1.0 - self.show_tutorial_widndow_timer
                );
            }
            else if self.show_tutorial_widndow_timer <= 0.0
            {
                self.show_tutorial_widndow_timer = 0.0;

                ui_system.get_mut_ui_element(&UIElementType::RotatorTutorialDraft).get_ui_data_mut().set_transparency(
                    1.0
                );
            }
        }
        let with_rotator_tool = {
            match self.active_hands_slot {
                ActiveHandsSlot::Zero => {
                    match self.hands_slot_0.get_device_type()
                    {
                        crate::actor::device::DeviceType::Gun => false,
                        crate::actor::device::DeviceType::Device => false,
                        crate::actor::device::DeviceType::RotatorTool => true,
                    }
                },
                ActiveHandsSlot::First => {
                    if self.hands_slot_1.is_some()
                    {
                        match self.hands_slot_1.as_ref().unwrap().get_device_type()
                        {
                            crate::actor::device::DeviceType::Gun => false,
                            crate::actor::device::DeviceType::Device => false,
                            crate::actor::device::DeviceType::RotatorTool => true,
                        }
                    }
                    else
                    {
                        false
                    }
                },
                ActiveHandsSlot::Second => {
                    if self.hands_slot_2.is_some()
                    {
                        match self.hands_slot_2.as_ref().unwrap().get_device_type()
                        {
                            crate::actor::device::DeviceType::Gun => false,
                            crate::actor::device::DeviceType::Device => false,
                            crate::actor::device::DeviceType::RotatorTool => true,
                        }
                    }
                    else
                    {
                        false
                    }
                },
                ActiveHandsSlot::Third => {
                    if self.hands_slot_3.is_some()
                    {
                        match self.hands_slot_3.as_ref().unwrap().get_device_type()
                        {
                            crate::actor::device::DeviceType::Gun => false,
                            crate::actor::device::DeviceType::Device => false,
                            crate::actor::device::DeviceType::RotatorTool => true,
                        }
                    }
                    else
                    {
                        false
                    }
                },
            }
        };
        
        self.volume_areas.clear();
        self.xwz_slice_point= Vec4::new(-9999.0,-9999.0,-9999.0,-9999.0);


        if with_rotator_tool
        {
            if input.anti_projection_mode.is_action_just_pressed()
            {
                if self.dithering_effect_target > 0.0
                {
                    self.dithering_effect_target = 0.0;
                }
                else
                {
                    self.dithering_effect_target = DITHERING_EFFET_FOV;
                }
            }
    
            if input.w_scanner.is_action_just_pressed()
            {
                self.navigation_lines_mode += 1u32;
    
                if self.navigation_lines_mode > 2u32
                {
                    self.navigation_lines_mode = 0u32;
                }
            }

            if input.first_mouse.is_action_pressed()
            {
                self.first_mouse_was_pressed = true;

                let player_pos = self.inner_state.get_eyes_position();
                let hit = physic_system.ray_cast(
                    player_pos,
                    self.get_transform().get_rotation()*FORWARD,
                    50.0,
                    Some(self.get_id().expect("Obstacle course player have not an ActorID"))
                );

                if hit.is_some()
                {
                    let hit = hit.unwrap();

                    self.xwz_slice_point = hit.hit_point;

                    self.nav_slice_height_target = hit.hit_point.y - player_pos.y;
                    self.nav_slice_height = hit.hit_point.y - player_pos.y;
                    self.nav_slice_is_visible_target = 1.0;

                    self.nav_slice_is_visible = lerpf(
                        self.nav_slice_is_visible,
                        self.nav_slice_is_visible_target,
                        delta*5.0
                    );
                    // self.nav_slice_is_visible = 1.0;

                    let pointed_from = self.get_transform().get_rotation() * Vec4::new(-0.6,-0.2,0.0,0.0);
                    let pointed_to = hit.hit_point - self.get_transform().get_position();

                    let charging_volume_area = VolumeArea::SphericalVolumeArea(
                        SphericalVolumeArea {
                            translation: pointed_from,
                            radius: 0.05,
                            color: POINTER_COLOR,
                        }
                    );

                    let beam = VolumeArea::BeamVolumeArea(
                        BeamVolumeArea {
                            translation_pos_1: pointed_from,
                            translation_pos_2: pointed_to,
                            radius: 0.015,
                            color: POINTER_COLOR, 
                        }
                    );

                    let point = VolumeArea::SphericalVolumeArea(
                        SphericalVolumeArea {
                            translation: pointed_to,
                            radius: 0.10,
                            color: POINTER_COLOR, 
                        }
                    );

                    self.volume_areas.push(charging_volume_area);
                    self.volume_areas.push(beam);
                    self.volume_areas.push(point);
                }
                else
                {
                    self.nav_slice_height_target = 0.0;
            
                    self.nav_slice_height = lerpf(
                        self.nav_slice_height,
                        self.nav_slice_height_target,
                        delta*8.0
                    );
            
                    if (self.nav_slice_height - self.nav_slice_height_target).abs() < 0.02
                    {
                        self.nav_slice_height = self.nav_slice_height_target;
                    }
            
                    if (self.nav_slice_height - self.nav_slice_height_target).abs() > 0.04
                    {
                        self.nav_slice_is_visible_target = 1.0;
                    }
                    else
                    {
                        self.nav_slice_is_visible_target = 0.0;
                    }
            
                    self.nav_slice_is_visible = lerpf(
                        self.nav_slice_is_visible,
                        self.nav_slice_is_visible_target,
                        delta*5.0
                    );
                }
            }
            else
            {
                if self.first_mouse_was_pressed == true
                {
                    self.first_mouse_was_pressed = false;
                    self.nav_slice_height_target = 0.0;
                }

                self.nav_slice_height_target -= input.mouse_wheel_delta.y*0.00105;
        
                if input.middle_mouse.is_action_just_pressed()
                {
                    self.nav_slice_height_target = 0.0;
                }
        
                self.nav_slice_height = lerpf(
                    self.nav_slice_height,
                    self.nav_slice_height_target,
                    delta*8.0
                );
        
                if (self.nav_slice_height - self.nav_slice_height_target).abs() < 0.02
                {
                    self.nav_slice_height = self.nav_slice_height_target;
                }
        
                if (self.nav_slice_height - self.nav_slice_height_target).abs() > 0.04
                {
                    self.nav_slice_is_visible_target = 1.0;
                }
                else
                {
                    self.nav_slice_is_visible_target = 0.0;
                }
        
                self.nav_slice_is_visible = lerpf(
                    self.nav_slice_is_visible,
                    self.nav_slice_is_visible_target,
                    delta*5.0
                );
            }
        }
        else
        {
            self.dithering_effect_target = 0.0;
            self.navigation_lines_mode = 0;
            self.nav_slice_is_visible_target = 0.0;
            self.nav_slice_is_visible = 0.0;
            self.nav_slice_height = 0.0;
            self.nav_slice_height_target = 0.0;

        }

        self.dithering_effect = self.dithering_effect.lerp(self.dithering_effect_target, delta*3.0);

        if (self.dithering_effect - self.dithering_effect_target).abs() < 0.03
        {
            self.dithering_effect = self.dithering_effect_target
        }

    }

    fn respawn(
        &mut self,
        physic_system: &PhysicsSystem,
        ui_system: &mut UISystem,
        audio_system: &mut AudioSystem,
        engine_handle: &mut EngineHandle,
    ) {

        set_right_team_hud(
            &self.inner_state,
            ui_system
        );


        let hits = physic_system.sphere_cast_on_dynamic_colliders(
            self.current_spawn,
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
                    physic_system,
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
                        physic_system,
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
                        physic_system,
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
                device.activate(
                    my_id,
                    &mut self.inner_state,
                    physic_system,
                    audio_system,
                    ui_system,
                    engine_handle,
                );
            }
        }

        self.screen_effects.w_scanner_ring_intesity = 0.0;
        self.screen_effects.w_scanner_radius = 0.0;
        self.screen_effects.w_scanner_is_active = false;

        self.inner_state.collider.reset_forces_and_velocity();

        self.inner_state.transform = Transform::from_position(self.current_spawn);

        // self.current_w_level = current_spawn.w_level;

        self.inner_state.player_previous_w_position = self.current_spawn.w;

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


    pub fn process_player_respawn(
        &mut self,
        physic_system: &PhysicsSystem,
        ui_system: &mut UISystem,
        audio_system: &mut AudioSystem,
        engine_handle: &mut EngineHandle,
        input: &ActionsFrameState,
    )
    {
        if self.inner_state.after_death_timer >= self.player_settings.max_respawn_timer
        {
            self.respawn(physic_system, ui_system, audio_system, engine_handle);

            return;
        }

        if input.first_mouse.is_action_just_pressed()
        {
            if self.inner_state.after_death_timer >= self.player_settings.min_respawn_timer {
                
                self.respawn(physic_system, ui_system, audio_system, engine_handle);
                
                return;
            }
        }
    }

    pub fn process_player_rotation(
        &mut self,
        input: &ActionsFrameState,
        ui: &mut UISystem,
        delta: f32,
    )
    {
        if input.w_aim.is_action_just_pressed()
        {
            self.zw_rotation_enabled = !self.zw_rotation_enabled;
        }

        let mut xz = self.inner_state.saved_angle_of_rotation.x;
        let mut yz = self.inner_state.saved_angle_of_rotation.y;
        let mut zw = self.inner_state.saved_angle_of_rotation.z;
        let mut xw = self.inner_state.saved_angle_of_rotation.w;

        self.inner_state.last_frame_zw_rotation = zw;

        self.inner_state.w_aim_ui_frame_intensity = 0.20;

        let with_rotator_tool = {
            match self.active_hands_slot {
                ActiveHandsSlot::Zero => {
                    match self.hands_slot_0.get_device_type()
                    {
                        crate::actor::device::DeviceType::Gun => false,
                        crate::actor::device::DeviceType::Device => false,
                        crate::actor::device::DeviceType::RotatorTool => true,
                    }
                },
                ActiveHandsSlot::First => {
                    if self.hands_slot_1.is_some()
                    {
                        match self.hands_slot_1.as_ref().unwrap().get_device_type()
                        {
                            crate::actor::device::DeviceType::Gun => false,
                            crate::actor::device::DeviceType::Device => false,
                            crate::actor::device::DeviceType::RotatorTool => true,
                        }
                    }
                    else
                    {
                        false
                    }
                },
                ActiveHandsSlot::Second => {
                    if self.hands_slot_2.is_some()
                    {
                        match self.hands_slot_2.as_ref().unwrap().get_device_type()
                        {
                            crate::actor::device::DeviceType::Gun => false,
                            crate::actor::device::DeviceType::Device => false,
                            crate::actor::device::DeviceType::RotatorTool => true,
                        }
                    }
                    else
                    {
                        false
                    }
                },
                ActiveHandsSlot::Third => {
                    if self.hands_slot_3.is_some()
                    {
                        match self.hands_slot_3.as_ref().unwrap().get_device_type()
                        {
                            crate::actor::device::DeviceType::Gun => false,
                            crate::actor::device::DeviceType::Device => false,
                            crate::actor::device::DeviceType::RotatorTool => true,
                        }
                    }
                    else
                    {
                        false
                    }
                },
            }
        };

        if input.show_hide_controls.is_action_just_pressed()
        {
            let is_visible = ui
                .get_mut_ui_element(&UIElementType::RotatorTutorialDraft)
                .get_ui_data_mut()
                .get_is_visible_mut();

            if *is_visible
            {
                *is_visible = false;
            }
            else
            {
                if with_rotator_tool
                {
                    *is_visible = true;
                }    
            }
        }

        self.rotator_tool_equiped = with_rotator_tool;

        if with_rotator_tool
        {
            if input.second_mouse.is_action_pressed() {

                if self.zw_rotation_enabled
                {
                    zw = (input.mouse_axis.y *
                        *self.player_settings.mouse_sensivity.lock().unwrap() +
                        zw);
                    
                    zw = (input.gamepad_right_stick_axis_delta.y *
                        *self.player_settings.mouse_sensivity.lock().unwrap()*GAMEPAD_STICK_SENSIVITY_MULT +
                        zw);
                }
                else
                {
                    zw = my_mod(zw, PI*2.0);

                    zw = zw.lerp(0.0, delta*4.0);
                }
                
                
                xw = input.mouse_axis.x *
                    *self.player_settings.mouse_sensivity.lock().unwrap() +
                    xw;

                
                xw = input.gamepad_right_stick_axis_delta.x *
                    *self.player_settings.mouse_sensivity.lock().unwrap()*-GAMEPAD_STICK_SENSIVITY_MULT +
                    xw;
                
            }
            else
            {
                xz =
                    input.mouse_axis.x *
                    *self.player_settings.mouse_sensivity.lock().unwrap() +
                    xz;

                yz = (
                    input.mouse_axis.y *
                    *self.player_settings.mouse_sensivity.lock().unwrap() +
                    yz
                ).clamp(-PI/2.0, PI/2.0);

                xz = input.gamepad_right_stick_axis_delta.x *
                    *self.player_settings.mouse_sensivity.lock().unwrap()*-GAMEPAD_STICK_SENSIVITY_MULT +
                    xz;

                yz = (
                    input.gamepad_right_stick_axis_delta.y *
                    *self.player_settings.mouse_sensivity.lock().unwrap()*GAMEPAD_STICK_SENSIVITY_MULT +
                    yz
                ).clamp(-PI/2.0, PI/2.0);
            }

            if self.zw_rotation_enabled
            {
                zw = (input.gamepad_left_stick_axis_delta.y *
                    *self.player_settings.mouse_sensivity.lock().unwrap()*GAMEPAD_STICK_SENSIVITY_MULT +
                    zw);
                
            }
            else
            {
                zw = my_mod(zw, PI*2.0);

                zw = zw.lerp(0.0, delta*4.0);
            }

            xw = input.gamepad_left_stick_axis_delta.x *
                *self.player_settings.mouse_sensivity.lock().unwrap()*-GAMEPAD_STICK_SENSIVITY_MULT +
                xw;

        }
        else
        {
            xz =
                input.mouse_axis.x *
                *self.player_settings.mouse_sensivity.lock().unwrap() +
                xz;

            yz = (
                input.mouse_axis.y *
                *self.player_settings.mouse_sensivity.lock().unwrap() +
                yz
            ).clamp(-PI/2.0, PI/2.0);

            xz = input.gamepad_right_stick_axis_delta.x *
                *self.player_settings.mouse_sensivity.lock().unwrap()*-GAMEPAD_STICK_SENSIVITY_MULT +
                xz;

            yz = (
                input.gamepad_right_stick_axis_delta.y *
                *self.player_settings.mouse_sensivity.lock().unwrap()*GAMEPAD_STICK_SENSIVITY_MULT +
                yz
            ).clamp(-PI/2.0, PI/2.0);
        }

        let zy_rotation = Mat4::from_rotation_x(yz);

        let zx_rotation = Mat4::from_rotation_y(xz);

        let zw_rotation = Mat4::from_cols_slice(&[
            1.0,     0.0,      0.0,             0.0,
            0.0,     1.0,      0.0,             0.0,
            0.0,     0.0,      (-zw).cos(),     (-zw).sin(),
            0.0,     0.0,      -(-zw).sin(),    (-zw).cos()

        ]);

        let xw_rotation = Mat4::from_cols_slice(&[
            (-xw).cos(),     0.0,      0.0,     (-xw).sin(),
            0.0,             1.0,      0.0,     0.0,
            0.0,             0.0,      1.0,     0.0,
            -(-xw).sin(),    0.0,      0.0,     (-xw).cos()

        ]);

        self.inner_state.saved_angle_of_rotation.x = xz;
        self.inner_state.saved_angle_of_rotation.y = yz;
        self.inner_state.saved_angle_of_rotation.z = zw;
        self.inner_state.saved_angle_of_rotation.w = xw;

        // inner_state.zw_rotation = zw_rotation;
        // inner_state.zy_rotation = zy_rotation;
        // inner_state.zx_rotation = zx_rotation;

        let mut rotation = Mat4::IDENTITY;
        rotation *= zw_rotation;
        rotation *= xw_rotation;
        rotation *= zx_rotation;
        rotation *= zy_rotation;

        // temporally
        self.inner_state.zw_rotation = rotation;

        self.inner_state.set_rotation_matrix(rotation);
    }
}

impl ControlledActor for ObstacleCourseFreeMovementPlayer
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

    // let hud_elem = ui.get_mut_ui_element(&UIElementType::ScannerHPointer);
    // *hud_elem.get_ui_data_mut().get_is_visible_mut() = true;

    // let hud_elem = ui.get_mut_ui_element(&UIElementType::ZWScannerArrow);
    // *hud_elem.get_ui_data_mut().get_is_visible_mut() = true;

    // let hud_elem = ui.get_mut_ui_element(&UIElementType::ZXScannerArrow);
    // *hud_elem.get_ui_data_mut().get_is_visible_mut() = true;

    // let hud_elem = ui.get_mut_ui_element(&UIElementType::TitlePressTForTutorial);
    // *hud_elem.get_ui_data_mut().get_is_visible_mut() = true;


    match inner_state.team
    {
        Team::Red =>
        {
            // let hud_elem = ui.get_mut_ui_element(&UIElementType::ScannerRed);
            // *hud_elem.get_ui_data_mut().get_is_visible_mut() = true;

            // let hud_elem = ui.get_mut_ui_element(&UIElementType::HeathBarRed);
            // *hud_elem.get_ui_data_mut().get_is_visible_mut() = true;

            // let hud_elem = ui.get_mut_ui_element(&UIElementType::LeftScannerDsiplayRed);
            // *hud_elem.get_ui_data_mut().get_is_visible_mut() = true;

            // let hud_elem = ui.get_mut_ui_element(&UIElementType::RightScannerDsiplayRed);
            // *hud_elem.get_ui_data_mut().get_is_visible_mut() = true;

            // let hud_elem = ui.get_mut_ui_element(&UIElementType::ScannerBlue);
            // *hud_elem.get_ui_data_mut().get_is_visible_mut() = false;

            // let hud_elem = ui.get_mut_ui_element(&UIElementType::HeathBarBlue);
            // *hud_elem.get_ui_data_mut().get_is_visible_mut() = false;

            // let hud_elem = ui.get_mut_ui_element(&UIElementType::LeftScannerDsiplayBlue);
            // *hud_elem.get_ui_data_mut().get_is_visible_mut() = false;

            // let hud_elem = ui.get_mut_ui_element(&UIElementType::RightScannerDsiplayBlue);
            // *hud_elem.get_ui_data_mut().get_is_visible_mut() = false;
        }

        Team::Blue =>
        {
            // let hud_elem = ui.get_mut_ui_element(&UIElementType::ScannerRed);
            // *hud_elem.get_ui_data_mut().get_is_visible_mut() = false;

            // *hud_elem.get_ui_data_mut().get_is_visible_mut() = false;

            // let hud_elem = ui.get_mut_ui_element(&UIElementType::HeathBarRed);
            // *hud_elem.get_ui_data_mut().get_is_visible_mut() = false;

            // let hud_elem = ui.get_mut_ui_element(&UIElementType::LeftScannerDsiplayRed);
            // *hud_elem.get_ui_data_mut().get_is_visible_mut() = false;

            // let hud_elem = ui.get_mut_ui_element(&UIElementType::RightScannerDsiplayRed);
            // *hud_elem.get_ui_data_mut().get_is_visible_mut() = false;

            // let hud_elem = ui.get_mut_ui_element(&UIElementType::ScannerBlue);
            // *hud_elem.get_ui_data_mut().get_is_visible_mut() = true;

            // let hud_elem = ui.get_mut_ui_element(&UIElementType::HeathBarBlue);
            // *hud_elem.get_ui_data_mut().get_is_visible_mut() = true;

            // let hud_elem = ui.get_mut_ui_element(&UIElementType::LeftScannerDsiplayBlue);
            // *hud_elem.get_ui_data_mut().get_is_visible_mut() = true;

            // let hud_elem = ui.get_mut_ui_element(&UIElementType::RightScannerDsiplayBlue);
            // *hud_elem.get_ui_data_mut().get_is_visible_mut() = true;
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

    match movement_vec.try_normalize()
    {
        Some(vec) => movement_vec = vec,
        None => movement_vec = Vec4::ZERO,
    }

    // add w gravity
    // inner_state.collider.add_force(
    //     W_DOWN *
    //     (inner_state.get_position().w*25.0).clamp(-player_settings.gravity_w_speed, player_settings.gravity_w_speed)
    //     * delta
    // );

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

const GAMEPAD_STICK_SENSIVITY_MULT: f32 = 0.15;

fn my_mod(x: f32, y: f32) -> f32
{
    return x - y * (x / y).floor();
}