use fyrox_core::pool::Handle;
use fyrox_sound::source::{SoundSource, Status};
use glam::{FloatExt, Vec3, Vec4};

use client_server_protocol::{
    NetCommand,
    NetMessageToPlayer,
    RemoteMessage,
    Team
};

use crate::{
    engine::{
        audio::{AudioSystem, Sound}, effects::EffectsSystem, engine_handle::{
            Command,
            CommandType,
            EngineHandle
        }, physics::{
            colliders_container::PhysicalElement,
            dynamic_collider::PlayersDollCollider,
            kinematic_collider::KinematicCollider,
            PhysicsSystem
        }, render::VisualElement, time::TimeSystem, ui::UISystem, world::static_object::{
            SphericalVolumeArea, VisualWave, VolumeArea
        }
    },
    transform::{Transform, BACKWARD, DOWN, FORWARD, LEFT, RIGHT, UP, W_DOWN},
};

use super::{
    device::holegun::{HOLE_GUN_BLUE_COLOR, HOLE_GUN_RED_COLOR}, flag::FlagMessage, holegun_miss::HoleGunMiss, holegun_shot::HoleGunShot, machinegun_shot::MachinegunShot, main_player::{
        player_settings::PlayerSettings, PlayerMessage, PlayerMovingState, BLUE_SCANNER_WAVE_COLOR, PLAYER_MAX_HP, RED_SCANNER_WAVE_COLOR, TIME_TO_DIE_SLOWLY, W_SCANNER_EXPANDING_SPEED, W_SCANNER_MAX_RADIUS
    }, mover_w::MoverWMessage, players_death_explosion::PlayersDeathExplosion, session_controller::SessionControllerMessage, shooting_impact::ShootingImpact, shotgun_shot_source::ShotgunShotSource, Actor, ActorID, ActorWrapper, CommonActorsMessage, Message, MessageType, SpecificActorMessage
};

#[derive(Clone)]
pub struct PlayerDollInputState
{
    pub move_forward: bool,
    pub move_backward: bool,
    pub move_right: bool,
    pub move_left: bool,
    pub will_jump: bool,
    // pub player_moving_state: PlayerMovingState
}


impl PlayerDollInputState {
    pub fn serialize(self) -> (bool,bool,bool,bool,bool)
    {
        (
            self.move_forward,
            self.move_backward,
            self.move_right,
            self.move_left,
            self.will_jump,
        )
    }

    pub fn deserialize(input: (bool,bool,bool,bool,bool)) -> Self
    {
        PlayerDollInputState
        {
            move_forward: input.0,
            move_backward: input.1,
            move_right: input.2,
            move_left: input.3,
            will_jump: input.4,
        }
    }
}


const PLAYERS_DOLL_COLOR: Vec3 = Vec3::new(0.8, 0.8, 0.8);
pub struct PlayersDoll {
    team: Team,
    id: Option<ActorID>,
    transform: Transform,
    target_transform: Transform,
    input_state: PlayerDollInputState,
    // masters_peer_id: PeerId,
    weapon_shooting_point: Vec4,
    is_alive: bool,

    volume_area: Vec<VolumeArea>,
    charging_time: f32,

    interpolating_model: Vec<PlayersDollCollider>,
    interpolating_model_target: KinematicCollider,
    prev_interpolating_model_set_target_time: u128,
    is_enable: bool,

    need_to_die_slowly: bool,
    die_slowly_timer: f32,

    holegun_charge_sound: Option<Handle<SoundSource>>,

    player_settings: PlayerSettings,
    radius: f32,
    my_color: Vec3,

    on_way_to_next_w_level: bool,
    current_w_level_prev_frame: u32,

    w_scanner_enable: bool,
    w_scanner_radius: f32,
    w_scanner_ring_intesity: f32,
    visual_wave: Vec<VisualWave>,
}

