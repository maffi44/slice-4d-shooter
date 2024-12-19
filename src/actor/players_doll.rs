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
            SphericalVolumeArea,
            VolumeArea
        }
    },
    transform::Transform,
};

use super::{
    device::holegun::HOLE_GUN_COLOR,
    flag::FlagMessage,
    holegun_miss::HoleGunMiss,
    holegun_shot::HoleGunShot,
    machinegun_shot::MachinegunShot,
    player::{
        player_settings::PlayerSettings,
        PlayerMessage,
        PLAYER_MAX_HP,
        TIME_TO_DIE_SLOWLY
    },
    players_death_explosion::PlayersDeathExplosion,
    session_controller::SessionControllerMessage,
    shooting_impact::ShootingImpact,
    Actor,
    ActorID,
    ActorWrapper,
    CommonActorsMessages,
    Component,
    Message,
    MessageType,
    SpecificActorMessage
};

#[derive(Clone)]
pub struct PlayerDollInputState
{
    pub move_forward: bool,
    pub move_backward: bool,
    pub move_right: bool,
    pub move_left: bool,
    pub will_jump: bool,
    pub current_w_level: u32,
}


impl PlayerDollInputState {
    pub fn serialize(self) -> (bool,bool,bool,bool,bool,u32)
    {
        (
            self.move_forward,
            self.move_backward,
            self.move_right,
            self.move_left,
            self.will_jump,
            self.current_w_level
        )
    }

