// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2026  Timofei Molokov

use crate::engine::shared_systems::audio::AudioSystem;
use crate::engine::shared_systems::resources::{ResourceRequests, Retention};
use fyrox_sound::buffer::SoundBufferResource;
use crate::engine::world::WorldContext;
use crate::engine::shared_systems::EngineSharedSystems;
use crate::actor::session_controller::send_to_session_controller;
use crate::engine::world::effects;

use client_server_protocol::{NetMessageToServer, Team};
use fyrox_sound::source::Status;
use glam::{FloatExt, Vec3, Vec4};
use rand::Rng;

use crate::{
    engine::{
        world::messenger::Messenger, world::physics::{area::{Area, AreaMessage}, colliders_container::PhysicalElement, physics_system_data::ShapeType}, shared_systems::render::VisualElement, shared_systems::ui::UIElementType, world::static_object::{SphericalVolumeArea, VisualWave, VolumeArea}
    },
    engine::transform::Transform
};

use super::{
    main_player::{BLUE_TEAM_COLOR, RED_TEAM_COLOR}, session_controller::SessionControllerMessage, Actor, ActorID, CommonActorsMessage, Message, MessageType, PhysicsMessages, SpecificActorMessage
};

#[derive(Clone)]
pub enum FlagMessage
{
    SetFlagStatus(Team, FlagStatus),
    YouInteractingWithFlag(Team, FlagStatus),
    GiveMeTargetPosition,
    SetTargetPosition(Vec4),
    PlayerDied(
        // true if dead outside of map (in space)
        bool
    ),
}

#[derive(Clone, Copy)]
pub enum FlagStatus
{
    Captured(ActorID),
    Missed(Vec4),
    OnTheBase,
}

impl From<client_server_protocol::FlagStatus> for FlagStatus
{
    fn from(value: client_server_protocol::FlagStatus) -> Self {
        match value
        {
            client_server_protocol::FlagStatus::OnTheBase =>
            {
                FlagStatus::OnTheBase
            }
            client_server_protocol::FlagStatus::Droped(arr) =>
            {
                FlagStatus::Missed(Vec4::from_array(arr))
            }
            client_server_protocol::FlagStatus::Captured(id) =>
            {
                FlagStatus::Captured(id)
            }
        }
    }
}

const TIME_TO_CHANGE_NEXT_TARGET_SWING_POSITION: f32 = 0.8;
const FLAG_SWING_RANGE: f32 = 0.07;
const FLAG_UI_TICK_TIME: f32 = 0.5;

const WAVE_TICK_TIME: f32 = 1.5;
const WAVE_TARGET_RADIUS: f32 = 2.0;

fn get_random_vec4(range_min: f32, range_max: f32) -> Vec4
{
    assert!(range_min < range_max);

    let mut rng = rand::rng();
    let x = rng.random_range(range_min..=range_max);
    let y = rng.random_range(range_min..=range_max);
    let z = rng.random_range(range_min..=range_max);
    let w = rng.random_range(range_min..=range_max);

    return Vec4::new(x, y, z, w);
}

pub const FLAG_AREA_VISUAL_RADIUS: f32 = 1.0;
pub const FLAG_AREA_PHYSICAL_RADIUS: f32 = 1.5;

pub struct Flag
{
    flag_captured_sound: Option<SoundBufferResource>,
    flag_returned_sound: Option<SoundBufferResource>,


    transform: Transform,
    next_target_swing_position_in_secs: f32,
    target_flag_swing_position: Vec4,
    current_flag_swing_position: Vec4,
    target_position: Vec4,
    transfrom_of_the_base: Transform,
    id: Option<ActorID>,
    status: FlagStatus,
    owned_by_team: Team,
    area: Area,
    visual_areas: Vec<VolumeArea>,
    my_color: Vec3,
    opponent_color: Vec3,
    flag_ui_tick_switch: bool,
    flag_ui_tick_timer: f32,

    radius_mult: f32,

    wave_tick_timer: f32,
    waves: Vec<VisualWave>,
    wave_1_target_rad: f32,
    wave_2_target_rad: f32,
}

