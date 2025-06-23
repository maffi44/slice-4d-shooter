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
        main_player::{player_inner_state::PlayerInnerState, player_input_master, player_settings, PlayerMessage, PlayerMovingState, PlayerScreenEffects, PlayersProjections},
        players_doll::PlayerDollInputState,
        Actor,
        ActorID,
        CommonActorsMessage,
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
    transform::{Transform, BACKWARD, DOWN, FORWARD, LEFT, RIGHT, UP, W_DOWN, W_UP},
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
    device::machinegun::MachineGun, flag::{FlagMessage, FlagStatus}, main_player::{self, ActiveHandsSlot, WScanner, GET_DAMAGE_PROJECTION_INTENSITY, PLAYER_PROJECTION_DISPLAY_TIME, Y_DEATH_PLANE_LEVEL}, move_w_bonus::{BonusSpotStatus, MoveWBonusSpotMessage}, mover_w::MoverWMessage, players_death_explosion::PlayersDeathExplosion, players_doll::PlayersDollMessage, session_controller::{SessionControllerMessage, DEFAULT_TEAM}, ControlledActor, PhysicsMessages
};


pub struct Observer {
    id: Option<ActorID>,
    inner_state: PlayerInnerState,
    pub player_settings: PlayerSettings,
    pub master: InputMaster,
    screen_effects: PlayerScreenEffects,

    free_movement_is_enabled: bool, 
}

impl Actor for Observer {

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
                                engine_handle.send_command(
                                    Command {
                                        sender: self.get_id().expect("Player have not ActorID"),
                                        command_type: CommandType::RespawnPlayer(
                                            self.get_id().expect("Player have not ActorID")
                                        )
                                    }
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
                    SpecificActorMessage::PlayerMessage(message) =>
                    {
                        match message {

                            PlayerMessage::SetNewTeam(team) =>
                            {
                                self.inner_state.team = team;
                                self.inner_state.amount_of_move_w_bonuses_do_i_have = 0u32;

                                engine_handle.send_command(
                                    Command {
                                        sender: self.get_id().expect("Player have not ActorID"),
                                        command_type: CommandType::RespawnPlayer(
                                            self.get_id().expect("Player have not ActorID")
                                        )
                                    }
                                );
                            }

                            PlayerMessage::DataForProjection(
                                updated_projection_position,
                                updated_projection_radius
                            ) =>
                            {
                                self.screen_effects.player_projections.update_projection_postiton(
                                    from,
                                    updated_projection_position,
                                    updated_projection_radius,
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
                                self.inner_state.team = your_team;

                                engine_handle.send_command(
                                    Command {
                                        sender: self.get_id().expect("Player have not ActorID"),
                                        command_type: CommandType::RespawnPlayer(
                                            self.get_id().expect("Player have not ActorID")
                                        )
                                    }
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

        let my_id = self.get_id().expect("Player have not ActorID");

        main_player::process_player_rotation(
            &input,
            &self.player_settings,
            &mut self.inner_state,
            &self.screen_effects,
            delta
        );

        if input.w_scanner.is_action_just_pressed()
        {
            self.free_movement_is_enabled = !self.free_movement_is_enabled; 
        }

        if !self.free_movement_is_enabled {

            main_player::process_player_movement_input(
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

            main_player::process_player_second_jump_input(
                &input,
                &mut self.inner_state,
                &self.player_settings,
                audio_system,
                true,
                delta,
            );

            // self.screen_effects.player_projections.projections_tick(
            //     my_id,
            //     engine_handle,
            //     audio_system,
            //     delta
            // );
        }
        else
        {
            process_free_movement_input(
                &input,
                &mut self.inner_state,
                &self.player_settings,
                delta
            );
        }
        // main_player::procces_w_rotation_sound(
        //     audio_system,
        //     &mut self.inner_state,
        //     delta,
        // );

        // main_player::procces_w_shift_sound(
        //     audio_system,
        //     &mut self.inner_state,
        // );

        if self.inner_state.get_position().y < Y_DEATH_PLANE_LEVEL
        {
            engine_handle.send_command(
                Command {
                    sender: self.get_id().expect("Player have not ActorID"),
                    command_type: CommandType::RespawnPlayer(
                        self.get_id().expect("Player have not ActorID")
                    )
                }
            );
        }
        
        main_player::set_audio_listener_position
        (
            audio_system,
            &self.inner_state,
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

    inner_state.collider.set_wish_direction(
        movement_vec,
        1.0
    );

    inner_state.collider.set_friction_on_air(
        inner_state.friction_on_air*17.0
    );
}


impl Observer {

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
            death_screen_effect: 0.0,
            getting_damage_screen_effect: 0.0,
            w_shift_coef: 0.0,
            w_shift_intensity: 0.0,
            player_projections: PlayersProjections::new(),
        };
        
        Observer {
            id: None,

            inner_state: PlayerInnerState::new(
                Transform::new(),
                &player_settings,
                false,
                false,
                blue_base_position,
                red_base_position,
                UP*0.6,
                UP * player_settings.collider_radius * 0.2,
                audio_system,
            ),
            
            player_settings,

            master,

            screen_effects,

            free_movement_is_enabled: false
        }
    }
}

impl ControlledActor for Observer
{
    fn get_camera(&self) -> Camera {
        Camera {
            position: self.inner_state.get_eyes_position()+Vec4::Y*0.28,
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
        let current_spawn = spawns.last().expect("spawns in respawn function has zero length");

        self.inner_state.is_alive = true;
        self.inner_state.is_enable = true;
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

        self.screen_effects.w_scanner_ring_intesity = 0.0;
        self.screen_effects.w_scanner_radius = 0.0;
        self.screen_effects.w_scanner_is_active = false;

        self.inner_state.collider.reset_forces_and_velocity();

        self.inner_state.transform = Transform::from_position(current_spawn.spawn_position);

        self.inner_state.player_previous_w_position = current_spawn.spawn_position.w;
    }
}