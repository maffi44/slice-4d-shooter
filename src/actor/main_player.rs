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
            holegun::HoleGun, shotgun::Shotgun, Device, DeviceType
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
            colliders_container::PhysicalElement, dynamic_collider::PlayersDollCollider, kinematic_collider::{
                KinematicCollider,
                KinematicColliderMessage
            }, physics_system_data::{Hit, PhysicsState}, PhysicsSystem
        }, render::{camera::Camera, VisualElement}, time::TimeSystem, ui::{
            self, RectSize, UIElement, UIElementType, UISystem
        }, world::level::Spawn
    },
    transform::{Transform, BACKWARD, DOWN, FORWARD, LEFT, RIGHT, UP, W_DOWN, W_UP},
};

use self::{
    player_input_master::InputMaster,
    player_settings::PlayerSettings,
    player_inner_state::PlayerInnerState,
};

use core::panic;
use std::{collections::btree_set::Difference, f32::consts::PI, iter::Enumerate, path::Iter, usize};
use fyrox_core::{math::lerpf, pool::Handle};
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
pub enum ActiveHandsSlot {
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

    pub player_projections: PlayersProjections,
}

pub struct PlayersProjections
{
    pub projections: Vec<PlayerProjection>,
}

impl PlayersProjections
{
    pub fn new() -> Self
    {
        PlayersProjections
        {
            projections: Vec::with_capacity(10),
        }
    }


    pub fn clear(&mut self)
    {
        self.projections.clear();
    }


    pub fn get_intersected_projection
    (
        &self,
        origin: Vec4,
        view_vec: Vec4,
    ) -> Option<&PlayerProjection>
    {
        let mut closest_intr = (99999.0_f32, None);
        
        for projection in &self.projections
        {
            if let Some(body) = &projection.body
            {
                let current_intr = get_sphere_intersection(
                    origin - body.projected_position,
                    view_vec,
                    projection.get_projection_radius().expect("Projection have not body during getting intersection"),
                );

                if current_intr.x > 0.0
                {
                    if current_intr.x < closest_intr.0 
                    {
                        closest_intr.0 = current_intr.x; 
                        closest_intr.1 = Some(projection)            
                    }
                }
            }
        }

        closest_intr.1
    }


    pub fn set_projection_active(
        &mut self,
        projection_id: ActorID,
    )
    {
        for projection in &mut self.projections
        {
            if projection.id == projection_id
            {
                projection.is_active_by_timer = PROJECTION_ACTIVE_TIME;
            }
            else
            {
                projection.is_active_by_timer = 0.0;
            }
        }
    }


    pub fn update_or_add_projection(
        &mut self,
        projection_id: ActorID,
        projection_show_time: f32,
        damage_intensity: f32,
        my_id: ActorID,
        projection_by_scanner: bool,
        audio_system: &mut AudioSystem,
        engine_handle: &mut EngineHandle,
    )
    {
        let player_projection = self.find_projection_mut(
            projection_id,
        );

        match player_projection {
            Some(projection) =>
            {
                projection.timer = projection.timer.max(projection_show_time);
                projection.damage_intensity = projection.damage_intensity.max(damage_intensity);
            }
            None =>
            {
                if projection_by_scanner
                {
                    engine_handle.send_command(
                        Command {
                            sender: my_id,
                            command_type: CommandType::NetCommand(
                                NetCommand::SendDirectNetMessageReliable(
                                    NetMessageToPlayer::RemoteDirectMessage(
                                        projection_id,
                                        RemoteMessage::YouWasScanned
                                    ),
                                    projection_id
                                )
                            )
                        }
                    );
                }

                audio_system.spawn_non_spatial_sound(
                    Sound::NewProjecion,
                    0.7,
                    1.0,
                    false,
                    true,
                    Status::Playing
                );

                let projection = PlayerProjection::new(
                    projection_id,
                    projection_show_time,
                    damage_intensity,
                );

                self.projections.push(projection);
            }
        }
    }

    pub fn projections_tick(
        &mut self,
        my_id: ActorID,
        engine_handle: &mut EngineHandle,
        delta: f32,
    )
    {
        self.projections.retain_mut(|projection|
        {
            projection.timer -= delta;
            if projection.timer <= 0.0
            {
                return false;
            }

            if projection.damage_intensity > 0.0
            {
                projection.damage_intensity -= delta;
            }
            else
            {
                projection.damage_intensity = 0.0;
            }

            if projection.is_active_by_timer > 0.0
            {
                projection.is_active_by_timer -= delta;

                projection.is_active_intensity = lerpf(
                    projection.is_active_intensity,
                    projection.is_active_by_timer /
                    PROJECTION_ACTIVE_TIME,
                    delta*30.0
                );
            }
            else
            {
                projection.is_active_by_timer = 0.0;
                projection.is_active_intensity = 0.0;
            }
    
            projection.intensity = {
                lerpf(
                    projection.intensity,
                    (projection.timer*0.3).clamp(0.0, 1.0),
                    delta*12.0
                )
            };
    
            projection.body = None;
    
            engine_handle.send_direct_message(
                projection.id,
                Message {
                    from: my_id,
                    remote_sender: false,
                    message: MessageType::SpecificActorMessage(
                        SpecificActorMessage::PlayerMessage(
                            PlayerMessage::GiveMeDataForProjection,
                        )
                    )
                }
            );
    
            true
        });
    }

    pub fn find_projection(&self, id: ActorID) -> Option<&PlayerProjection>
    {
        for projection in &self.projections
        {
            if projection.id == id
            {
                return Some(projection);
            }
        }
        return None;
    }

    pub fn find_projection_mut(&mut self, id: ActorID) -> Option<&mut PlayerProjection>
    {
        for projection in &mut self.projections
        {
            if projection.id == id
            {
                return Some(projection);
            }
        }
        return None;
    }

    pub fn get_active_projection(&self) -> Option<&PlayerProjection>
    {
        for projection in &self.projections
        {
            if projection.is_active_by_timer > 0.0
            {
                return Some(projection);
            }
        }
        None
    }

    pub fn get_active_projection_mut(&mut self) -> Option<&mut PlayerProjection>
    {
        for projection in &mut self.projections
        {
            if projection.is_active_by_timer > 0.0
            {
                return Some(projection);
            }
        }
        None
    }

    pub fn update_projection_postiton(
        &mut self,
        projection_id: ActorID,
        updated_projection_original_position: Vec4,
        projection_updated_radius: f32,
        inner_state: &PlayerInnerState
    )
    {
        let projection = self.find_projection_mut(
            projection_id,
        );
     
        if let Some(projection) = projection
        {
            let player_position = inner_state.get_position();
        
            let player_to_projection_vec = updated_projection_original_position - player_position;
            let player_w_vertical_dir = inner_state.get_rotation_matrix() * W_UP;
    
            let rel_projection_w_offset = player_w_vertical_dir.dot(player_to_projection_vec.normalize());
    
            let mut rotated_player_to_projection_vec = player_to_projection_vec -
                ((rel_projection_w_offset * player_to_projection_vec.length()) * player_w_vertical_dir);
            
            // it possible if the projected player is exactly above the player on the W axis
            if rotated_player_to_projection_vec.length() == 0.0 ||
                rotated_player_to_projection_vec.is_nan() ||
                !rotated_player_to_projection_vec.is_finite()
            {
                return;
            }
    
            rotated_player_to_projection_vec = {
                (player_to_projection_vec.length() / rotated_player_to_projection_vec.length())
                *
                rotated_player_to_projection_vec
            };
            
            let projected_position = {
                player_position +
                rotated_player_to_projection_vec
            };
    
            let abs_zw_rotation_offset = W_UP.dot(player_to_projection_vec.normalize()).asin();
    
    
            if abs_zw_rotation_offset.is_nan() {panic!("Got NAN during update player projection")}
        
            let body = PlayerProjectionBody
            {
                projected_position,
                original_position: updated_projection_original_position,
                radius: projection_updated_radius * 1.111,
                abs_zw_rotation_offset,
            };
    
            projection.body = Some(body);
        }
    }
}

