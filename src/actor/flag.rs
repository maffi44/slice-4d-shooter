use client_server_protocol::{NetCommand, NetMessageToServer, Team};
use fyrox_sound::source::Status;
use glam::{Vec3, Vec4};
use rand::Rng;

use crate::{
    engine::{
        audio::{AudioSystem, Sound}, effects::EffectsSystem, engine_handle::{Command, CommandType, EngineHandle}, physics::{area::{Area, AreaMessage}, colliders_container::PhysicalElement, physics_system_data::ShapeType, PhysicsSystem}, render::VisualElement, time::TimeSystem, ui::{UIElementType, UISystem}, world::static_object::{SphericalVolumeArea, VolumeArea}
    },
    transform::Transform
};

use super::{
    player::{BLUE_TEAM_COLOR, RED_TEAM_COLOR}, session_controller::SessionControllerMessage, Actor, ActorID, CommonActorsMessage, Message, MessageType, PhysicsMessages, SpecificActorMessage
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

fn get_random_vec4(range_min: f32, range_max: f32) -> Vec4
{
    assert!(range_min < range_max);

    let mut rng = rand::thread_rng();
    let x = rng.gen_range(range_min..=range_max);
    let y = rng.gen_range(range_min..=range_max);
    let z = rng.gen_range(range_min..=range_max);
    let w = rng.gen_range(range_min..=range_max);

    return Vec4::new(x, y, z, w);
}

pub const FLAG_AREA_RADIUS: f32 = 1.0;

pub struct Flag
{
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
}

impl Flag
{
    pub fn new(flag_owned_by_team: Team, transfrom_of_the_base: Transform) -> Self
    {
        let target_flag_swing_position = get_random_vec4(
            -FLAG_SWING_RANGE,
            FLAG_SWING_RANGE
        );

        let area: Area = Area::new(
            Vec4::ZERO,
            ShapeType::Sphere,
            Vec4::new(
                FLAG_AREA_RADIUS,
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

        let test_visual_area =  VolumeArea::SphericalVolumeArea(
            SphericalVolumeArea {
                radius: FLAG_AREA_RADIUS,
                translation: Vec4::ZERO,
                color: my_color,
            }
        );

        visual_areas.push(test_visual_area);

        Flag {
            transform: transfrom_of_the_base,
            target_flag_swing_position,
            current_flag_swing_position: Vec4::ZERO,
            target_position: transfrom_of_the_base.get_position(),
            transfrom_of_the_base: transfrom_of_the_base,
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
        }
    }

    pub fn set_flag_on_base_status(
        &mut self,
        effects_system: &mut EffectsSystem,
        audio_system: &mut AudioSystem,
        engine_handle: &mut EngineHandle,
    )
    {
        effects_system.spawn_wave(
            engine_handle,
            self.transform.get_position(),
            vec![
                FLAG_AREA_RADIUS,
                FLAG_AREA_RADIUS * 5.0,
                FLAG_AREA_RADIUS,
            ],
            vec![
                self.my_color,
                self.my_color,
                self.my_color
            ],
            vec![
                1.0,
                3.0,
            ]
        );

        self.transform = self.transfrom_of_the_base;
        self.target_position = self.transfrom_of_the_base.get_position();

        effects_system.spawn_wave(
            engine_handle,
            self.transform.get_position(),
            vec![
                FLAG_AREA_RADIUS,
                FLAG_AREA_RADIUS * 5.0,
                FLAG_AREA_RADIUS,
            ],
            vec![
                self.my_color,
                self.my_color,
                self.my_color
            ],
            vec![
                1.0,
                3.0,
            ]
        );

        audio_system.spawn_non_spatial_sound(
            Sound::FlagOnTheBase,
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
        effects_system: &mut EffectsSystem,
        engine_handle: &mut EngineHandle,
    )
    {
        self.target_position = pos;
        effects_system.spawn_wave(
            engine_handle,
            self.transform.get_position(),
            vec![
                FLAG_AREA_RADIUS,
                FLAG_AREA_RADIUS * 6.0,
            ],
            vec![
                self.my_color,
                Vec3::ZERO
            ],
            vec![
                3.0,
            ]
        );
        self.status = FlagStatus::Missed(pos);
    }

    pub fn set_flag_captured_status(
        &mut self,
        captured_by: ActorID,
        engine_handle: &mut EngineHandle,
        effects_system: &mut EffectsSystem,
        audio_system: &mut AudioSystem,
    )
    {
        self.area.clear_containing_colliders_list();
        
        engine_handle.send_direct_message(
            captured_by,
            Message {
                from: self.id.expect("Flag has no ActorID"),
                message: MessageType::SpecificActorMessage(
                    SpecificActorMessage::FlagMessage(
                        FlagMessage::GiveMeTargetPosition
                    )
                )
            }
        );

        effects_system.spawn_wave(
            engine_handle,
            self.transform.get_position(),
            vec![
                FLAG_AREA_RADIUS,
                FLAG_AREA_RADIUS * 3.0,
                FLAG_AREA_RADIUS * 6.0,
            ],
            vec![
                self.my_color,
                self.opponent_color,
                Vec3::ZERO
            ],
            vec![
                1.0,
                3.0,
            ]
        );
        
        audio_system.spawn_spatial_sound(
            Sound::FlagCuptured,
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
            physic_system: &crate::engine::physics::PhysicsSystem,
            engine_handle: &mut EngineHandle,
            audio_system: &mut crate::engine::audio::AudioSystem,
            ui_system: &mut crate::engine::ui::UISystem,
            time_system: &mut crate::engine::time::TimeSystem,
            effects_system: &mut EffectsSystem,
            delta: f32
        )
        {

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

            match self.owned_by_team
            {
                Team::Red =>
                {
                    let ui_flag = ui_system.get_mut_ui_element(&UIElementType::RedFlagMark);
    
                    *ui_flag.get_ui_data_mut().get_is_visible_cloned_arc().lock().unwrap() = self.flag_ui_tick_switch;
                }

                Team::Blue =>
                {
                    let ui_flag = ui_system.get_mut_ui_element(&UIElementType::BlueFlagMark);
    
                    *ui_flag.get_ui_data_mut().get_is_visible_cloned_arc().lock().unwrap() = self.flag_ui_tick_switch;
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
                delta * 8.0
            );

            current_flag_position += self.current_flag_swing_position;

            self.transform.set_position(current_flag_position);

            match self.status
            {
                FlagStatus::Captured(id) =>
                {
                    engine_handle.send_direct_message(
                        id,
                        Message {
                            from: self.get_id().expect("Flag have not ActorID"),
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
    }

    fn get_mut_transform(&mut self) -> &mut Transform {
        &mut self.transform
    }

    fn get_physical_element(&mut self) -> Option<PhysicalElement>
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


    fn get_visual_element(&self) -> Option<VisualElement>
    {
        Some(
            VisualElement
            {
                transform: &self.transform,
                static_objects: None,
                coloring_areas: None,
                volume_areas: Some(&self.visual_areas),
                waves: None,
                player: None,
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
            message: Message,
            engine_handle: &mut EngineHandle,
            physics_system: &PhysicsSystem,
            audio_system: &mut AudioSystem,
            ui_system: &mut UISystem,
            time_system: &TimeSystem,
            effects_system: &mut EffectsSystem,
        ) {
        
        let from = message.from;

        match message.message
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
                                            engine_handle.send_command(
                                                Command {
                                                    sender: self.get_id().expect("Player have not ActorID"),
                                                    command_type: CommandType::NetCommand(
                                                        NetCommand::SendMessageToServer(
                                                            NetMessageToServer::DropedFlag(
                                                                self.owned_by_team,
                                                                self.get_transform().get_position().to_array(),
                                                                in_space,
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

                            FlagMessage::SetFlagStatus(team, status) =>
                            {
                                if self.owned_by_team == team
                                {
                                    self.area.clear_containing_colliders_list();
                                    
                                    match status {
                                        FlagStatus::OnTheBase =>
                                        {
                                            self.set_flag_on_base_status(effects_system, audio_system, engine_handle);
                                        }
                                        FlagStatus::Missed(pos) =>
                                        {
                                            self.set_flag_missed_status(pos, effects_system, engine_handle);
                                        }
                                        FlagStatus::Captured(captured_by) =>
                                        {
                                            self.set_flag_captured_status(captured_by, engine_handle, effects_system, audio_system);
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
                                            self.target_position = position;
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
                                self.set_flag_on_base_status(effects_system, audio_system, engine_handle);
                            }
                            SessionControllerMessage::TeamWin(team) =>
                            {
                                match team {
                                    Team::Red =>
                                    {
                                        effects_system.spawn_wave(
                                            engine_handle,
                                            self.transform.get_position(),
                                            vec![
                                                FLAG_AREA_RADIUS,
                                                FLAG_AREA_RADIUS * 3.0,
                                                FLAG_AREA_RADIUS * 6.0,
                                            ],
                                            vec![
                                                self.my_color,
                                                RED_TEAM_COLOR,
                                                Vec3::ZERO
                                            ],
                                            vec![
                                                1.0,
                                                2.0,
                                            ]
                                        );
                                    }
                                    Team::Blue =>
                                    {
                                        effects_system.spawn_wave(
                                            engine_handle,
                                            self.transform.get_position(),
                                            vec![
                                                FLAG_AREA_RADIUS,
                                                FLAG_AREA_RADIUS * 3.0,
                                                FLAG_AREA_RADIUS * 6.0,
                                            ],
                                            vec![
                                                self.my_color,
                                                BLUE_TEAM_COLOR,
                                                Vec3::ZERO
                                            ],
                                            vec![
                                                1.0,
                                                2.0,
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
                                        match red_flag_status
                                        {
                                            FlagStatus::OnTheBase =>
                                            {
                                                self.set_flag_on_base_status(effects_system, audio_system, engine_handle);
                                            }

                                            FlagStatus::Missed(pos) =>
                                            {
                                                self.set_flag_missed_status(pos, effects_system, engine_handle);
                                            }

                                            FlagStatus::Captured(id) =>
                                            {
                                                self.set_flag_captured_status(id, engine_handle, effects_system, audio_system);
                                            }
                                        }
                                    }
                                    Team::Blue =>
                                    {
                                        match blue_flag_status
                                        {
                                            FlagStatus::OnTheBase =>
                                            {
                                                self.set_flag_on_base_status(effects_system, audio_system, engine_handle);
                                            }

                                            FlagStatus::Missed(pos) =>
                                            {
                                                self.set_flag_missed_status(pos, effects_system, engine_handle);
                                            }

                                            FlagStatus::Captured(id) =>
                                            {
                                                self.set_flag_captured_status(id, engine_handle, effects_system, audio_system);
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
                        self.transform = tr;
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
                                engine_handle.send_direct_message(
                                    id,
                                    Message {
                                        from: self.id.expect("Flag have not ActorID"),
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
                            AreaMessage::ActorIsContainedInsideArea(id) =>
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