impl Flag
{
    pub fn new(
        resources: &mut ResourceRequests,
        flag_owned_by_team: Team, transform_of_the_base: Transform) -> Self
    {
        let target_flag_swing_position = get_random_vec4(
            -FLAG_SWING_RANGE,
            FLAG_SWING_RANGE
        );

        let area: Area = Area::new(
            Vec4::ZERO,
            ShapeType::Sphere,
            Vec4::new(
                FLAG_AREA_PHYSICAL_RADIUS,
                0.0, 0.0, 0.0
            )
        );

        let mut visual_areas = Vec::with_capacity(1);

        let my_color = match flag_owned_by_team
        {
            Team::Red =>
            {
                RED_TEAM_COLOR
            }
            
            Team::Blue =>
            {
                BLUE_TEAM_COLOR
            }
        };

        let opponent_color = match flag_owned_by_team
        {
            Team::Red =>
            {
                BLUE_TEAM_COLOR
            }
            
            Team::Blue =>
            {
                RED_TEAM_COLOR
            }
        };

        let visual_area =  VolumeArea::SphericalVolumeArea(
            SphericalVolumeArea {
                radius: FLAG_AREA_VISUAL_RADIUS,
                translation: Vec4::ZERO,
                color: my_color,
            }
        );

        visual_areas.push(visual_area);

        let waves = vec![
            VisualWave {
                translation: Vec4::ZERO,
                radius: 0.0,
                color: my_color,
            },
            VisualWave {
                translation: Vec4::ZERO,
                radius: 1.6,
                color: my_color,
            }

        ];

        Flag {
            flag_captured_sound: resources.sound(
                crate::embedded_asset!("sounds/flag_captured.wav"), Retention::Session,
            ),
            flag_returned_sound: resources.sound(
                crate::embedded_asset!("sounds/flag_returned.wav"), Retention::Session,
            ),

            transform: transform_of_the_base,
            target_flag_swing_position,
            current_flag_swing_position: Vec4::ZERO,
            target_position: transform_of_the_base.get_position(),
            transfrom_of_the_base: transform_of_the_base,
            id: None,
            status:FlagStatus::OnTheBase,
            owned_by_team: flag_owned_by_team,
            next_target_swing_position_in_secs: TIME_TO_CHANGE_NEXT_TARGET_SWING_POSITION,
            area,
            visual_areas,
            my_color,
            opponent_color,
            flag_ui_tick_switch: true,
            flag_ui_tick_timer: 0.0,
            waves,
            radius_mult: 1.0,
            wave_tick_timer: 0.0,
            wave_1_target_rad: WAVE_TARGET_RADIUS,
            wave_2_target_rad: 0.0,
        }
    }