pub struct PlayerProjection
{
    pub id: ActorID,
    pub timer: f32,
    pub intensity: f32,
    pub is_active_intensity: f32,
    pub damage_intensity: f32,

    pub is_active_by_timer: f32,

    pub body: Option<PlayerProjectionBody>
}


impl PlayerProjection
{
    pub fn get_projection_radius(&self) -> Option<f32>
    {
        if let Some(body) = self.body.as_ref()
        {
            let radius = body.radius * (1.0 + 0.111*self.is_active_intensity);
            
            Some(radius)
        }
        else
        {
            None
        }
    }
}

pub struct PlayerProjectionBody
{
    pub projected_position: Vec4,
    pub original_position: Vec4,
    radius: f32,
    pub abs_zw_rotation_offset: f32,
}



impl PlayerProjection
{
    pub fn new(
        player_id: ActorID,
        timer: f32,
        damage_intensity: f32,
    ) -> Self
    {
        PlayerProjection {
            id: player_id,
            timer,
            intensity: 0.0,
            is_active_by_timer: 0.0,
            is_active_intensity: 0.0,
            damage_intensity,
            body: None,
        }
    }
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
            player_projections: PlayersProjections::new(),
        }
    }
}


pub struct WScanner
{
    pub w_scanner_enable: bool,
    pub w_scanner_radius: f32,
    pub w_scanner_reloading_time: f32,
    pub w_scanner_enemies_show_time: f32,
}

impl WScanner
{
    pub fn new(
        player_settings: &PlayerSettings
    ) -> WScanner
    {
        let w_scanner_reloading_time =  player_settings.scanner_reloading_time;
        let w_scanner_enemies_show_time =  player_settings.scanner_show_enemies_time;

        WScanner {
            w_scanner_enable: false,
            w_scanner_radius: 0.0,
            w_scanner_reloading_time,
            w_scanner_enemies_show_time,
        }
    }

    pub fn restore_scanner_values(&mut self, player_settings: &PlayerSettings) {
        self.w_scanner_enable = false;
        self.w_scanner_radius = 0.0;
        self.w_scanner_reloading_time = player_settings.scanner_reloading_time;
        self.w_scanner_enemies_show_time = player_settings.scanner_show_enemies_time;
    }
}