#[derive(Clone)]
pub enum PlayersDollMessage{
    ScannerTurnedOn,
    SpawnShotgunShot(
        // shot's start position
        Vec4,
        // shot's main direction
        Vec4,
        // random seed
        u64,
        //damage dealer's id
        u128,
        //damage dealer's team
        Team,
    ),
    SetInterploatedModelTargetState(
        // postition
        Transform,
        // player's input state for extrapolation reason
        PlayerDollInputState,
        // force for extrapolation
        Vec4,
        // timestamp in millis
        u128
    ),
    SpawnHoleGunShotActor(
        Vec4,
        f32,
        Vec3,
        f32
    ),
    SpawHoleGunMissActor(
        Vec4,
        f32,
        Vec3,
        f32
    ),
    SpawnMachineGunShot(
        Vec4,
        bool
    ),
    HoleGunStartCharging,
    Respawn(
        // postition
        Transform,
        // player's input state for extrapolation reason
        PlayerDollInputState,
        // force for extrapolation
        Vec4,
        // team of respawned player
        Team,
    ),
    SetNewTeamAndPosition(
        // team player joined to
        Team,
        //is_alive
        bool,
        // postition
        Transform,
        // player's input state for extrapolation reason
        PlayerDollInputState,
        // force for extrapolation
        Vec4,
        // timestamp in millis
        u128,
    ),
    YouHitedMe(
        //damage
        u32,
        //my position
        Vec4,
        //my radius
        f32,
    ),
}



const VISUAL_BEAM_MULT: f32 = 2.0;
const VISUAL_FIRE_SHPERE_MULT: f32 = 2.4;

impl PlayersDoll {
    pub fn new(
        id: ActorID,
        player_sphere_radius: f32,
        transform: Transform,
        is_alive: bool,
        audio_system: &mut AudioSystem,
        player_settings: PlayerSettings,
        team: Team,
    ) -> Self {

        let my_color = match team {
            Team::Red =>
            {
                Vec3::new(1.0, 0.0, 0.0)
            }

            Team::Blue =>
            {
                Vec3::new(0.0, 0.0, 1.0)
            }
        };

        let weapon_offset = {
            Vec4::new(
                1.0,
                0.26,
                0.0,
                0.0
            ).normalize() * (player_sphere_radius * 1.35)
        };

        let interpolated_model = {
            let mut vec = Vec::with_capacity(1);

            vec.push(PlayersDollCollider {
                position: Vec4::ZERO,
                radius: player_sphere_radius,
                friction: 0.0,
                bounce_rate: 0.0,
                actor_id: Some(id),
                weapon_offset,
                actors_team: team,
            });
            vec
        };

        let mut interpolated_model_target = KinematicCollider::new(
            player_settings.max_speed,
            player_settings.max_accel,
            player_sphere_radius,
            player_settings.friction_on_air
        );

        let weapon_shooting_point = weapon_offset + FORWARD * (player_sphere_radius * 0.49);

        let input_state = PlayerDollInputState {
            move_forward: false,
            move_backward: false,
            move_right: false,
            move_left: false,
            will_jump: false,
            // player_moving_state: PlayerMovingState::MovingPerpendicularW(0.0),
        };

        PlayersDoll {
            input_state,
            current_w_level_prev_frame: 0u32,
            weapon_shooting_point,
            id: Some(id),
            target_transform: transform.clone(),
            transform,
            charging_time: 0.0,
            volume_area: Vec::with_capacity(1),
            is_alive,
            is_enable: is_alive,
            interpolating_model: interpolated_model,
            interpolating_model_target: interpolated_model_target,
            prev_interpolating_model_set_target_time: 0u128,
            need_to_die_slowly: false,
            die_slowly_timer: 0.0,
            holegun_charge_sound: None,
            player_settings,
            team,
            radius: player_sphere_radius,
            my_color,
            on_way_to_next_w_level: false,
            w_scanner_enable: false,
            w_scanner_radius: 0.0,
            w_scanner_ring_intesity: 0.0,
            visual_wave: Vec::with_capacity(1),
        }
    }



    fn die_immediately(&mut self, engine_handle: &mut EngineHandle, audio_system: &mut AudioSystem) {
        if self.is_alive {

            self.w_scanner_enable = false;
            self.visual_wave.clear();

            self.volume_area.clear();

            self.is_alive = false;
            self.is_enable = false;
            self.need_to_die_slowly = false;

            self.play_die_effects(engine_handle, audio_system);
        }
    }