    pub fn set_flag_on_base_status(
        &mut self,
        audio_system: &mut AudioSystem,
        messenger: &mut Messenger,
    )
    {

        self.transform = self.transfrom_of_the_base;
        self.target_position = self.transfrom_of_the_base.get_position();

        match self.status
        {
            FlagStatus::Missed(_) =>
            {
                effects::spawn_wave(
                    messenger,
                    self.transform.get_position(),
                    vec![
                        0.0,
                        24.0,
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
            _ => {}
        }

        audio_system.spawn_non_spatial_sound(
            &self.flag_returned_sound,
            1.0,
            1.0,
            false,
            true,
            Status::Playing,
        );
        
        self.status = FlagStatus::OnTheBase;
    }

    pub fn set_flag_missed_status(
        &mut self,
        pos: Vec4,
        messenger: &mut Messenger,
    )
    {
        self.target_position = pos;
        effects::spawn_wave(
            messenger,
            self.transform.get_position(),
            vec![
                0.0,
                24.0,
            ],
            vec![
                self.my_color,
                Vec3::ZERO
            ],
            vec![
                1.5,
            ]
        );
        self.status = FlagStatus::Missed(pos);
    }

    pub fn set_flag_captured_status(
        &mut self,
        captured_by: ActorID,
        messenger: &mut Messenger,
        audio_system: &mut AudioSystem,
    )
    {
        self.area.clear_containing_colliders_list();
        
        messenger.send_direct_message(
            captured_by,
            Message {
                from: self.id.expect("Flag has no ActorID"),
                remote_sender: false,
                message: MessageType::SpecificActorMessage(
                    SpecificActorMessage::FlagMessage(
                        FlagMessage::GiveMeTargetPosition
                    )
                )
            }
        );

        effects::spawn_wave(
            messenger,
            self.transform.get_position(),
            vec![
                0.0,
                2.0,
                24.0,
            ],
            vec![
                self.my_color,
                self.opponent_color,
                Vec3::ZERO
            ],
            vec![
                0.3,
                1.2,
            ]
        );
        
        audio_system.spawn_spatial_sound(
            &self.flag_captured_sound,
            1.0,
            1.0,
            false,
            true,
            Status::Playing,
            self.transform.get_position(),
            1.0,
            1.0,
            15.0
        );

        self.status = FlagStatus::Captured(captured_by);
    }
}

impl Actor for Flag
{
    fn tick(
        &mut self,
        world: &mut WorldContext,
        shared: &mut EngineSharedSystems,
    )
        {
        let delta = world.time.delta();
        let messenger = &mut *world.messenger;
        let ui_system = &mut shared.ui;


        match self.status
        {
            FlagStatus::OnTheBase =>
            {
                self.flag_ui_tick_switch = true;
            }

            _ =>
            {
                if self.flag_ui_tick_timer >= FLAG_UI_TICK_TIME
                {
                    self.flag_ui_tick_timer = 0.0;
                    self.flag_ui_tick_switch = !self.flag_ui_tick_switch;
                }

                self.flag_ui_tick_timer += delta;
            }
        }

        match self.status
        {
            FlagStatus::Captured(_) =>
            {
                self.radius_mult = self.radius_mult.lerp(0.25, 1.0 - delta*3.0);
            }

            _ =>
            {
                self.radius_mult = self.radius_mult.lerp(1.0, 1.0 - delta*3.0);
            }
        }

        match self.owned_by_team
        {
            Team::Red =>
            {
                let ui_flag = ui_system.get_mut_ui_element(&UIElementType::RedFlagMark);

                *ui_flag.get_ui_data_mut().get_is_visible_mut() = self.flag_ui_tick_switch;
            }

            Team::Blue =>
            {
                let ui_flag = ui_system.get_mut_ui_element(&UIElementType::BlueFlagMark);

                *ui_flag.get_ui_data_mut().get_is_visible_mut() = self.flag_ui_tick_switch;
            }
        }

        self.next_target_swing_position_in_secs -= delta;

        if self.next_target_swing_position_in_secs <= 0.0
        {
            self.target_flag_swing_position = get_random_vec4(
                -FLAG_SWING_RANGE,
                FLAG_SWING_RANGE
            );

            self.next_target_swing_position_in_secs = TIME_TO_CHANGE_NEXT_TARGET_SWING_POSITION;
        }

        self.current_flag_swing_position = self.current_flag_swing_position.lerp(
            self.target_flag_swing_position,
            delta * 0.3
        );

        let mut current_flag_position = self.transform.get_position();

        current_flag_position = current_flag_position.lerp(
            self.target_position,
            delta * 9.0
        );

        current_flag_position += self.current_flag_swing_position;

        self.transform.set_position(current_flag_position);

        match self.status
        {
            FlagStatus::Captured(id) =>
            {
                messenger.send_direct_message(
                    id,
                    Message {
                        from: self.get_id().expect("Flag have not ActorID"),
                        remote_sender: false,
                        message: MessageType::SpecificActorMessage(
                            SpecificActorMessage::FlagMessage(
                                FlagMessage::GiveMeTargetPosition
                            )
                        )
                    }
                );
            }
            _ => {}
        }

        self.wave_tick_timer += delta;

        if self.wave_tick_timer >= WAVE_TICK_TIME
        {
            self.wave_tick_timer = 0.0;

            if self.wave_1_target_rad == 0.0
            {
                self.wave_1_target_rad = WAVE_TARGET_RADIUS;
                self.wave_2_target_rad = 0.0;
            }
            else
            {
                self.wave_1_target_rad = 0.0;
                self.wave_2_target_rad = WAVE_TARGET_RADIUS;
            }
        }
        
        self.waves[0].radius = self.waves[0].radius.lerp(
            self.wave_1_target_rad * self.radius_mult,
            delta*WAVE_TICK_TIME*3.0
        );
        self.waves[0].color = self.waves[0].color.lerp(
            self.wave_2_target_rad * self.my_color,
            delta*WAVE_TICK_TIME*3.0
        );

        self.waves[1].radius = self.waves[1].radius.lerp(
            self.wave_2_target_rad  * self.radius_mult,
            delta*WAVE_TICK_TIME*0.9
        );

        self.waves[1].color = self.waves[1].color.lerp(
            self.wave_1_target_rad * self.my_color,
            delta*WAVE_TICK_TIME*0.9
        );

        match &mut self.visual_areas[0]
        {
            VolumeArea::SphericalVolumeArea(area) =>
            {
                area.radius = FLAG_AREA_VISUAL_RADIUS * self.radius_mult;
            }
            _ => {}
        }
    }

    fn get_mut_transform(&mut self) -> &mut Transform {
        &mut self.transform
    }

    fn get_physical_element(&mut self) -> Option<PhysicalElement<'_>>
    {
        match self.status
        {
            FlagStatus::Captured(_) =>
            {
                None
            }

            _ =>
            {
                Some(
                    PhysicalElement
                    {
                        id: self.get_id().expect("Actor have not ActorID"),
                        transform: &mut self.transform,
                        kinematic_collider: None,
                        dynamic_colliders: None,
                        static_colliders: None,
                        static_objects: None,
                        area: Some(&mut self.area)
                    }
                )
            }
        }

    }


    fn get_visual_element(&self) -> Option<VisualElement<'_>>
    {
        Some(
            VisualElement
            {
                transform: &self.transform,
                static_objects: None,
                coloring_areas: None,
                volume_areas: Some(&self.visual_areas),
                waves: Some(&self.waves),
                player: None,
                child_visual_elem: None,
            }
        )
    }

    fn get_transform(&self) -> &Transform {
        &self.transform
    }

    fn get_id(&self) -> Option<ActorID> {
        self.id
    }

    fn set_id(&mut self, id: ActorID) {
        self.id = Some(id);
    }

    fn recieve_message(
        &mut self,
        message: &Message,
        world: &mut WorldContext,
        shared: &mut EngineSharedSystems,
    ) {
        let messenger = &mut *world.messenger;
        let audio_system = &mut shared.audio;

        
        let from = message.from;

        match &message.message
        {
            MessageType::SpecificActorMessage(message) =>
            {
                match message {
                    SpecificActorMessage::FlagMessage(message) =>
                    {
                        match message
                        {
                            FlagMessage::PlayerDied(in_space) =>
                            {
                                match self.status
                                {
                                    FlagStatus::Captured(id) =>
                                    {
                                        if from == id
                                        {
                                            send_to_session_controller(
                                                messenger,
                                                self.get_id().expect("Player have not ActorID"),
                                                SessionControllerMessage::SendToServer(
                                                    NetMessageToServer::DropedFlag(
                                                        self.owned_by_team,
                                                        self.get_transform().get_position().to_array(),
                                                        *in_space,
                                                    ),
                                                ),
                                            );
                                        }
                                    }
                                    _ => {}
                                }
                            }

                            FlagMessage::SetFlagStatus(team, status) =>
                            {
                                if self.owned_by_team == *team
                                {
                                    self.area.clear_containing_colliders_list();
                                    
                                    match *status {
                                        FlagStatus::OnTheBase =>
                                        {
                                            self.set_flag_on_base_status(audio_system, messenger);
                                        }
                                        FlagStatus::Missed(pos) =>
                                        {
                                            self.set_flag_missed_status(pos, messenger);
                                        }
                                        FlagStatus::Captured(captured_by) =>
                                        {
                                            self.set_flag_captured_status(captured_by, messenger, audio_system);
                                        }
                                    }
                                }
                            }
                            FlagMessage::SetTargetPosition(position) =>
                            {
                                match self.status
                                {
                                    FlagStatus::Captured(id) =>
                                    {
                                        if id == from
                                        {
                                            self.target_position = *position;
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            
                            _ => {}
                        }
                    }
                    SpecificActorMessage::SessionControllerMessage(message) =>
                    {
                        match message {
                            SessionControllerMessage::NewSessionStarted(_) =>
                            {
                                self.set_flag_on_base_status(audio_system, messenger);
                            }
                            SessionControllerMessage::TeamWin(team) =>
                            {
                                match team {
                                    Team::Red =>
                                    {
                                        effects::spawn_wave(
                                            messenger,
                                            self.transform.get_position(),
                                            vec![
                                                0.0,
                                                2.0,
                                                24.0,
                                                0.0,
                                                2.0,
                                                24.0,
                                                0.0,
                                                2.0,
                                                24.0,
                                            ],
                                            vec![
                                                self.my_color,
                                                RED_TEAM_COLOR,
                                                Vec3::ZERO,
                                                Vec3::ZERO,
                                                RED_TEAM_COLOR,
                                                Vec3::ZERO,
                                                Vec3::ZERO,
                                                RED_TEAM_COLOR,
                                                Vec3::ZERO
                                            ],
                                            vec![
                                                0.3,
                                                1.2,
                                                0.1,
                                                0.3,
                                                1.2,
                                                0.1,
                                                0.3,
                                                1.2,
                                            ]
                                        );
                                    }
                                    Team::Blue =>
                                    {
                                        effects::spawn_wave(
                                            messenger,
                                            self.transform.get_position(),
                                            vec![
                                                0.0,
                                                2.0,
                                                24.0,
                                                0.0,
                                                2.0,
                                                24.0,
                                                0.0,
                                                2.0,
                                                24.0,
                                            ],
                                            vec![
                                                self.my_color,
                                                BLUE_TEAM_COLOR,
                                                Vec3::ZERO,
                                                Vec3::ZERO,
                                                BLUE_TEAM_COLOR,
                                                Vec3::ZERO,
                                                Vec3::ZERO,
                                                BLUE_TEAM_COLOR,
                                                Vec3::ZERO
                                            ],
                                            vec![
                                                0.3,
                                                1.2,
                                                0.1,
                                                0.3,
                                                1.2,
                                                0.1,
                                                0.3,
                                                1.2,
                                            ]
                                        );
                                    }
                                }
                            }
                            SessionControllerMessage::JoinedToSession(
                                _,
                                red_flag_status,
                                blue_flag_status,
                                _,
                                _,
                                _
                            ) =>
                            {
                                match self.owned_by_team
                                {
                                    Team::Red =>
                                    {
                                        match *red_flag_status
                                        {
                                            FlagStatus::OnTheBase =>
                                            {
                                                self.set_flag_on_base_status(audio_system, messenger);
                                            }

                                            FlagStatus::Missed(pos) =>
                                            {
                                                self.set_flag_missed_status(pos, messenger);
                                            }

                                            FlagStatus::Captured(id) =>
                                            {
                                                self.set_flag_captured_status(id, messenger, audio_system);
                                            }
                                        }
                                    }
                                    Team::Blue =>
                                    {
                                        match *blue_flag_status
                                        {
                                            FlagStatus::OnTheBase =>
                                            {
                                                self.set_flag_on_base_status(audio_system, messenger);
                                            }

                                            FlagStatus::Missed(pos) =>
                                            {
                                                self.set_flag_missed_status(pos, messenger);
                                            }

                                            FlagStatus::Captured(id) =>
                                            {
                                                self.set_flag_captured_status(id, messenger, audio_system);
                                            }
                                        }
                                    }
                                }

                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            MessageType::CommonActorsMessages(message) =>
            {
                match message {
                    CommonActorsMessage::SetTransform(tr) =>
                    {
                        self.transform = *tr;
                    }
                    CommonActorsMessage::ClientDisconnectedFromGameServer =>
                    {
                        self.set_flag_on_base_status(audio_system, messenger);
                    }
                    _ => {}
                }
            }
            MessageType::PhysicsMessages(message) =>
            {
                match message {
                    PhysicsMessages::AreaMessage(message) =>
                    {
                        match message
                        {
                            AreaMessage::ActorEnterArea(id) =>
                            {
                                messenger.send_direct_message(
                                    *id,
                                    Message {
                                        from: self.id.expect("Flag have not ActorID"),
                                        remote_sender: false,
                                        message: MessageType::SpecificActorMessage(
                                            SpecificActorMessage::FlagMessage(
                                                FlagMessage::YouInteractingWithFlag(
                                                    self.owned_by_team,
                                                    self.status
                                                )
                                            )
                                        )
                                    }
                                );
                            }
                            AreaMessage::ActorIsContainedInsideArea(_id) =>
                            {

                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        }
        
    }
}