pub struct MainPlayer {
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

const CROSSHAIR_ROTATION_SPEED: f32 = -12.0;
const CROSSHAIR_CHANGE_WEAPON_TARGET_ROTATION: f32 = -PI*0.5;
const CROSSHAIR_CHANGE_WEAPON_TARGET_SIZE: f32 = 0.1;
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

const BASE_EFFECT_HP_IMPACT_SPEED: f32 = 2.6;

const PROJECTION_ACTIVE_TIME: f32 = 1.0;

pub const DEFAULT_ZW_ROTATION_TARGET_IN_RADS: f32 = 0.0;

pub const PLAYER_PROJECTION_DISPLAY_TIME: f32 = 3.4;

pub const GET_DAMAGE_PROJECTION_INTENSITY: f32 = 1.2;

#[derive(Clone)]
pub enum PlayerMessage {
    YouWasScanned,
    DealDamageAndAddForce(
        // damage
        u32,
        //force
        Vec4,
        // pos of impact (for spawn get damage effect)
        Vec4,
        // team damage from
        Team,
        // damage dealer's id
        u128
    ),
    GiveMeDataForProjection,
    DataForProjection(
        // position
        Vec4,
        // player radius
        f32,
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


impl Actor for MainPlayer {

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

                                die(
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
                    SpecificActorMessage::PlayerMessage(message) =>
                    {
                        match message {
                            PlayerMessage::YouWasScanned =>
                            {
                                audio_system.spawn_non_spatial_sound(
                                    Sound::PlayerGetScanned,
                                    0.45,
                                    0.8,
                                    false,
                                    true,
                                    Status::Playing,
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

                            PlayerMessage::GiveMeDataForProjection => {}

                            PlayerMessage::Telefrag =>
                            {
                                let my_id = self.get_id().expect("Player Have not ActorID");

                                die(
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

                                die(
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

                                die(
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
                                team,
                                damage_dealer_id
                            ) =>
                            {
                                if team != self.inner_state.team
                                {
                                    self.screen_effects.player_projections.update_or_add_projection(
                                        damage_dealer_id,
                                        PLAYER_PROJECTION_DISPLAY_TIME,
                                        GET_DAMAGE_PROJECTION_INTENSITY,
                                        self.get_id().expect("Player have not ActorID"),
                                        false,
                                        audio_system,
                                        engine_handle,
                                    );

                                    let my_id = self.get_id().expect("Player Have not ActorID");

                                    get_damage_and_add_force(
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

                                set_right_team_hud(
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

                                set_right_team_hud(
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

                                set_right_team_hud(
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
                            PlayersDollMessage::YouHitedMe(_,position, radius) =>
                            {
                                self.inner_state.show_crosshaier_hit_mark_timer = SHOW_CROSSHAIER_HIT_MARK_TIME;

                                self.screen_effects.player_projections.update_or_add_projection(
                                    from,
                                    PLAYER_PROJECTION_DISPLAY_TIME,
                                    0.0,
                                    self.get_id().expect("Main Player havn't ActorID"),
                                    false,
                                    audio_system,
                                    engine_handle,
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

        *ui_system.get_ui_element(&UIElementType::WAimFrame)
            .get_ui_data()
            .get_is_visible_cloned_arc()
            .lock()
            .unwrap()
            =
            false;

        let my_id = self.get_id().expect("Player have not ActorID");
        
        if self.inner_state.is_alive {

            process_screen_effects_while_alive
            (
                &mut self.screen_effects,
                delta,
            );

            process_projection_w_aim(
                &input,
                &mut self.inner_state,
                &mut self.screen_effects,
                ui_system,
            );

            process_player_rotation(
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

            procces_w_rotation_sound(
                audio_system,
                &mut self.inner_state,
                delta,
            );

            procces_w_shift_sound(
                audio_system,
                &mut self.inner_state,
            );

            process_active_devices_input
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

            process_switch_active_hand_slot_input
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

            process_player_primary_jump_input(
                &input,
                &mut player_doll_input_state,
                &mut self.inner_state,
                &self.player_settings,
            );

            process_player_second_jump_input(
                &input,
                &mut self.inner_state,
                &self.player_settings,
                audio_system,
                W_UP,
            );

            process_w_scanner(
                &input,
                &self.inner_state,
                &self.player_settings,
                &mut self.screen_effects,
                &mut self.w_scanner,
                physic_system,
                ui_system,
                audio_system,
                engine_handle,
                my_id,
                delta,
            );

            self.screen_effects.player_projections.projections_tick(
                my_id,
                engine_handle,
                delta
            );
            
            get_effected_by_base(
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
                &self.player_settings,
                physic_system,
                audio_system,
                ui_system,
                engine_handle,
                delta,
            );

            check_if_touching_death_plane(
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
                &self.player_settings,
                physic_system,
                audio_system,
                ui_system,
                engine_handle,
            );

        } else {
            
            //while player is not alive
            update_after_death_timer(
                &mut self.inner_state,
                delta
            );

            process_screen_effects_while_dead
            (
                &mut self.screen_effects,
                delta,
            );

            process_devices_while_player_is_dead
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

            process_player_respawn(
                engine_handle,
                &self.player_settings,
                &input,
                &self.inner_state,
                my_id,
            );
        }

        self.inner_state.process_crosshair_size_and_ui(ui_system, delta);

        decrease_getting_damage_screen_effect
        (
            &mut self.screen_effects,
            delta,
        );

        make_hud_transparency_as_death_screen_effect(
            &self.screen_effects,
            &self.inner_state,
            ui_system
        );

        set_audio_listener_position
        (
            audio_system,
            &self.inner_state,
        );

        send_player_state_to_remote_player_doll
        (
            player_doll_input_state,
            &self.inner_state,
            my_id,
            time_system,
            engine_handle,
        );
    }
}



pub fn process_projection_w_aim(
    input: &ActionsFrameState,
    inner_state: &mut PlayerInnerState,
    screen_effects: &mut PlayerScreenEffects,
    ui_system: &mut UISystem,
)
{
    if input.w_aim.is_action_just_pressed()
    {
        inner_state.w_aim_enabled = !inner_state.w_aim_enabled; 
    }

    if inner_state.w_aim_enabled
    {
        {
            *ui_system.get_ui_element(&UIElementType::WAimFrame)
                .get_ui_data()
                .get_is_visible_cloned_arc()
                .lock()
                .unwrap()
                =
                true;
        }
        
        let view_vec = inner_state.get_rotation_matrix() * FORWARD;
        let hited_projection = screen_effects
            .player_projections
            .get_intersected_projection
            (
                inner_state.get_position(),
                view_vec,
            );

        let projection_id = if let Some(projection) = hited_projection
        {

            Some(projection.id)
        }
        else
        {
            None
        };

        if let Some(projection_id) = projection_id
        {
            screen_effects.player_projections.set_projection_active(projection_id);
        }
    }
}


fn get_sphere_intersection(
    ray_origin: Vec4,
    ray_direction: Vec4,
    radius: f32
) -> Vec2
{
    let b = ray_origin.dot(ray_direction);
    let c = ray_origin.dot(ray_origin) - radius*radius;
    let mut h = b*b - c;
    
    if h < 0.0
    {
        return Vec2::NEG_ONE; // no intersection
    }

    h = h*h;

    return Vec2::new(-b-h, -b+h);
}


pub fn process_player_rotation(
    input: &ActionsFrameState,
    player_settings: &PlayerSettings,
    inner_state: &mut PlayerInnerState,
    screen_effects: &PlayerScreenEffects,
    delta: f32,
)
{
    let mut xz = inner_state.saved_angle_of_rotation.x;
    let mut yz = inner_state.saved_angle_of_rotation.y;
    let mut zw = inner_state.saved_angle_of_rotation.w;

    inner_state.last_frame_zw_rotation = zw;

    inner_state.w_aim_ui_frame_intensity = 0.20;

    if input.second_mouse.is_action_pressed() {
        zw = (input.mouse_axis.y * player_settings.mouse_sensivity + zw).clamp(-PI/2.0, PI/2.0);
        xz = input.mouse_axis.x * player_settings.mouse_sensivity + xz;
        
    }
    else
    {
        let (target_zw_angle, rotation_speed) = {
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

                        (projection_body.abs_zw_rotation_offset, 2.1)
                    }
                    else
                    {
                        (DEFAULT_ZW_ROTATION_TARGET_IN_RADS, 1.0)
                    }
                }
                else
                {
                    (DEFAULT_ZW_ROTATION_TARGET_IN_RADS, 1.0)
                }
            }
            else
            {
                (DEFAULT_ZW_ROTATION_TARGET_IN_RADS, 1.0)
            }
        };

        // target player's rotation along W
        zw = lerpf(
            zw,
            target_zw_angle,
            (delta * 4.8) * rotation_speed,
        );
        if (zw - target_zw_angle).abs() < 0.0005 {
            zw = target_zw_angle;
        }

        xz = input.mouse_axis.x * player_settings.mouse_sensivity + xz;
        yz = (input.mouse_axis.y * player_settings.mouse_sensivity + yz).clamp(-PI/2.0, PI/2.0);
    }

    let zy_rotation = Mat4::from_rotation_x(yz);

    let zx_rotation = Mat4::from_rotation_y(xz);

    let zw_rotation = Mat4::from_cols_slice(&[
        1.0,    0.0,    0.0,        0.0,
        0.0,    1.0,    0.0,        0.0,
        0.0,    0.0,    (-zw).cos(),   (-zw).sin(),
        0.0,    0.0,    -(-zw).sin(),   (-zw).cos()
    ]);

    inner_state.saved_angle_of_rotation.x = xz;
    inner_state.saved_angle_of_rotation.y = yz;
    inner_state.saved_angle_of_rotation.w = zw;

    inner_state.zw_rotation = zw_rotation;
    inner_state.zy_rotation = zy_rotation;
    inner_state.zx_rotation = zx_rotation;

    let mut rotation = zx_rotation;
    rotation *= zy_rotation;
    rotation *= zw_rotation;

    inner_state.set_rotation_matrix(rotation);
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
    inner_state: &mut PlayerInnerState,
    delta: f32,
)
{
    let zw = inner_state.saved_angle_of_rotation.w;

    let base_pitch = {
        0.8.lerp(
            1.5,
            (std::f64::consts::PI/2.0 + zw as f64) / std::f64::consts::PI
        )
    };

    let addition_pitch = {
        inner_state.rotating_around_w_sound_pitch * (1.0-delta as f64*22.0) +
        ((inner_state.last_frame_zw_rotation - zw) as f64).abs() * 2.0
    };

    inner_state.rotating_around_w_sound_pitch = addition_pitch;

    let gain = {
        inner_state.rotating_around_w_sound_gain * (1.0-(delta*42.0)) +
        (inner_state.last_frame_zw_rotation - zw).abs() * 10.0
    };

    inner_state.rotating_around_w_sound_gain = gain;

    audio_system.sound_set_pitch_and_gain(
        inner_state.rotating_around_w_sound_handle,
        base_pitch,//D + addition_pitch,
        gain
    );
}


pub fn procces_w_shift_sound(
    audio_system: &mut AudioSystem,
    inner_state: &mut PlayerInnerState,
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
        inner_state.shifting_along_w_sound_handle,
        shift_pitch as f64,//D + addition_pitch,
        shift_gain
    );

    inner_state.player_previous_w_position = inner_state.get_position().w;
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
    inner_state.collider.add_force(W_DOWN * player_settings.gravity_w_speed * delta);
    
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


pub fn process_player_primary_jump_input(
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
            inner_state.collider.add_force(UP * player_settings.jump_y_speed);

            inner_state.jumped_to_y_on_current_action = true;
            
            player_doll_input_state.will_jump = false;
        }
    }

    if input.jump.is_action_pressed() {
        if !inner_state.jumped_to_y_on_current_action {
            if inner_state.collider.is_on_y_ground {
                inner_state.collider.add_force(UP * player_settings.jump_y_speed);

                inner_state.jumped_to_y_on_current_action = true;
                
                player_doll_input_state.will_jump = false;
            }
        }
    } else {
        player_doll_input_state.will_jump = false;
    }
}


pub fn process_player_second_jump_input(
    input: &ActionsFrameState,
    inner_state: &mut PlayerInnerState,
    player_settings: &PlayerSettings,
    audio_system: &mut AudioSystem,
    mut axis: Vec4,
)
{
    axis = axis.normalize();

    if input.move_w_up.is_action_just_pressed() {
        
        // audio_system.spawn_non_spatial_sound(
        //     Sound::WJump,
        //     1.0,
        //     1.0,
        //     false,
        //     true,
        //     Status::Playing
        // );

        inner_state.collider.add_force(axis * player_settings.jump_w_speed);
    }
}




pub fn process_w_scanner(
    input: &ActionsFrameState,
    inner_state: &PlayerInnerState,
    player_settings: &PlayerSettings,
    screen_effects: &mut PlayerScreenEffects,
    w_scanner: &mut WScanner,
    physic_system: &PhysicsSystem,
    ui_system: &mut UISystem,
    audio_system: &mut AudioSystem,
    engine_handle: &mut EngineHandle,
    my_id: ActorID,
    delta: f32,
)
{
    if input.w_scanner.is_action_just_pressed() {
        if !w_scanner.w_scanner_enable {
            if w_scanner.w_scanner_reloading_time >= player_settings.scanner_reloading_time {

                audio_system.spawn_non_spatial_sound(
                    crate::engine::audio::Sound::ScannerSound,
                    0.9,
                    1.0,
                    false,
                    true,
                    fyrox_sound::source::Status::Playing,
                );

                w_scanner.w_scanner_reloading_time = 0.0;
                
                w_scanner.w_scanner_enable = true;

                w_scanner.w_scanner_enemies_show_time = 0.0;

                w_scanner.w_scanner_radius = inner_state.collider.get_collider_radius() + 0.1;
            }
        }
    }
    
    if w_scanner.w_scanner_enable {
        w_scanner.w_scanner_radius += delta * W_SCANNER_EXPANDING_SPEED;

        if w_scanner.w_scanner_radius >= W_SCANNER_MAX_RADIUS {
            w_scanner.w_scanner_enable = false;
            w_scanner.w_scanner_reloading_time = 0.0;
        }
    }

    w_scanner.w_scanner_enemies_show_time += delta;

    if !w_scanner.w_scanner_enable {

        if w_scanner.w_scanner_reloading_time < player_settings.scanner_reloading_time {
            w_scanner.w_scanner_reloading_time += delta;
        }
    }

    screen_effects.w_scanner_is_active = w_scanner.w_scanner_enable;
    screen_effects.w_scanner_radius = w_scanner.w_scanner_radius;
    screen_effects.w_scanner_ring_intesity = {
        let mut intensity = W_SCANNER_MAX_RADIUS - w_scanner.w_scanner_radius;

        intensity /= W_SCANNER_MAX_RADIUS/3.0;

        intensity.clamp(0.0, 1.0)
    };
    screen_effects.w_scanner_enemies_intesity = {
        let intensity = player_settings.scanner_show_enemies_time - w_scanner.w_scanner_enemies_show_time;

        intensity.clamp(0.0, 1.0)
    };

    // update player projections if scanner is enabled
    if w_scanner.w_scanner_enable
    {
        let hits = physic_system.sphere_cast_on_dynamic_colliders(
            inner_state.get_position(),
            screen_effects.w_scanner_radius,
            Some(my_id)
        );

        for hit in hits
        {
            let team = hit.hited_actors_team
                .expect("scanned by W Scanner dynamic collider have not Team");

            if team != inner_state.team
            {
                let projection_id = hit.hited_actors_id
                    .expect("scanned by W Scanner dynamic collider have not ActorID");

                screen_effects.player_projections.update_or_add_projection(
                    projection_id,
                    PLAYER_PROJECTION_DISPLAY_TIME,
                    0.0,
                    my_id,

                    true,
                    audio_system,
                    engine_handle,
                );
            }
        }
    }

    let scanner_ui = match inner_state.team {
        Team::Blue => ui_system.get_mut_ui_element(&UIElementType::ScannerBlue),
        Team::Red => ui_system.get_mut_ui_element(&UIElementType::ScannerRed),
    };

    if let UIElement::ProgressBar(bar) = scanner_ui {
        let bar_value = {
            (w_scanner.w_scanner_reloading_time / player_settings.scanner_reloading_time)
                .clamp(0.0, 1.0)
        };

        bar.set_bar_value(bar_value)

    } else {
        panic!("Scanner UI is not Progress Bar")
    }
}


pub fn update_after_death_timer
(
    inner_state: &mut PlayerInnerState,
    delta: f32,
)
{
    inner_state.after_death_timer += delta;
}


pub fn process_screen_effects_while_dead
(
    screen_effects: &mut PlayerScreenEffects,
    delta: f32,
)
{
    screen_effects.death_screen_effect += delta*DEATH_EFFECT_COEF_INCREASE_SPEED;
    screen_effects.death_screen_effect = screen_effects.death_screen_effect.clamp(0.0, 1.0);
}


pub fn process_screen_effects_while_alive
(
    screen_effects: &mut PlayerScreenEffects,
    delta: f32,
)
{
    screen_effects.death_screen_effect -= delta*DEATH_EFFECT_COEF_DECREASE_SPEED;
    screen_effects.death_screen_effect = screen_effects.death_screen_effect.clamp(0.0, 1.0);
}


pub fn process_player_respawn(
    engine_handle: &mut EngineHandle,
    player_settings: &PlayerSettings,
    input: &ActionsFrameState,
    inner_state: &PlayerInnerState,
    my_id: ActorID,
)
{
    if inner_state.after_death_timer >= player_settings.max_respawn_timer {
        engine_handle.send_command(
            Command {
                sender: my_id,
                command_type: CommandType::RespawnPlayer(
                    my_id
                )
            }
        );
        return;
    }

    if input.first_mouse.is_action_just_pressed() {
        if inner_state.after_death_timer >= player_settings.min_respawn_timer {
            engine_handle.send_command(
                Command {
                    sender: my_id,
                    command_type: CommandType::RespawnPlayer(
                        my_id
                    )
                }
            );
            return;
        }
    }
}


pub fn set_audio_listener_position
(
    audio_system: &mut AudioSystem,
    inner_state: &PlayerInnerState,
)
{
    audio_system.set_listener_position_and_look_vector(
        inner_state.get_position(),
        inner_state.transform.get_direction_for_audio_system()
    );
}


pub fn send_player_state_to_remote_player_doll
(
    player_doll_input_state: PlayerDollInputState,
    inner_state: &PlayerInnerState,
    my_id: ActorID,
    time_system: &TimeSystem,
    engine_handle: &mut EngineHandle,
)
{
    let remote_velocity = {
        let mut v = inner_state.collider.current_velocity;

        for force in inner_state.collider.forces.iter() {
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
                        inner_state.transform.to_serializable_transform(),
                        player_doll_input_state.serialize(),
                        remote_velocity,
                        time_system.get_server_time()
                    )
                )
            )
        )
    });
}


pub fn decrease_getting_damage_screen_effect
(
    screen_effects: &mut PlayerScreenEffects,
    delta: f32,
)
{
    screen_effects.getting_damage_screen_effect -= delta * GETTING_DAMAGE_EFFECT_COEF_DECREASE_SPEED;
    screen_effects.getting_damage_screen_effect = screen_effects.getting_damage_screen_effect.clamp(0.0, 1.0);
}


pub fn process_devices_while_player_is_dead
(
    active_hands_slot: &ActiveHandsSlot,
    hands_slot_0: &mut Box<dyn Device>,
    hands_slot_1: &mut Option<Box<dyn Device>>,
    hands_slot_2: &mut Option<Box<dyn Device>>,
    hands_slot_3: &mut Option<Box<dyn Device>>,
    devices: &mut [Option<Box<dyn Device>>;4],
    inner_state: &mut PlayerInnerState,
    input: &ActionsFrameState,
    my_id: ActorID,
    physic_system: &PhysicsSystem,
    audio_system: &mut AudioSystem,
    ui_system: &mut UISystem,
    engine_handle: &mut EngineHandle,
    delta: f32,
)
{
    match active_hands_slot {
        ActiveHandsSlot::Zero => {
            hands_slot_0.process_while_player_is_not_alive(my_id, inner_state, input, physic_system, audio_system, ui_system, engine_handle, delta);

        },
        ActiveHandsSlot::First => {
            if let Some(device) = hands_slot_1.as_mut() {
                device.process_while_player_is_not_alive(my_id, inner_state, input, physic_system, audio_system, ui_system, engine_handle, delta);
            }

        },
        ActiveHandsSlot::Second => {
            if let Some(device) = hands_slot_2.as_mut() {
                device.process_while_player_is_not_alive(my_id, inner_state, input, physic_system, audio_system, ui_system, engine_handle, delta);
            }

        },
        ActiveHandsSlot::Third => {
            if let Some(device) = hands_slot_3.as_mut() {
                device.process_while_player_is_not_alive(my_id, inner_state, input, physic_system, audio_system, ui_system, engine_handle, delta);
            }

        }
    }

    for device in devices.iter_mut() {
        if let Some(device) = device {
            device.process_while_player_is_not_alive(my_id, inner_state, input, physic_system, audio_system, ui_system, engine_handle, delta);
        }
    }
}


pub fn process_active_devices_input
(
    active_hands_slot: &ActiveHandsSlot,
    hands_slot_0: &mut Box<dyn Device>,
    hands_slot_1: &mut Option<Box<dyn Device>>,
    hands_slot_2: &mut Option<Box<dyn Device>>,
    hands_slot_3: &mut Option<Box<dyn Device>>,
    devices: &mut [Option<Box<dyn Device>>;4],
    inner_state: &mut PlayerInnerState,
    screen_effects: &mut PlayerScreenEffects,
    input: &ActionsFrameState,
    my_id: ActorID,
    physic_system: &PhysicsSystem,
    audio_system: &mut AudioSystem,
    ui_system: &mut UISystem,
    engine_handle: &mut EngineHandle,
    delta: f32,
)
{
    match active_hands_slot {
        ActiveHandsSlot::Zero => {
            hands_slot_0.process_input(my_id, inner_state, screen_effects, input, physic_system, audio_system, ui_system, engine_handle, delta);

            if let Some(device) = hands_slot_1 {
                device.process_while_deactive(my_id, inner_state, input, physic_system, audio_system, ui_system, engine_handle, delta);
            }
            if let Some(device) = hands_slot_2 {
                device.process_while_deactive(my_id, inner_state, input, physic_system, audio_system, ui_system, engine_handle, delta);
            }
            if let Some(device) = hands_slot_3 {
                device.process_while_deactive(my_id, inner_state, input, physic_system, audio_system, ui_system, engine_handle, delta);
            }
        },
        ActiveHandsSlot::First => {
            if let Some(device) = hands_slot_1.as_mut() {
                device.process_input(my_id, inner_state, screen_effects, input, physic_system, audio_system, ui_system, engine_handle, delta);
            }

            hands_slot_0.process_while_deactive(my_id, inner_state, input, physic_system, audio_system, ui_system, engine_handle, delta);
            if let Some(device) = hands_slot_2 {
                device.process_while_deactive(my_id, inner_state, input, physic_system, audio_system, ui_system, engine_handle, delta);
            }
            if let Some(device) = hands_slot_3 {
                device.process_while_deactive(my_id, inner_state, input, physic_system, audio_system, ui_system, engine_handle, delta);
            }
        },
        ActiveHandsSlot::Second => {
            if let Some(device) = hands_slot_2.as_mut() {
                device.process_input(my_id, inner_state, screen_effects, input, physic_system, audio_system, ui_system, engine_handle, delta);
            }

            hands_slot_0.process_while_deactive(my_id, inner_state, input, physic_system, audio_system, ui_system, engine_handle, delta);
            if let Some(device) = hands_slot_1 {
                device.process_while_deactive(my_id, inner_state, input, physic_system, audio_system, ui_system, engine_handle, delta);
            }
            if let Some(device) = hands_slot_3 {
                device.process_while_deactive(my_id, inner_state, input, physic_system, audio_system, ui_system, engine_handle, delta);
            }
        },
        ActiveHandsSlot::Third => {
            if let Some(device) = hands_slot_3.as_mut() {
                device.process_input(my_id, inner_state, screen_effects, input, physic_system, audio_system, ui_system, engine_handle, delta);
            }

            hands_slot_0.process_while_deactive(my_id, inner_state, input, physic_system, audio_system, ui_system, engine_handle, delta);
            if let Some(device) = hands_slot_1 {
                device.process_while_deactive(my_id, inner_state, input, physic_system, audio_system, ui_system, engine_handle, delta);
            }
            if let Some(device) = hands_slot_2 {
                device.process_while_deactive(my_id, inner_state, input, physic_system, audio_system, ui_system, engine_handle, delta);
            }
        }
    }

    for device in devices.iter_mut() {
        if let Some(device) = device {
            device.process_input(my_id, inner_state, screen_effects, input, physic_system, audio_system, ui_system, engine_handle, delta);
        }
    }
}


pub fn process_switch_active_hand_slot_input
(
    active_hands_slot: &mut ActiveHandsSlot,
    hands_slot_0: &mut Box<dyn Device>,
    hands_slot_1: &mut Option<Box<dyn Device>>,
    hands_slot_2: &mut Option<Box<dyn Device>>,
    hands_slot_3: &mut Option<Box<dyn Device>>,
    inner_state: &mut PlayerInnerState,
    screen_effects: &mut PlayerScreenEffects,
    input: &ActionsFrameState,
    my_id: ActorID,
    physic_system: &PhysicsSystem,
    audio_system: &mut AudioSystem,
    ui_system: &mut UISystem,
    engine_handle: &mut EngineHandle,
)
{
    if input.activate_hand_slot_0.is_action_just_pressed() {

        match active_hands_slot
        {
            ActiveHandsSlot::Zero => {},
            _ => {
                audio_system.spawn_non_spatial_sound(
                    Sound::SwitchWeapon,
                    0.35,
                    1.0,
                    false,
                    true,
                    Status::Playing
                );

                inner_state.crosshair_target_rotation = CROSSHAIR_CHANGE_WEAPON_TARGET_ROTATION;
                
                deavctivate_previous_device(
                    ActiveHandsSlot::Zero,
                    active_hands_slot,
                    hands_slot_0,
                    hands_slot_1,
                    hands_slot_2,
                    hands_slot_3,
                    inner_state,
                    screen_effects,
                    my_id,
                    physic_system,
                    audio_system,
                    ui_system,
                    engine_handle,
                );
                *active_hands_slot = ActiveHandsSlot::Zero;
        
                hands_slot_0.activate(
                    my_id,
                    inner_state,
                    physic_system,
                    audio_system,
                    ui_system,
                    engine_handle,
                );
            }
        }
    }

    if input.activate_hand_slot_1.is_action_just_pressed() {

        match active_hands_slot
        {
            ActiveHandsSlot::First => {},
            _ => {
                audio_system.spawn_non_spatial_sound(
                    Sound::SwitchWeapon,
                    0.35,
                    1.0,
                    false,
                    true,
                    Status::Playing
                );

                inner_state.crosshair_target_rotation = CROSSHAIR_CHANGE_WEAPON_TARGET_ROTATION;
        
                if hands_slot_1.is_some() {
                    deavctivate_previous_device(
                        ActiveHandsSlot::First,
                        active_hands_slot,
                        hands_slot_0,
                        hands_slot_1,
                        hands_slot_2,
                        hands_slot_3,
                        inner_state,
                        screen_effects,
                        my_id,
                        physic_system,
                        audio_system,
                        ui_system,
                        engine_handle,
                    );
                    *active_hands_slot = ActiveHandsSlot::First;
        
                    hands_slot_1.as_mut().unwrap().activate(
                        my_id,
                        inner_state,
                        physic_system,
                        audio_system,
                        ui_system,
                        engine_handle,
                    );
                }
            }
        }
    }

    if input.activate_hand_slot_2.is_action_just_pressed() {

        match active_hands_slot
        {
            ActiveHandsSlot::Second => {},
            _ => {
                audio_system.spawn_non_spatial_sound(
                    Sound::SwitchWeapon,
                    0.35,
                    1.0,
                    false,
                    true,
                    Status::Playing
                );

                inner_state.crosshair_target_rotation = CROSSHAIR_CHANGE_WEAPON_TARGET_ROTATION;
        
                if hands_slot_2.is_some() {
                    deavctivate_previous_device(
                        ActiveHandsSlot::Second,
                        active_hands_slot,
                        hands_slot_0,
                        hands_slot_1,
                        hands_slot_2,
                        hands_slot_3,
                        inner_state,
                        screen_effects,
                        my_id,
                        physic_system,
                        audio_system,
                        ui_system,
                        engine_handle,
                    );
                    *active_hands_slot = ActiveHandsSlot::Second;
        
                    hands_slot_2.as_mut().unwrap().activate(
                        my_id,
                        inner_state,
                        physic_system,
                        audio_system,
                        ui_system,
                        engine_handle,
                    );
                }
            }
        }
    }

    if input.activate_hand_slot_3.is_action_just_pressed() {

        match active_hands_slot
        {
            ActiveHandsSlot::Third => {},
            _ => {
                audio_system.spawn_non_spatial_sound(
                    Sound::SwitchWeapon,
                    0.35,
                    1.0,
                    false,
                    true,
                    Status::Playing
                );

                inner_state.crosshair_target_rotation = CROSSHAIR_CHANGE_WEAPON_TARGET_ROTATION;
        
                if hands_slot_3.is_some() {
                    deavctivate_previous_device(
                        ActiveHandsSlot::Third,
                        active_hands_slot,
                        hands_slot_0,
                        hands_slot_1,
                        hands_slot_2,
                        hands_slot_3,
                        inner_state,
                        screen_effects,
                        my_id,
                        physic_system,
                        audio_system,
                        ui_system,
                        engine_handle,
                    );
                    *active_hands_slot = ActiveHandsSlot::Third;
        
                    hands_slot_3.as_mut().unwrap().activate(
                        my_id,
                        inner_state,
                        physic_system,
                        audio_system,
                        ui_system,
                        engine_handle,
                    );
                }
            }
        }
    }
}


pub fn deavctivate_previous_device
(
    new_active_slot: ActiveHandsSlot,
    active_hands_slot: &ActiveHandsSlot,
    hands_slot_0: &mut Box<dyn Device>,
    hands_slot_1: &mut Option<Box<dyn Device>>,
    hands_slot_2: &mut Option<Box<dyn Device>>,
    hands_slot_3: &mut Option<Box<dyn Device>>,
    inner_state: &mut PlayerInnerState,
    screen_effects: &mut PlayerScreenEffects,
    my_id: ActorID,
    physic_system: &PhysicsSystem,
    audio_system: &mut AudioSystem,
    ui_system: &mut UISystem,
    engine_handle: &mut EngineHandle,
)
{
    match active_hands_slot {
        ActiveHandsSlot::Zero => {
            if new_active_slot != ActiveHandsSlot::Zero {
                hands_slot_0
                    .deactivate(
                        my_id,
                        inner_state,
                        physic_system,
                        audio_system,
                        ui_system,
                        engine_handle,
                        screen_effects,
                    );
            }
        },
        ActiveHandsSlot::First => {
            if new_active_slot != ActiveHandsSlot::First {
                hands_slot_1.as_mut().expect("Player have not any device in active hand's slot")
                    .deactivate(
                        my_id,
                        inner_state,
                        physic_system,
                        audio_system,
                        ui_system,
                        engine_handle,
                        screen_effects,
                    );
            }
        }
        ActiveHandsSlot::Second => {
            if new_active_slot != ActiveHandsSlot::Second {
                hands_slot_2.as_mut().expect("Player have not any device in active hand's slot")
                    .deactivate(
                        my_id,
                        inner_state,
                        physic_system,
                        audio_system,
                        ui_system,
                        engine_handle,
                        screen_effects,
                    );
            }
        }
        ActiveHandsSlot::Third => {
            if new_active_slot != ActiveHandsSlot::Third {
                hands_slot_3.as_mut().expect("Player have not any device in active hand's slot")
                    .deactivate(
                        my_id,
                        inner_state,
                        physic_system,
                        audio_system,
                        ui_system,
                        engine_handle,
                        screen_effects,
                    );
            }
        }
    }
}


pub fn get_effected_by_base
(
    inner_state: &mut PlayerInnerState,
    active_hands_slot: &ActiveHandsSlot,
    hands_slot_0: &mut Box<dyn Device>,
    hands_slot_1: &mut Option<Box<dyn Device>>,
    hands_slot_2: &mut Option<Box<dyn Device>>,
    hands_slot_3: &mut Option<Box<dyn Device>>,
    w_scanner: &mut WScanner,
    screen_effects: &mut PlayerScreenEffects,
    devices: &mut [Option<Box<dyn Device>>;4],
    my_id: ActorID,
    player_settings: &PlayerSettings,
    physic_system: &PhysicsSystem,
    audio_system: &mut AudioSystem,
    ui_system: &mut UISystem,
    engine_handle: &mut EngineHandle,
    delta: f32,
)
{
    let base_coef = 
    {
        let w_pos = inner_state.get_position().w;

        let mut coef = f32::clamp(
            (w_pos - inner_state.blue_map_w_level) /
            (inner_state.red_map_w_level - inner_state.blue_map_w_level),
                0.0,
                1.0
        );

        if inner_state.team == Team::Blue
        {
            coef = 1.0 - coef;
        }

        coef = (coef * 2.0) - 1.0;

        coef.max(0.0)
    };

    inner_state.hp += BASE_EFFECT_HP_IMPACT_SPEED * delta * base_coef;

    if inner_state.hp > PLAYER_MAX_HP
    {
        inner_state.hp = PLAYER_MAX_HP;
    }

    let health_bar = match inner_state.team {
        Team::Red => ui_system.get_mut_ui_element(&UIElementType::HeathBarRed), 
        Team::Blue => ui_system.get_mut_ui_element(&UIElementType::HeathBarBlue), 
    };

    if let UIElement::ProgressBar(bar) = health_bar {
        let bar_value = {
            (inner_state.hp as f32 / PLAYER_MAX_HP as f32)
                .clamp(0.0, 1.0)
        };

        bar.set_bar_value(bar_value)

    } else {
        panic!("Health Bar is not Progress Bar")
    }

    if inner_state.hp <= 0.0
    {
        die
        (
            inner_state,
            active_hands_slot,
            hands_slot_0,
            hands_slot_1,
            hands_slot_2,
            hands_slot_3,
            w_scanner,
            screen_effects,
            devices,
            my_id,
            player_settings,
            physic_system,
            audio_system,
            ui_system,
            engine_handle,
        );
    }
}


pub fn check_if_touching_death_plane(
    inner_state: &mut PlayerInnerState,
    active_hands_slot: &ActiveHandsSlot,
    hands_slot_0: &mut Box<dyn Device>,
    hands_slot_1: &mut Option<Box<dyn Device>>,
    hands_slot_2: &mut Option<Box<dyn Device>>,
    hands_slot_3: &mut Option<Box<dyn Device>>,
    w_scanner: &mut WScanner,
    screen_effects: &mut PlayerScreenEffects,
    devices: &mut [Option<Box<dyn Device>>;4],
    my_id: ActorID,
    player_settings: &PlayerSettings,
    physic_system: &PhysicsSystem,
    audio_system: &mut AudioSystem,
    ui_system: &mut UISystem,
    engine_handle: &mut EngineHandle,
)
{
    if inner_state.get_position().y < Y_DEATH_PLANE_LEVEL {
        die(
            inner_state,
            active_hands_slot,
            hands_slot_0,
            hands_slot_1,
            hands_slot_2,
            hands_slot_3,
            w_scanner,
            screen_effects,
            devices,
            my_id,
            player_settings,
            physic_system,
            audio_system,
            ui_system,
            engine_handle,
        );
    }
}



pub fn die
(
    inner_state: &mut PlayerInnerState,
    active_hands_slot: &ActiveHandsSlot,
    hands_slot_0: &mut Box<dyn Device>,
    hands_slot_1: &mut Option<Box<dyn Device>>,
    hands_slot_2: &mut Option<Box<dyn Device>>,
    hands_slot_3: &mut Option<Box<dyn Device>>,
    w_scanner: &mut WScanner,
    screen_effects: &mut PlayerScreenEffects,
    devices: &mut [Option<Box<dyn Device>>;4],
    my_id: ActorID,
    player_settings: &PlayerSettings,
    physic_system: &PhysicsSystem,
    audio_system: &mut AudioSystem,
    ui_system: &mut UISystem,
    engine_handle: &mut EngineHandle,
)
{
    match active_hands_slot {
        ActiveHandsSlot::Zero => {
            hands_slot_0.deactivate(
                my_id,
                inner_state,
                physic_system,
                audio_system,
                ui_system,
                engine_handle,
                screen_effects,
            );

        },
        ActiveHandsSlot::First => {
            if let Some(device) = hands_slot_1.as_mut() {
                device.deactivate(
                    my_id,
                    inner_state,
                    physic_system,
                    audio_system,
                    ui_system,
                    engine_handle,
                    screen_effects,
                );
            }

        },
        ActiveHandsSlot::Second => {
            if let Some(device) = hands_slot_2.as_mut() {
                device.deactivate(
                    my_id,
                    inner_state,
                    physic_system,
                    audio_system,
                    ui_system,
                    engine_handle,
                    screen_effects,
                );
            }

        },
        ActiveHandsSlot::Third => {
            if let Some(device) = hands_slot_3.as_mut() {
                device.deactivate(
                    my_id,
                    inner_state,
                    physic_system,
                    audio_system,
                    ui_system,
                    engine_handle,
                    screen_effects,
                );
            }

        }
    }

    let in_space = {
        inner_state.get_position().y < Y_DEATH_PLANE_LEVEL+1.0
    };

    engine_handle.send_boardcast_message(
        Message {
            from: my_id,
            remote_sender: false,
            message: MessageType::SpecificActorMessage(
                SpecificActorMessage::FlagMessage(
                    FlagMessage::PlayerDied(in_space)
                )
            )
        }
    );

    for device in devices.iter_mut() {
        if let Some(device) = device {
            device.deactivate(
                my_id,
                inner_state,
                physic_system,
                audio_system,
                ui_system,
                engine_handle,
                screen_effects,
            );
        }
    }

    w_scanner.restore_scanner_values(player_settings);
    inner_state.restore_w_shift_and_rotate_values();
    screen_effects.player_projections.clear();

    if inner_state.is_alive {

        inner_state.is_alive = false;
        inner_state.is_enable = false;
        inner_state.after_death_timer = 0.0;

        audio_system.spawn_non_spatial_sound(
            Sound::PlayerDied,
            0.37,
            1.0,
            false,
            true,
            fyrox_sound::source::Status::Playing
        );

        let players_death_explode = PlayersDeathExplosion::new(
            inner_state.transform.get_position()
        );

        inner_state.screen_effects.death_screen_effect = 0.0;
        inner_state.screen_effects.getting_damage_screen_effect = 1.0;

        engine_handle.send_command(
            Command {
                sender: my_id,
                command_type: CommandType::SpawnActor(
                    super::ActorWrapper::PlayersDeathExplosion(players_death_explode)
                )
            }
        );

        engine_handle.send_command(
            Command {
                sender: my_id,
                command_type: CommandType::NetCommand(
                    NetCommand::SendBoardcastNetMessageReliable(
                        NetMessageToPlayer::RemoteDirectMessage(
                            my_id,
                            RemoteMessage::DieImmediately
                        )
                    )
                )
            }
        );
    }
}


pub fn telefraged
(
    inner_state: &mut PlayerInnerState,
    active_hands_slot: &ActiveHandsSlot,
    hands_slot_0: &mut Box<dyn Device>,
    hands_slot_1: &mut Option<Box<dyn Device>>,
    hands_slot_2: &mut Option<Box<dyn Device>>,
    hands_slot_3: &mut Option<Box<dyn Device>>,
    w_scanner: &mut WScanner,
    screen_effects: &mut PlayerScreenEffects,
    devices: &mut [Option<Box<dyn Device>>;4],
    my_id: ActorID,
    player_settings: &PlayerSettings,
    physic_system: &PhysicsSystem,
    audio_system: &mut AudioSystem,
    ui_system: &mut UISystem,
    engine_handle: &mut EngineHandle,
)
{
    die
    (
        inner_state,
        active_hands_slot,
        hands_slot_0,
        hands_slot_1,
        hands_slot_2,
        hands_slot_3,
        w_scanner,
        screen_effects,
        devices,
        my_id,
        player_settings,
        physic_system,
        audio_system,
        ui_system,
        engine_handle,
    );
}


pub fn make_hud_transparency_as_death_screen_effect(
    screen_effects: &PlayerScreenEffects,
    inner_state: &PlayerInnerState,
    ui: &mut UISystem
) {
    let a = 1.0 - screen_effects.death_screen_effect.clamp(0.0, 1.0);

    let hud_elem = ui.get_mut_ui_element(&UIElementType::WAimFrame);
    hud_elem.get_ui_data_mut().rect.transparency = inner_state.w_aim_ui_frame_intensity * a;

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

    let hud_elem = ui.get_mut_ui_element(&UIElementType::ScannerHPointer);
    hud_elem.get_ui_data_mut().rect.transparency = a;

    let hud_elem = ui.get_mut_ui_element(&UIElementType::ZWScannerArrow);
    hud_elem.get_ui_data_mut().rect.transparency = a;

    let hud_elem = ui.get_mut_ui_element(&UIElementType::ZXScannerArrow);
    hud_elem.get_ui_data_mut().rect.transparency = a;

    
    match inner_state.team
    {
        Team::Red =>
        {
            let hud_elem = ui.get_mut_ui_element(&UIElementType::ScannerRed);
            hud_elem.get_ui_data_mut().rect.transparency = a;

            hud_elem.get_ui_data_mut().rect.transparency = a;

            let hud_elem = ui.get_mut_ui_element(&UIElementType::HeathBarRed);
            hud_elem.get_ui_data_mut().rect.transparency = a;

            let hud_elem = ui.get_mut_ui_element(&UIElementType::EnergyGunBarRed);
            hud_elem.get_ui_data_mut().rect.transparency = a;
            
            let hud_elem = ui.get_mut_ui_element(&UIElementType::MachinegunBarRed);
            hud_elem.get_ui_data_mut().rect.transparency = a;

            let hud_elem = ui.get_mut_ui_element(&UIElementType::ShotgunBarRed);
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

            hud_elem.get_ui_data_mut().rect.transparency = a;

            let hud_elem = ui.get_mut_ui_element(&UIElementType::HeathBarBlue);
            hud_elem.get_ui_data_mut().rect.transparency = a;

            let hud_elem = ui.get_mut_ui_element(&UIElementType::EnergyGunBarBlue);
            hud_elem.get_ui_data_mut().rect.transparency = a;
            
            let hud_elem = ui.get_mut_ui_element(&UIElementType::MachinegunBarBlue);
            hud_elem.get_ui_data_mut().rect.transparency = a;

            let hud_elem = ui.get_mut_ui_element(&UIElementType::ShotgunBarBlue);
            hud_elem.get_ui_data_mut().rect.transparency = a;

            let hud_elem = ui.get_mut_ui_element(&UIElementType::LeftScannerDsiplayBlue);
            hud_elem.get_ui_data_mut().rect.transparency = a;

            let hud_elem = ui.get_mut_ui_element(&UIElementType::RightScannerDsiplayBlue);
            hud_elem.get_ui_data_mut().rect.transparency = a;
        }
    }
}


pub fn set_right_team_hud(
    inner_state: &PlayerInnerState,
    ui: &mut UISystem
)
{
    let hud_elem = ui.get_mut_ui_element(&UIElementType::Crosshair);
    *hud_elem.get_ui_data_mut().get_is_visible_cloned_arc().lock().unwrap() = true;

    let hud_elem = ui.get_mut_ui_element(&UIElementType::ScoreBar);
    *hud_elem.get_ui_data_mut().get_is_visible_cloned_arc().lock().unwrap() = true;

    let hud_elem = ui.get_mut_ui_element(&UIElementType::ScannerHPointer);
    *hud_elem.get_ui_data_mut().get_is_visible_cloned_arc().lock().unwrap() = true;

    let hud_elem = ui.get_mut_ui_element(&UIElementType::ZWScannerArrow);
    *hud_elem.get_ui_data_mut().get_is_visible_cloned_arc().lock().unwrap() = true;

    let hud_elem = ui.get_mut_ui_element(&UIElementType::ZXScannerArrow);
    *hud_elem.get_ui_data_mut().get_is_visible_cloned_arc().lock().unwrap() = true;


    match inner_state.team
    {
        Team::Red =>
        {
            let hud_elem = ui.get_mut_ui_element(&UIElementType::ScannerRed);
            *hud_elem.get_ui_data_mut().get_is_visible_cloned_arc().lock().unwrap() = true;

            let hud_elem = ui.get_mut_ui_element(&UIElementType::HeathBarRed);
            *hud_elem.get_ui_data_mut().get_is_visible_cloned_arc().lock().unwrap() = true;

            let hud_elem = ui.get_mut_ui_element(&UIElementType::LeftScannerDsiplayRed);
            *hud_elem.get_ui_data_mut().get_is_visible_cloned_arc().lock().unwrap() = true;

            let hud_elem = ui.get_mut_ui_element(&UIElementType::RightScannerDsiplayRed);
            *hud_elem.get_ui_data_mut().get_is_visible_cloned_arc().lock().unwrap() = true;

            let hud_elem = ui.get_mut_ui_element(&UIElementType::ScannerBlue);
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

            *hud_elem.get_ui_data_mut().get_is_visible_cloned_arc().lock().unwrap() = false;

            let hud_elem = ui.get_mut_ui_element(&UIElementType::HeathBarRed);
            *hud_elem.get_ui_data_mut().get_is_visible_cloned_arc().lock().unwrap() = false;

            let hud_elem = ui.get_mut_ui_element(&UIElementType::LeftScannerDsiplayRed);
            *hud_elem.get_ui_data_mut().get_is_visible_cloned_arc().lock().unwrap() = false;

            let hud_elem = ui.get_mut_ui_element(&UIElementType::RightScannerDsiplayRed);
            *hud_elem.get_ui_data_mut().get_is_visible_cloned_arc().lock().unwrap() = false;

            let hud_elem = ui.get_mut_ui_element(&UIElementType::ScannerBlue);
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


pub fn get_damage_and_add_force(
    damage: i32,
    force: Vec4,
    screen_effects: &mut PlayerScreenEffects,
    inner_state: &mut PlayerInnerState,
    active_hands_slot: &ActiveHandsSlot,
    hands_slot_0: &mut Box<dyn Device>,
    hands_slot_1: &mut Option<Box<dyn Device>>,
    hands_slot_2: &mut Option<Box<dyn Device>>,
    hands_slot_3: &mut Option<Box<dyn Device>>,
    w_scanner: &mut WScanner,
    devices: &mut [Option<Box<dyn Device>>;4],
    my_id: ActorID,
    player_settings: &PlayerSettings,
    physic_system: &PhysicsSystem,
    audio_system: &mut AudioSystem,
    ui_system: &mut UISystem,
    engine_handle: &mut EngineHandle,
) {

    audio_system.spawn_non_spatial_sound(
        Sound::PlayerHited,
        0.4.lerp(0.6, (damage as f32/PLAYER_MAX_HP as f32).clamp(0.0, 1.0)),
        1.0,
        false,
        true,
        fyrox_sound::source::Status::Playing
    );

    screen_effects.getting_damage_screen_effect = 1.0;

    inner_state.hp -= damage as f32;
    inner_state.collider.add_force(force);

    let health_bar = match inner_state.team {
        Team::Red => ui_system.get_mut_ui_element(&UIElementType::HeathBarRed), 
        Team::Blue => ui_system.get_mut_ui_element(&UIElementType::HeathBarBlue), 
    };

    if let UIElement::ProgressBar(bar) = health_bar {
        let bar_value = {
            (inner_state.hp as f32 / PLAYER_MAX_HP as f32)
                .clamp(0.0, 1.0)
        };

        bar.set_bar_value(bar_value)

    } else {
        panic!("Health Bar is not Progress Bar")
    }

    if inner_state.hp <= 0.0 {
        die(
            inner_state,
            active_hands_slot,
            hands_slot_0,
            hands_slot_1,
            hands_slot_2,
            hands_slot_3,
            w_scanner,
            screen_effects,
            devices,
            my_id,
            player_settings,
            physic_system,
            audio_system,
            ui_system,
            engine_handle,
        );
    }
}


pub fn set_gun_to_slot(
    hands_slot: &mut Option<Box<dyn Device>>,
    device: Box<dyn Device>
) -> Option<Box<dyn Device>>
{
    match device.get_device_type() {
        DeviceType::Gun => {
            let prev_device = hands_slot.take();
            *hands_slot = Some(device);

            return prev_device;
        }
        _ => {
            Some(device)
        }
    }
}


pub fn put_device_into_device_slot(
    devices: &mut [Option<Box<dyn Device>>;4],
    slot_number: PlayersDeviceSlotNumber,
    device: Box<dyn Device>
) -> Option<Box<dyn Device>> {

    match device.get_device_type() {
        DeviceType::Device => {
            match slot_number {
                PlayersDeviceSlotNumber::First => {
                    let prev_device = devices[0].take();
                    devices[0] = Some(device);
                    prev_device
                }
                PlayersDeviceSlotNumber::Second => {
                    let prev_device = devices[1].take();
                    devices[1] = Some(device);
                    prev_device
                }
                PlayersDeviceSlotNumber::Third => {
                    let prev_device = devices[2].take();
                    devices[2] = Some(device);
                    prev_device
                }
                PlayersDeviceSlotNumber::Fourth => {
                    let prev_device = devices[3].take();
                    devices[3] = Some(device);
                    prev_device
                }
            }
        },
        _ => {Some(device)}
    }
}



impl MainPlayer {

    pub fn new(
        master: InputMaster,
        player_settings: PlayerSettings,
        audio_system: &mut AudioSystem,
        w_levels_of_map: Vec<f32>
    ) -> Self {

        assert!(w_levels_of_map.len() > 1);

        let blue_map_w_level = w_levels_of_map[0];

        let red_map_w_level = *w_levels_of_map.last().unwrap();
        
        let screen_effects = PlayerScreenEffects::default();

        let w_scanner = WScanner::new(&player_settings);
        
        MainPlayer {
            id: None,

            inner_state: PlayerInnerState::new(
                Transform::new(),
                &player_settings,
                false,
                false,
                blue_map_w_level,
                red_map_w_level,
                Vec4::X*0.6,
                audio_system,
            ),
            active_hands_slot: ActiveHandsSlot::Zero,

            hands_slot_0: Box::new(MachineGun::new(
                player_settings.machinegun_damage,
                player_settings.machinegun_add_force, 
                player_settings.machinegun_heat_add_on_shot, 
                player_settings.machinegun_cooling_speed,
                Vec4::new(
                    1.0,
                    -0.3,
                    -1.0,
                    0.0
                ),
            )),
            hands_slot_1: Some(Box::new(Shotgun::new(
                Vec4::new(
                    1.0,
                    -0.3,
                    -1.0,
                    0.0
                ),
            ))),
            hands_slot_2: Some(Box::new(HoleGun::new(
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
            hands_slot_3: None,

            devices: [None, None, None, None],
            
            player_settings,

            master,

            screen_effects,

            w_scanner,
        }
    }

    pub fn get_xz_rotation(&self) -> f32
    {
        self.inner_state.saved_angle_of_rotation.x
    }
}

impl ControlledActor for MainPlayer
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
        self.inner_state.hp = PLAYER_MAX_HP;
        self.inner_state.amount_of_move_w_bonuses_do_i_have = 0u32;
        // self.inner_state.player_moving_state =
        //     PlayerMovingState::MovingPerpendicularW(self.w_levels_of_map[current_spawn.w_level]);

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