    fn play_die_effects(&mut self, engine_handle: &mut EngineHandle, audio_system: &mut AudioSystem) {
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

        audio_system.spawn_spatial_sound(
            Sound::PlayerDeathSignal,
            0.8,
            1.3,
            false,
            true,
            fyrox_sound::source::Status::Playing,
            self.transform.get_position(),
            2.0,
            1.0,
            50.0
        );

        audio_system.spawn_non_spatial_sound(
            Sound::PlayerDeathSignal,
            1.0,
            1.2,
            false,
            true,
            fyrox_sound::source::Status::Playing,
        );

        if let Some(handle) = self.holegun_charge_sound.take() {
            audio_system.remove_sound(handle);
        }
    }



    fn die_slowly(&mut self, engine_handle: &mut EngineHandle) {
        if self.is_alive {

            self.volume_area.clear();

            self.is_alive = false;
            self.is_enable = true;
            self.need_to_die_slowly = true;
            self.die_slowly_timer = 0.0;
        }
    }



    fn respawn(
        &mut self,
        transform: Transform,
        input_state: PlayerDollInputState,
        velocity: Vec4,
        team: Team,
        physics_system: &PhysicsSystem,
        audio_system: &mut AudioSystem,
        engine_handle: &mut EngineHandle,
    ) {
        audio_system.spawn_spatial_sound(
            Sound::PlayerRespawned,
            1.0,
            1.0,
            false,
            true,
            fyrox_sound::source::Status::Playing,
            transform.get_position(),
            1.5,
            1.0,
            50.0
        );
        

        let collider_radius = self.interpolating_model[0].radius;

        let hits = physics_system.sphere_cast_on_dynamic_colliders(
            transform.get_position(),
            collider_radius,
            Some(self.get_id().expect("PlayerDoll have not ActorID"))
        );

        for hit in hits {
            engine_handle.send_direct_message(
                hit.hited_actors_id.expect("In respawn func in death on resapwn hit have not ActorID"),
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

        // self.current_w_level_prev_frame = input_state.current_w_level;
        self.team = team;
        self.on_way_to_next_w_level = false;
        self.is_alive = true;
        self.is_enable = true;
        self.transform = transform.clone();
        self.target_transform = transform;
        self.input_state = input_state;
        self.interpolating_model_target.current_velocity = velocity;
        self.w_scanner_enable = false;
        self.w_scanner_radius = 0.0;
        self.w_scanner_ring_intesity = 0.0;
        self.visual_wave.clear();
    }


    fn extrapolate_interpolatating_model_target(&mut self, delta: f32, audio_system: &mut AudioSystem)
    {
        let mut movement_vec = Vec4::ZERO;
        
        if self.input_state.move_forward { 
            movement_vec += FORWARD;
        }

        if self.input_state.move_backward {
            movement_vec += BACKWARD;
        }

        if self.input_state.move_right {
            movement_vec += RIGHT
        }

        if self.input_state.move_left {
            movement_vec += LEFT;
        }

        if self.input_state.will_jump {

            if self.interpolating_model_target.is_on_y_ground {
                self.interpolating_model_target.add_force(UP * self.player_settings.jump_y_speed);

                self.input_state.will_jump = false;
            }
        }

        movement_vec = self.target_transform.get_rotation() * movement_vec;

        movement_vec.y = 0.0;
        movement_vec.w = 0.0;

        match movement_vec.try_normalize()
        {
            Some(vec) => movement_vec = vec,
            None => movement_vec = Vec4::ZERO,
        }

        self.interpolating_model_target.add_force(DOWN * self.player_settings.gravity_y_speed * delta);

        self.interpolating_model_target.add_force(W_DOWN * self.player_settings.gravity_w_speed * delta);

        if self.interpolating_model_target.is_on_y_ground {
            self.interpolating_model_target.set_wish_direction(
                movement_vec,
                1.0
            );
        } else {
            self.interpolating_model_target.set_wish_direction(
                movement_vec,
                self.player_settings.air_speed_mult
            );
        }

        self.interpolating_model_target.set_friction_on_air(
            self.player_settings.friction_on_air
        );
    }


    fn interpolate_model(&mut self, delta: f32)
    {
        let dist = self.target_transform.get_position() - self.transform.get_position();

        self.transform.set_position(self.transform.get_position() + dist * (18_f32*delta));
        // self.transform.set_position(self.target_transform.get_position());
    }

    fn process_player_doll_w_scanner(
        &mut self,
        delta: f32,
    )
    {   
        if self.w_scanner_enable {
            self.w_scanner_radius += delta * W_SCANNER_EXPANDING_SPEED;

            self.w_scanner_ring_intesity = {
                let mut intensity = W_SCANNER_MAX_RADIUS - self.w_scanner_radius;
        
                intensity /= W_SCANNER_MAX_RADIUS/3.0;
        
                intensity.clamp(0.0, 1.0)
            };
    
            self.visual_wave[0].radius = self.w_scanner_radius;
    
            self.visual_wave[0].color = match self.team
            {
                Team::Blue => {
                    BLUE_SCANNER_WAVE_COLOR * self.w_scanner_ring_intesity
                },
                Team::Red =>
                {
                    RED_SCANNER_WAVE_COLOR * self.w_scanner_ring_intesity
                }
            };
    
            if self.w_scanner_radius >= W_SCANNER_MAX_RADIUS {
    
                self.visual_wave.clear();
    
                self.w_scanner_enable = false;
            }
        }
    }
}



impl Actor for PlayersDoll {
    fn recieve_message(
        &mut self,
        message: Message,
        engine_handle: &mut EngineHandle,
        physics_system: &PhysicsSystem,
        audio_system: &mut AudioSystem,
        ui_system: &mut UISystem,
        time_system: &TimeSystem,
        effects_system: &mut EffectsSystem,
    ) {
        let from = message.from;

        let message = message.message;
        
        match message {
            MessageType::CommonActorsMessages(message) => {
                match message {
                    CommonActorsMessage::SetTransform(transform) => {
                        self.transform = transform.clone();
                    },
                    CommonActorsMessage::Enable(switch) => {
                        self.is_enable = switch;
                    },

                    CommonActorsMessage::IncrementPosition(increment) => {
                        self.transform.increment_position(increment);
                    },
                    CommonActorsMessage::IWasChangedMyId(new_id) => {}
                }
            }
            MessageType::PhysicsMessages(message) => {
                match message {
                    _ => {}
                }
            },
            MessageType::SpecificActorMessage(message) => {
                match message
                {
                    SpecificActorMessage::MoverW(message) =>
                    {
                        match message {
                            MoverWMessage::Rotate(_,_,_) =>
                            {
                                audio_system.spawn_spatial_sound(
                                    Sound::WShiftEnd,
                                    0.7,
                                    1.0,
                                    false,
                                    true,
                                    fyrox_sound::source::Status::Playing,
                                    self.transform.get_position(),
                                    self.radius,
                                    1.0,
                                    50.0,
                                );
                            }
                        }
                    }

                    SpecificActorMessage::SessionControllerMessage(message) =>
                    {
                        match message
                        {
                            SessionControllerMessage::JoinedToSession(_,_,_,_,_,_) =>
                            {
                                self.prev_interpolating_model_set_target_time = 0u128;
                            }

                            SessionControllerMessage::NewSessionStarted(_) =>
                            {
                                // self.prev_interpolating_model_set_target_time = 0u128;
                            }

                            _ => {}
                        }
                    }

                    SpecificActorMessage::PlayerMessage(message) => 
                    {
                        match message
                        {
                            PlayerMessage::YouWasScanned => {},

                            PlayerMessage::GiveMeDataForProjection => {
                                if self.is_alive
                                {
                                    engine_handle.send_direct_message(
                                        from,
                                        Message {
                                            from: self.get_id().expect("Player Doll have not ActorID"),
                                            remote_sender: false,
                                            message: MessageType::SpecificActorMessage(
                                                SpecificActorMessage::PlayerMessage(
                                                    PlayerMessage::DataForProjection(
                                                        self.transform.get_position(),
                                                        self.player_settings.collider_radius
                                                    )
                                                )
                                            )
                                        }
                                    );
                                }
                            }

                            PlayerMessage::DataForProjection(_,_) => {}

                            PlayerMessage::Telefrag =>
                            {
                                self.die_immediately(engine_handle, audio_system);

                                engine_handle.send_command(
                                    Command {
                                        sender: self.id.expect("Player's Doll have not Actor's ID"),
                                        command_type: CommandType::NetCommand(
                                            NetCommand::SendDirectNetMessageReliable(
                                                NetMessageToPlayer::RemoteDirectMessage(
                                                    self.id.expect("Player's Doll have not Actor's ID"),
                                                    RemoteMessage::DieImmediately
                                                ),
                                                self.id.unwrap()
                                            )
                                        )
                                    }
                                )
                            }

                            PlayerMessage::DieImmediately =>
                            {
                                self.die_immediately(engine_handle, audio_system);
                            }

                            PlayerMessage::DieSlowly =>
                            {
                                self.die_slowly(engine_handle);
                            }

                            PlayerMessage::GetDamageAndForce(
                                damage,
                                force,
                                impact_pos,
                                team,
                                damage_dealer_id,
                            ) =>
                            {
                                if team != self.team && damage > 0
                                {
                                    engine_handle.send_command(
                                        Command {
                                            sender: self.id.expect("Player's Doll have not Actor's ID"),
                                            command_type: CommandType::NetCommand(
                                                NetCommand::SendDirectNetMessageReliable(
                                                    NetMessageToPlayer::RemoteDirectMessage(
                                                        self.id.expect("Player's Doll have not Actor's ID"),
                                                        RemoteMessage::DealDamageAndForce(
                                                            damage,
                                                            force.to_array(),
                                                            impact_pos.to_array(),
                                                            team
                                                        )
                                                    ),
                                                    self.id.unwrap()
                                                )
                                            )
                                        }
                                    );
    
                                    engine_handle.send_command(Command {
                                        sender: 0u128,
                                        command_type: CommandType::SpawnActor(
                                            ActorWrapper::ShootingImpact(
                                                ShootingImpact::new(impact_pos, damage)
                                            )
                                        )
                                    });
    
                                    engine_handle.send_direct_message(
                                        damage_dealer_id,
                                        Message {
                                            from: self.get_id().expect("Player Doll have not ActorID"),
                                            remote_sender: false,
                                            message: MessageType::SpecificActorMessage(
                                                SpecificActorMessage::PlayersDollMessage(
                                                    PlayersDollMessage::YouHitedMe(
                                                        damage,
                                                        self.transform.get_position(),
                                                        self.radius
                                                    )
                                                )
                                            )
                                        }
                                    );
                                    
                                    audio_system.spawn_non_spatial_sound(
                                        Sound::PlayerHitSignal,
                                        0.14.lerp(0.22, (damage as f32 / PLAYER_MAX_HP as f32).clamp(0.0, 1.0)),
                                        1.0,
                                        false,
                                        true,
                                        fyrox_sound::source::Status::Playing
                                    );                         
                                }
                            }

                            PlayerMessage::SetNewTeam(team) =>
                            {
                                self.team = team;
                            }

                            PlayerMessage::NewPeerConnected(_) => {}
                        }
                    },

                    SpecificActorMessage::PlayersDollMessage(message) =>
                    {
                        match message
                        {
                            PlayersDollMessage::ScannerTurnedOn =>
                            {
                                self.w_scanner_enable = true;
                
                                self.w_scanner_radius = self.player_settings.collider_radius + 0.1;

                                self.visual_wave.push(
                                    VisualWave {
                                        translation: Vec4::ZERO,
                                        radius: 0.001,
                                        color: match self.team {
                                            Team::Blue => BLUE_SCANNER_WAVE_COLOR,
                                            Team::Red => RED_SCANNER_WAVE_COLOR,
                                        }
                                    }
                                );

                                audio_system.spawn_spatial_sound(
                                    crate::engine::audio::Sound::ScannerSound,
                                    0.7,
                                    1.0,
                                    false,
                                    true,
                                    fyrox_sound::source::Status::Playing,
                                    self.transform.get_position(),
                                    1.0,
                                    1.0,
                                    50.0
                                );
                            },

                            PlayersDollMessage::SpawnShotgunShot(
                                start_pos,
                                shot_dir ,
                                rng_seed,
                                damage_dealer_id,
                                damage_dealer_team,
                            ) =>
                            {
                                let shooted_from = self.transform.get_position() + self.transform.get_rotation() * self.weapon_shooting_point;
                        
                                let shotgun_shot_source = ShotgunShotSource::new(
                                    start_pos,
                                    shooted_from,
                                    shot_dir,
                                    rng_seed,
                                    true,
                                    damage_dealer_id,
                                    damage_dealer_team,
                                    1.25,
                                    engine_handle,
                                    physics_system,
                                    audio_system,
                                );

                                engine_handle.send_command(Command {
                                    sender: 0u128,
                                    command_type: CommandType::SpawnActor(
                                        ActorWrapper::ShotgunShotSource(shotgun_shot_source)
                                    )
                                })
                            },

                            PlayersDollMessage::YouHitedMe(_,_,_) => {}

                            PlayersDollMessage::SetInterploatedModelTargetState(
                                transform,
                                input,
                                velocity,
                                time,
                            ) =>
                            {
                                if self.prev_interpolating_model_set_target_time < time
                                {
                                    self.prev_interpolating_model_set_target_time = time;
                                    self.transform.set_rotation(transform.get_rotation());
                                    self.target_transform = transform.clone();
                                    self.input_state = input.clone();
                                    self.interpolating_model_target.current_velocity = velocity;
                                    self.interpolating_model_target.forces.clear();
                                }
                            }

                            PlayersDollMessage::HoleGunStartCharging =>
                            {
                                if self.volume_area.is_empty() {

                                    let color = match self.team {
                                        Team::Red => HOLE_GUN_RED_COLOR,
                                        Team::Blue => HOLE_GUN_BLUE_COLOR,
                                    };

                                    let volume_area = VolumeArea::SphericalVolumeArea(
                                        SphericalVolumeArea {
                                            color,
                                            translation: self.transform.get_rotation() * self.weapon_shooting_point,
                                            radius: 0.1 * VISUAL_FIRE_SHPERE_MULT,
                                        }
                                    );
                    
                                    self.volume_area.push(volume_area);

                                    self.holegun_charge_sound = Some(audio_system.spawn_spatial_sound(
                                        Sound::HolegunCharging,
                                        0.08,
                                        1.0,
                                        false,
                                        true,
                                        fyrox_sound::source::Status::Playing,
                                        self.transform.get_position(),
                                        1.0,
                                        1.0,
                                        50.0
                                    ));

                                }
                            }

                            PlayersDollMessage::Respawn(
                                transform,
                                input_state,
                                velocity,
                                team,
                            ) =>
                            {
                                self.respawn(
                                    transform,
                                    input_state,
                                    velocity,
                                    team,
                                    physics_system,
                                    audio_system,
                                    engine_handle
                                );
                            }

                            PlayersDollMessage::SpawnHoleGunShotActor(
                                position,
                                radius,
                                color,
                                charging_volume_area
                            ) =>
                            {
                                self.volume_area.clear();
                                self.charging_time = 0.0;

                                if let Some(handle) = self.holegun_charge_sound.take() {
                                    audio_system.remove_sound(handle);
                                }

                                audio_system.spawn_spatial_sound(
                                    Sound::HolegunShot,
                                    0.08,
                                    1.0,
                                    false,
                                    true,
                                    fyrox_sound::source::Status::Playing,
                                    self.transform.get_position(),
                                    1.0,
                                    1.0,
                                    50.0
                                );

                                let shooted_from = self.transform.get_position() + self.transform.get_rotation() * self.weapon_shooting_point;

                                let charging_volume_area = VolumeArea::SphericalVolumeArea(
                                    SphericalVolumeArea {
                                        translation: shooted_from,
                                        radius: (charging_volume_area + 0.05) *VISUAL_FIRE_SHPERE_MULT,
                                        color: color,
                                    }
                                );
    
                                let holegun_shot = HoleGunShot::new(
                                    position,
                                    shooted_from,
                                    radius,
                                    color,
                                    charging_volume_area,
                                    VISUAL_BEAM_MULT,
                                );
    
                                let actor = ActorWrapper::HoleGunShot(holegun_shot);
    
                                engine_handle.send_command(Command {
                                    sender: 0u128,
                                    command_type: CommandType::SpawnActor(actor)
                                })
                            },

                            PlayersDollMessage::SpawHoleGunMissActor(
                                position,
                                radius,
                                color,
                                charging_volume_area
                            ) =>
                            {
                                self.volume_area.clear();
                                self.charging_time = 0.0;

                                if let Some(handle) = self.holegun_charge_sound.take() {
                                    audio_system.remove_sound(handle);
                                }

                                audio_system.spawn_spatial_sound(
                                    Sound::HolegunShot,
                                    0.08,
                                    1.0,
                                    false,
                                    true,
                                    fyrox_sound::source::Status::Playing,
                                    self.transform.get_position(),
                                    1.0,
                                    1.0,
                                    50.0
                                );

                                let shooted_from = self.transform.get_position() + self.transform.get_rotation() * self.weapon_shooting_point;

                                let charging_volume_area = VolumeArea::SphericalVolumeArea(
                                    SphericalVolumeArea {
                                        translation: shooted_from,
                                        radius: (charging_volume_area + 0.05) *VISUAL_FIRE_SHPERE_MULT,
                                        color: color,
                                    }
                                );

                                let holegun_miss = HoleGunMiss::new(
                                    position,
                                    shooted_from,
                                    radius,
                                    color,
                                    charging_volume_area,
                                    VISUAL_BEAM_MULT,
                                );

                                let actor = ActorWrapper::HoleGunMiss(holegun_miss);

                                engine_handle.send_command(Command {
                                    sender: 0u128,
                                    command_type: CommandType::SpawnActor(actor)
                                })
                            },

                            PlayersDollMessage::SpawnMachineGunShot(position, it_is_miss) => {
                                let shooted_from = self.transform.get_position() + self.transform.get_rotation() * self.weapon_shooting_point;

                                audio_system.spawn_spatial_sound(
                                    Sound::MachinegunShot,
                                    0.17,
                                    1.0,
                                    false,
                                    true,
                                    fyrox_sound::source::Status::Playing,
                                    self.transform.get_position(),
                                    1.0,
                                    1.0,
                                    50.0
                                );

                                let machinegun_shot = MachinegunShot::new(
                                    position,
                                    shooted_from,
                                    7.0,
                                    6.0,
                                    it_is_miss,
                                );

                                let actor = ActorWrapper::MachinegunShot(machinegun_shot);

                                engine_handle.send_command(Command {
                                    sender: 0u128,
                                    command_type: CommandType::SpawnActor(actor)
                                })
                            }

                            PlayersDollMessage::SetNewTeamAndPosition(
                                team,
                                is_alive,
                                transform,
                                input,
                                velocity,
                                time,
                            ) =>
                            {
                                self.is_enable = is_alive;
                                self.is_alive = is_alive;
                                self.team = team;
                                self.prev_interpolating_model_set_target_time = time;
                                self.transform = transform;
                                self.target_transform = transform;
                                self.input_state = input;
                                self.interpolating_model_target.current_velocity = velocity;
                                self.interpolating_model_target.forces.clear();
                            }
                        }
                    }

                    SpecificActorMessage::FlagMessage(message) =>
                    {
                        match message
                        {
                            FlagMessage::GiveMeTargetPosition =>
                            {
                                engine_handle.send_direct_message(
                                    from,
                                    Message {
                                        from: self.id.expect("PlayerDoll have not ActorID"),
                                        remote_sender: false,
                                        message: MessageType::SpecificActorMessage(
                                            SpecificActorMessage::FlagMessage(
                                                FlagMessage::SetTargetPosition(
                                                    self.transform.get_position() +
                                                    Vec4::new(0.0, self.radius * 2.0, 0.0, 0.0)
                                                )
                                            )
                                        )
                                    }
                                );
                            }
                            _ => {}
                        }
                    }
                    
                    SpecificActorMessage::SessionControllerMessage(message) =>
                    {
                        match message
                        {
                            SessionControllerMessage::TeamWin(team) =>
                            {
                                if self.team == team
                                {
                                    match team
                                    {
                                        Team::Red =>
                                        {
                                            effects_system.spawn_wave(
                                                engine_handle,
                                                self.transform.get_position(),
                                                vec![
                                                    0.0,
                                                    12.0,
                                                ],
                                                vec![
                                                    self.my_color,
                                                    Vec3::ZERO
                                                ],
                                                vec![
                                                    1.5,
                                                ]
                                            );
                                        }
                                        Team::Blue =>
                                        {
                                            effects_system.spawn_wave(
                                                engine_handle,
                                                self.transform.get_position(),
                                                vec![
                                                    0.0,
                                                    12.0,
                                                ],
                                                vec![
                                                    self.my_color,
                                                    Vec3::ZERO
                                                ],
                                                vec![
                                                    1.5,
                                                ]
                                            );
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }

                    SpecificActorMessage::MoveWBonusSpotMessage(_) => {}
                }

            }  
        }
    }


    fn get_mut_transform(&mut self) -> &mut Transform {
        &mut self.transform
    }


    fn get_transform(&self) -> &Transform {
        &self.transform
    }


    fn set_id(&mut self, id: ActorID) {
        self.id = Some(id);
    }


    fn get_id(&self) -> Option<ActorID> {
        self.id
    }

    fn get_physical_element(&mut self) -> Option<PhysicalElement> {
        if self.is_enable {
            Some(
                PhysicalElement {
                    id: self.get_id().expect("Actor have not ActorID"),
                    transform: &mut self.transform,
                    kinematic_collider: Some((&mut self.interpolating_model_target, Some(&mut self.target_transform))),
                    static_colliders: None,
                    dynamic_colliders: Some((&mut self.interpolating_model, self.team)),
                    static_objects: None,
                    area: None,
                }
            )
        } else {
            None
        }
    }

    fn get_visual_element(&self) -> Option<VisualElement> {
        if self.is_enable {
            Some(
                VisualElement {
                    transform: &self.transform,
                    static_objects: None,
                    coloring_areas: None,
                    volume_areas: Some(&self.volume_area),
                    waves: Some(&self.visual_wave),
                    player: Some((&self.interpolating_model[0], self.team)),
                    child_visual_elem: None,
                }
            )
        } else {
            None
        }
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
        if self.is_alive {

            self.process_player_doll_w_scanner(delta);

            if let Some(handle) = &self.holegun_charge_sound {
                audio_system.sound_set_position(
                    handle.clone(),
                    self.transform.get_position()
                );
            }
            
            if !self.volume_area.is_empty() {
    
                self.charging_time += delta * 1.6;

                let mut clear = false;

                match &mut self.volume_area[0] {
    
                    VolumeArea::SphericalVolumeArea(area) => {
                        if self.charging_time < 4.4 {
                            area.radius = self.charging_time * 0.07 * VISUAL_FIRE_SHPERE_MULT;
                            area.translation = self.transform.get_rotation() * self.weapon_shooting_point;
                        }
                        else
                        {
                            clear = true;
                        }
                    }
                    _ => {
                        panic!("charging volume area in PlayersDoll is not SphericalVolumeArea")
                    }
                }

                if clear
                {
                    self.volume_area.clear();
                }
            }

            self.extrapolate_interpolatating_model_target(delta, audio_system);

            self.interpolate_model(delta);

        } else {
            if self.need_to_die_slowly {
                self.die_slowly_timer += delta;

                if self.die_slowly_timer >= TIME_TO_DIE_SLOWLY {
                    self.is_enable = false;
                    self.need_to_die_slowly = false;
                    self.play_die_effects(engine_handle, audio_system);
                }
            }
        }

    }
}


