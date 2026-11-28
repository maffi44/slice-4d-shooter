// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2026  Timofei Molokov

use crate::engine::shared_systems::audio::AudioSystem;
use crate::engine::shared_systems::resources::{ResourceLoader, Retention};
use fyrox_sound::buffer::SoundBufferResource;
use crate::engine::world::WorldContext;
use crate::engine::shared_systems::EngineSharedSystems;
use crate::actor::session_controller::send_to_session_controller;
use crate::engine::world::effects;

use net_protocol::gameplay::{NetMessageToServer, Team};
use fyrox_sound::source::Status;
use glam::{FloatExt, Vec3, Vec4};
use rand::RngExt;

use crate::{
    engine::{
        world::messenger::Messenger, world::physics::{area::{Area, AreaMessage}, colliders_container::PhysicalElement, physics_system_data::ShapeType}, shared_systems::render::VisualElement, world::ui::UIElementType, world::static_object::{SphericalVolumeArea, VisualWave, VolumeArea}
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
    /// A flag asks the player with this session peer ID to identify its local actor.
    FindCarrier(ActorID),
    /// Sent by a player from its local actor ID, on spawn or in reply to FindCarrier.
    CarrierAvailable(ActorID),
    GiveMeTargetPosition,
    SetTargetPosition(Vec4),
    PlayerDied { peer_id: Option<ActorID>, in_space: bool },
}

impl FlagMessage
{
    pub(super) fn from_actor(self, from: ActorID) -> Message
    {
        Message {
            from,
            remote_sender: false,
            message: MessageType::SpecificActorMessage(SpecificActorMessage::FlagMessage(self)),
        }
    }
}

#[derive(Clone, Copy)]
pub enum FlagStatus
{
    Captured(ActorID),
    Missed(Vec4),
    OnTheBase,
}

impl From<net_protocol::gameplay::FlagStatus> for FlagStatus
{
    fn from(value: net_protocol::gameplay::FlagStatus) -> Self {
        match value
        {
            net_protocol::gameplay::FlagStatus::OnTheBase =>
            {
                FlagStatus::OnTheBase
            }
            net_protocol::gameplay::FlagStatus::Droped(arr) =>
            {
                FlagStatus::Missed(Vec4::from_array(arr))
            }
            net_protocol::gameplay::FlagStatus::Captured(id) =>
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
    // Captured stores the server peer ID. Only a matching player can supply the local ID.
    carrier_actor_id: Option<ActorID>,
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
        resources: &mut ResourceLoader,
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
            flag_captured_sound: resources.load_media(
                crate::embedded_asset!("sounds/flag_captured.wav"), Retention::Session,
            ),
            flag_returned_sound: resources.load_media(
                crate::embedded_asset!("sounds/flag_returned.wav"), Retention::Session,
            ),

            transform: transform_of_the_base,
            target_flag_swing_position,
            current_flag_swing_position: Vec4::ZERO,
            target_position: transform_of_the_base.get_position(),
            transfrom_of_the_base: transform_of_the_base,
            id: None,
            status: FlagStatus::OnTheBase,
            carrier_actor_id: None,
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
        
        self.carrier_actor_id = None;
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
        self.carrier_actor_id = None;
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
        
        if !matches!(self.status, FlagStatus::Captured(peer) if peer == captured_by)
        {
            self.carrier_actor_id = None;
        }
        self.status = FlagStatus::Captured(captured_by);
        if self.carrier_actor_id.is_some()
        {
            self.request_target_position_from_carrier(messenger);
        }
        else
        {
            let query = FlagMessage::FindCarrier(captured_by).from_actor(self.id.expect("Flag has no ActorID"));
            messenger.send_broadcast_message(query);
        }

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
    }

    /// Follows the carrier when the world renumbers it, without replaying capture effects or
    /// notifying the server. On an id collision the displaced actor's notification is delivered
    /// first, so each notification is matched against the id the carrier had at that point.
    fn on_actor_changed_id(&mut self, old_id: ActorID, new_id: ActorID)
    {
        if let Some(carrier) = &mut self.carrier_actor_id
        {
            if *carrier == old_id
            {
                *carrier = new_id;
            }
        }
    }

    fn request_target_position_from_carrier(&self, messenger: &mut Messenger)
    {
        if let Some(carrier) = self.carrier_actor_id
        {
            messenger.send_direct_message(
                carrier,
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
    }

    /// The carrier's death drops the flag (reported to the server through the session controller).
    fn on_player_died(&self, from: ActorID, peer_id: Option<ActorID>, in_space: bool, messenger: &mut Messenger)
    {
        let is_carrier = match self.carrier_actor_id
        {
            Some(id) => id == from,
            // A death can already be queued when the flag sends its first lookup.
            None => matches!(self.status, FlagStatus::Captured(peer) if Some(peer) == peer_id),
        };
        if is_carrier
        {
            send_to_session_controller(
                messenger,
                self.get_id().expect("Flag have not ActorID"),
                SessionControllerMessage::SendToServer(
                    NetMessageToServer::DropedFlag(
                        self.owned_by_team,
                        self.get_transform().get_position().to_array(),
                        in_space,
                    ),
                ),
            );
        }
    }
}

impl Actor for Flag
{
    fn tick(
        &mut self,
        world: &mut WorldContext,
        _shared: &mut EngineSharedSystems,
    )
        {
        let delta = world.time.delta();
        let messenger = &mut *world.messenger;
        let ui_system = &mut *world.ui;


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

        self.request_target_position_from_carrier(messenger);

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
                            FlagMessage::PlayerDied { peer_id, in_space } =>
                            {
                                self.on_player_died(from, *peer_id, *in_space, messenger);
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
                            FlagMessage::CarrierAvailable(peer) =>
                            {
                                if self.carrier_actor_id != Some(from)
                                    && matches!(self.status, FlagStatus::Captured(expected) if expected == *peer)
                                {
                                    self.carrier_actor_id = Some(from);
                                    self.request_target_position_from_carrier(messenger);
                                }
                            }
                            FlagMessage::SetTargetPosition(position) =>
                            {
                                if self.carrier_actor_id == Some(from)
                                {
                                    self.target_position = *position;
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
                    CommonActorsMessage::IWasChangedMyId(new_id) =>
                    {
                        self.on_actor_changed_id(from, *new_id);
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



#[cfg(test)]
mod tests
{
    use super::*;
    use crate::{
        actor::{move_w_bonus::MoveWBonusSpot, ActorWrapper},
        engine::{tests::headless_loader, world::actors::Actors},
    };

    const FLAG_ID: ActorID = 5;
    const CARRIER: ActorID = 10;
    const OTHER: ActorID = 20;

    fn flag_with_status(status: FlagStatus) -> Flag
    {
        let runtime = tokio::runtime::Builder::new_current_thread().build().unwrap();
        let mut resources = headless_loader(&runtime);
        let mut flag = Flag::new(&mut resources, Team::Red, Transform::new());
        flag.set_id(FLAG_ID);
        flag.status = status;
        if let FlagStatus::Captured(id) = status { flag.carrier_actor_id = Some(id); }
        flag
    }

    /// A world holding the carrier and another actor.
    fn world() -> (Actors, Messenger)
    {
        let mut actors = Actors::new();
        let mut messenger = Messenger::new();

        for id in [CARRIER, OTHER]
        {
            let mut actor = ActorWrapper::MoveWBonusSpot(MoveWBonusSpot::new(Transform::new(), 0));
            actor.set_id(id);
            actors.insert(actor, &mut messenger);
        }

        (actors, messenger)
    }

    /// Delivers the world's queued id-change notifications to the flag in sending order.
    fn deliver_id_changes(flag: &mut Flag, messenger: &Messenger)
    {
        for message in messenger.queued_broadcast_messages()
        {
            if let MessageType::CommonActorsMessages(CommonActorsMessage::IWasChangedMyId(new_id)) = message.message
            {
                flag.on_actor_changed_id(message.from, new_id);
            }
        }
    }

    fn position_request_recipients(messenger: &Messenger) -> Vec<ActorID>
    {
        messenger.queued_direct_messages()
            .filter(|(_, message)| matches!(
                message.message,
                MessageType::SpecificActorMessage(SpecificActorMessage::FlagMessage(FlagMessage::GiveMeTargetPosition))
            ))
            .map(|(to, _)| to)
            .collect()
    }

    fn flag_drop_reports(messenger: &Messenger) -> usize
    {
        messenger.queued_broadcast_messages()
            .filter(|message| matches!(
                message.message,
                MessageType::SpecificActorMessage(SpecificActorMessage::SessionControllerMessage(
                    SessionControllerMessage::SendToServer(NetMessageToServer::DropedFlag(..))
                ))
            ))
            .count()
    }

    /// The next position request and the death check refer to `expected` only.
    fn assert_follows(flag: &Flag, expected: ActorID, stale: ActorID)
    {
        assert_eq!(flag.carrier_actor_id, Some(expected));

        let mut messenger = Messenger::new();
        flag.request_target_position_from_carrier(&mut messenger);
        assert_eq!(position_request_recipients(&messenger), vec![expected]);

        flag.on_player_died(stale, None, false, &mut messenger);
        assert_eq!(flag_drop_reports(&messenger), 0);
        flag.on_player_died(expected, None, false, &mut messenger);
        assert_eq!(flag_drop_reports(&messenger), 1);
    }

    #[test]
    fn join_snapshot_waits_for_the_peer_and_finds_it_again_after_a_local_rename()
    {
        use crate::{
            actor::{move_w_bonus::BonusSpotStatus, players_doll::PlayerDoll},
            engine::{tests::headless_map, world::{
                messenger::{Command, CommandType}, physics::PhysicsSystem, time::WorldTimeSystem,
            }},
        };

        let (mut shared, map) = headless_map();
        let mut ui = Default::default();
        let physics = PhysicsSystem::new(&map);
        let time = WorldTimeSystem::new();
        let (mut actors, mut messenger) = world(); // A map actor already occupies OTHER.
        actors.insert(ActorWrapper::Flag(flag_with_status(FlagStatus::OnTheBase)), &mut messenger);

        let joined = Message {
            from: 0,
            remote_sender: true,
            message: MessageType::SpecificActorMessage(SpecificActorMessage::SessionControllerMessage(
                SessionControllerMessage::JoinedToSession(Team::Blue, FlagStatus::Captured(OTHER),
                    FlagStatus::OnTheBase, BonusSpotStatus::BonusOnTheSpot, 0, 0),
            )),
        };
        messenger.send_broadcast_message(joined);
        messenger.process_messages(&mut actors, &physics, &time, &mut None, &mut ui, &mut shared);
        assert_eq!(stored_flag(&actors).carrier_actor_id, None);
        stored_flag(&actors).request_target_position_from_carrier(&mut messenger);
        assert!(position_request_recipients(&messenger).is_empty());

        // A queued death before the first binding reply must identify the server peer.
        let mut deaths = Messenger::new();
        stored_flag(&actors).on_player_died(OTHER, Some(CARRIER), false, &mut deaths);
        assert_eq!(flag_drop_reports(&deaths), 0);
        stored_flag(&actors).on_player_died(CARRIER, Some(OTHER), false, &mut deaths);
        assert_eq!(flag_drop_reports(&deaths), 1);

        let doll = PlayerDoll::new(OTHER, 1.0, Transform::new(), true, &mut shared.resources, Team::Blue, false);
        messenger.send_command(Command {
            sender: 0,
            command_type: CommandType::SpawnActor(Box::new(ActorWrapper::PlayersDoll(doll))),
        });
        messenger.process_messages(&mut actors, &physics, &time, &mut None, &mut ui, &mut shared);
        assert_eq!(stored_flag(&actors).carrier_actor_id, Some(OTHER));
        assert_eq!(stored_flag(&actors).target_position, Vec4::Y * 2.0);

        // The doll takes CARRIER from the other map actor. Its server peer ID is still OTHER.
        actors.change_id(OTHER, CARRIER, &mut messenger);
        messenger.process_messages(&mut actors, &physics, &time, &mut None, &mut ui, &mut shared);
        assert_follows(stored_flag(&actors), CARRIER, OTHER);

        // A fresh server capture after the doll was displaced must resolve the same peer.
        for status in [FlagStatus::OnTheBase, FlagStatus::Captured(OTHER)]
        {
            let update = FlagMessage::SetFlagStatus(Team::Red, status).from_actor(0);
            messenger.send_direct_message(FLAG_ID, update);
        }
        messenger.process_messages(&mut actors, &physics, &time, &mut None, &mut ui, &mut shared);
        assert_follows(stored_flag(&actors), CARRIER, OTHER);
    }

    fn stored_flag(actors: &Actors) -> &Flag
    {
        match actors.get(&FLAG_ID).unwrap()
        {
            ActorWrapper::Flag(flag) => flag,
            _ => panic!("expected the flag"),
        }
    }

    #[test]
    fn carrier_renamed_to_a_free_id_keeps_the_flag()
    {
        let (mut actors, _) = world();
        let mut messenger = Messenger::new();
        let mut flag = flag_with_status(FlagStatus::Captured(CARRIER));

        actors.change_id(CARRIER, 30, &mut messenger);
        deliver_id_changes(&mut flag, &messenger);

        assert_follows(&flag, 30, CARRIER);
    }

    #[test]
    fn carrier_renamed_onto_another_actor_keeps_the_flag()
    {
        let (mut actors, _) = world();
        let mut messenger = Messenger::new();
        let mut flag = flag_with_status(FlagStatus::Captured(CARRIER));

        // OTHER is displaced first (OTHER -> random), then the carrier takes OTHER's id.
        actors.change_id(CARRIER, OTHER, &mut messenger);
        deliver_id_changes(&mut flag, &messenger);

        assert_follows(&flag, OTHER, CARRIER);
    }

    #[test]
    fn displaced_carrier_keeps_the_flag()
    {
        let (mut actors, _) = world();
        let mut messenger = Messenger::new();
        let mut flag = flag_with_status(FlagStatus::Captured(OTHER));

        // The carrier (OTHER) is displaced to a random id when CARRIER takes its id.
        actors.change_id(CARRIER, OTHER, &mut messenger);
        let displaced_to = actors.iter()
            .map(|(id, _)| *id)
            .find(|id| *id != OTHER)
            .expect("the displaced carrier");
        deliver_id_changes(&mut flag, &messenger);

        assert_follows(&flag, displaced_to, OTHER);
    }

    #[test]
    fn unrelated_renames_and_uncaptured_flags_keep_their_state()
    {
        let mut flag = flag_with_status(FlagStatus::Captured(CARRIER));
        flag.on_actor_changed_id(OTHER, 40);
        assert_eq!(flag.carrier_actor_id, Some(CARRIER));

        for status in [FlagStatus::OnTheBase, FlagStatus::Missed(Vec4::ONE)]
        {
            let mut flag = flag_with_status(status);
            flag.on_actor_changed_id(CARRIER, 40);
            assert!(flag.carrier_actor_id.is_none());

            let mut messenger = Messenger::new();
            flag.request_target_position_from_carrier(&mut messenger);
            assert!(position_request_recipients(&messenger).is_empty());
        }
    }
}