    pub fn deserialize(input: (bool,bool,bool,bool,bool,u32)) -> Self
    {
        PlayerDollInputState
        {
            move_forward: input.0,
            move_backward: input.1,
            move_right: input.2,
            move_left: input.3,
            will_jump: input.4,
            current_w_level: input.5,
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
    w_levels_of_map: Vec<f32>,
    radius: f32,
    my_color: Vec3,

    on_way_to_next_w_level: bool,
    current_w_level_prev_frame: u32,
}

#[derive(Clone)]
pub enum PlayersDollMessage{
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
    YouHitMe(u32),
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
        w_levels_of_map: Vec<f32>,
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
                actors_id: Some(id),
                weapon_offset,
            });
            vec
        };

        let mut interpolated_model_target = KinematicCollider::new(
            player_settings.max_speed,
            player_settings.max_accel,
            player_sphere_radius,
            player_settings.friction_on_air
        );

        interpolated_model_target.set_id(id);

        let weapon_shooting_point = weapon_offset + Vec4::NEG_Z * (player_sphere_radius * 0.49);

        let input_state = PlayerDollInputState {
            move_forward: false,
            move_backward: false,
            move_right: false,
            move_left: false,
            will_jump: false,
            current_w_level: 0u32,
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
            w_levels_of_map,
            team,
            radius: player_sphere_radius,
            my_color,
            on_way_to_next_w_level: false,
        }
    }



    fn die_immediately(&mut self, engine_handle: &mut EngineHandle, audio_system: &mut AudioSystem) {
        if self.is_alive {

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
        
        self.team = team;

        let collider_radius = self.interpolating_model[0].radius;

        let hits = physics_system.sphere_cast_on_dynamic_colliders(transform.get_position(), collider_radius);

        for hit in hits {
            engine_handle.send_direct_message(
                hit.hited_actors_id.expect("In respawn func in death on resapwn hit have not ActorID"),
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

        self.current_w_level_prev_frame = input_state.current_w_level;
        self.on_way_to_next_w_level = false;
        self.is_alive = true;
        self.is_enable = true;
        self.transform = transform.clone();
        self.target_transform = transform;
        self.input_state = input_state;
        self.interpolating_model_target.current_velocity = velocity;
    }


    fn extrapolate_interpolatating_model_target(&mut self, audio_system: &mut AudioSystem)
    {
        let mut movement_vec = Vec4::ZERO;
        
        if self.input_state.move_forward { 
            movement_vec += Vec4::NEG_Z;
        }

        if self.input_state.move_backward {
            movement_vec += Vec4::Z;
        }

        if self.input_state.move_right {
            movement_vec += Vec4::X;
        }

        if self.input_state.move_left {
            movement_vec += Vec4::NEG_X;
        }

        if self.input_state.will_jump {

            if self.interpolating_model_target.is_on_y_ground {
                self.interpolating_model_target.add_force(Vec4::Y * self.player_settings.jump_y_speed);

                self.input_state.will_jump = false;
            }
        }

        movement_vec = self.target_transform.get_rotation().inverse() * movement_vec;

        movement_vec.y = 0_f32;
        movement_vec.w = 0_f32;

        if let Some(vec) = movement_vec.try_normalize() {
            movement_vec = vec;
        }

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

        self.interpolating_model_target.add_force(Vec4::NEG_Y * self.player_settings.gravity_y_speed);

        if self.input_state.current_w_level != self.current_w_level_prev_frame
        {
            audio_system.spawn_spatial_sound(
                Sound::WShiftStart,
                1.0,
                1.0,
                false,
                true,
                Status::Playing,
                self.transform.get_position(),
                1.0,
                1.0,
                30.0,
            );

            self.on_way_to_next_w_level = true;
        }

        self.current_w_level_prev_frame = self.input_state.current_w_level;
        
        let target_w_pos = self.w_levels_of_map
            .get(self.input_state.current_w_level as usize)
            .expect("PlayerDoll's current_w_level is not exist in w_levels_of_map")
            .clone();

        let w_dif = target_w_pos - self.target_transform.get_position().w;

        self.interpolating_model_target.current_velocity.w +=
            self.player_settings.gravity_w_speed*w_dif.clamp(-1.0, 1.0);

        self.interpolating_model_target.current_velocity.w *=
            (w_dif * 5.0_f32)
            .abs()
            .clamp(0.0, 1.0);

        if self.on_way_to_next_w_level
        {
            if w_dif.abs() < self.interpolating_model_target.get_collider_radius()*0.2
            {
                self.on_way_to_next_w_level = false;
    
                audio_system.spawn_spatial_sound(
                    Sound::WShiftEnd,
                    1.0,
                    1.0,
                    false,
                    true,
                    Status::Playing,
                    self.transform.get_position(),
                    1.0,
                    1.0,
                    30.0,
                );
            }
        }
    }


    fn interpolate_model(&mut self, delta: f32)
    {
        let dist = self.target_transform.get_position() - self.transform.get_position();

        self.transform.set_position(self.transform.get_position() + dist * (13_f32*delta));
        // self.transform.set_position(self.target_transform.get_position());
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
                    CommonActorsMessages::SetTransform(transform) => {
                        self.transform = transform.clone();
                    },
                    CommonActorsMessages::Enable(switch) => {
                        self.is_enable = switch;
                    },

                    CommonActorsMessages::IncrementPosition(increment) => {
                        self.transform.increment_position(increment);
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
                match message
                {
                    SpecificActorMessage::PLayerMessage(message) => 
                    {
                        match message
                        {
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

                            PlayerMessage::DealDamageAndAddForce(damage, force, impact_pos, team) =>
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
                                                        RemoteMessage::DealDamageAndAddForce(
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
                                        from,
                                        Message {
                                            from: self.get_id().expect("Player Doll have not ActorID"),
                                            message: MessageType::SpecificActorMessage(
                                                SpecificActorMessage::PlayersDollMessage(
                                                    PlayersDollMessage::YouHitMe(damage)
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
                            PlayersDollMessage::YouHitMe(_) => {}

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

                                    let volume_area = VolumeArea::SphericalVolumeArea(
                                        SphericalVolumeArea {
                                            color: HOLE_GUN_COLOR,
                                            translation: self.transform.get_rotation().inverse() * self.weapon_shooting_point,
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

                                let shooted_from = self.transform.get_position() + self.transform.get_rotation().inverse() * self.weapon_shooting_point;

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

                                let shooted_from = self.transform.get_position() + self.transform.get_rotation().inverse() * self.weapon_shooting_point;

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
                                let shooted_from = self.transform.get_position() + self.transform.get_rotation().inverse() * self.weapon_shooting_point;

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
                                    2.0,
                                    2.0,
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
                                                self.transform.get_position(),
                                                vec![
                                                    self.radius,
                                                    self.radius * 3.0,
                                                    self.radius * 6.0,
                                                ],
                                                vec![
                                                    self.my_color,
                                                    Vec3::new(1.0, 0.0, 0.0),
                                                    Vec3::ZERO
                                                ],
                                                vec![
                                                    20.0,
                                                    20.0,
                                                ]
                                            );
                                        }
                                        Team::Blue =>
                                        {
                                            effects_system.spawn_wave(
                                                self.transform.get_position(),
                                                vec![
                                                    self.radius,
                                                    self.radius * 3.0,
                                                    self.radius * 6.0,
                                                ],
                                                vec![
                                                    self.my_color,
                                                    Vec3::new(0.0, 0.0, 1.0),
                                                    Vec3::ZERO
                                                ],
                                                vec![
                                                    20.0,
                                                    20.0,
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

        self.interpolating_model[0].set_id(id);
        self.interpolating_model_target.set_id(id);

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
                    dynamic_colliders: Some(&mut self.interpolating_model),
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
                    player: Some(&self.interpolating_model[0])
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

            if let Some(handle) = &self.holegun_charge_sound {
                audio_system.sound_set_position(
                    handle.clone(),
                    self.transform.get_position()
                );
            }
            
            if !self.volume_area.is_empty() {
    
                self.charging_time += delta * 1.6;
    
                match &mut self.volume_area[0] {
    
                    VolumeArea::SphericalVolumeArea(area) => {
                        if self.charging_time < 3.4 {
                            area.radius = self.charging_time * 0.08 * VISUAL_FIRE_SHPERE_MULT;
                        }
                        area.translation = self.transform.get_rotation().inverse() * self.weapon_shooting_point;
                    }
                    _ => {
                        panic!("charging volume area in PlayersDoll is not SphericalVolumeArea")
                    }
                }
            }

            self.extrapolate_interpolatating_model_target(audio_system);

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


