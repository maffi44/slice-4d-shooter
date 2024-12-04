use client_server_protocol::Team;
use glam::Vec4;
use rand::Rng;
use web_sys::console::assert;

use crate::{
    engine::{
        engine_handle::EngineHandle,
        physics::{area::AreaMessages, colliders_container::PhysicalElement},
        render::VisualElement
    },
    transform::Transform
};

use super::{
    session_controller::SessionControllerMessage, Actor, ActorID, CommonActorsMessages, Message, MessageType, PhysicsMessages, SpecificActorMessage
};

#[derive(Clone)]
pub enum FlagMessage
{
    SetFlagStatus(Team, FlagStatus),
    YouTryingToGetFlag(Team, FlagStatus),
    GiveMeTargetPosition,
    SetTargetPosition(Vec4),
}

#[derive(Clone, Copy)]
enum FlagStatus
{
    Captured(ActorID),
    Missed(Vec4),
    OnTheBase,
}

const TIME_TO_CHANGE_NEXT_TARGET_SWING_POSITION: f32 = 3.0;
const FLAG_SWING_RANGE: f32 = 0.1;

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
}

impl Flag
{
    pub fn new(team: Team, transfrom_of_the_base: Transform) -> Self
    {
        let target_flag_swing_position = get_random_vec4(
            -FLAG_SWING_RANGE,
            FLAG_SWING_RANGE
        );

        Flag {
            transform: transfrom_of_the_base,
            target_flag_swing_position,
            current_flag_swing_position: Vec4::ZERO,
            target_position: transfrom_of_the_base.get_position(),
            transfrom_of_the_base: transfrom_of_the_base,
            id: None,
            status:FlagStatus::OnTheBase,
            owned_by_team: team,
            next_target_swing_position_in_secs: TIME_TO_CHANGE_NEXT_TARGET_SWING_POSITION,
        }
    }

    pub fn set_flag_on_base_status(
        &mut self
    )
    {
        self.transform = self.transfrom_of_the_base;
        self.target_position = self.transfrom_of_the_base.get_position();
        todo!("play effect on base");
        todo!("play status on base");
        self.status = FlagStatus::OnTheBase;
    }

    pub fn set_flag_missed_status(
        &mut self,
        pos: Vec4,
    )
    {
        self.target_position = pos;
        todo!("play effect missed");
        todo!("play status missed");
        self.status = FlagStatus::Missed(pos);
    }

    pub fn set_flag_captured_status(
        &mut self,
        captured_by: ActorID,
        engine_handle: &mut EngineHandle
    )
    {
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

        todo!("play effect captured");
        todo!("play status captured");
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
            delta: f32
        ) {
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
                1.0 - (delta * 15.0)
            );

            let mut current_flag_position = self.transform.get_position();

            current_flag_position = current_flag_position.lerp(
                self.target_position,
                1.0 - (delta * 8.0)
            );

            current_flag_position += self.current_flag_swing_position;

            self.transform.set_position(current_flag_position);
    }

    fn get_mut_transform(&mut self) -> &mut Transform {
        &mut self.transform
    }

    fn get_visual_element(&self) -> Option<VisualElement> {
        match self.status {
            FlagStatus::Captured(_) =>
            {
                None
            }
            _ =>
            {
                todo!()
            }
        }
    }

    fn get_physical_element(&mut self) -> Option<PhysicalElement> {
        match self.status {
            FlagStatus::Captured(_) =>
            {
                None
            }
            _ =>
            {
                todo!()
            }
        }
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
            physics_system: &crate::engine::physics::PhysicsSystem,
            audio_system: &mut crate::engine::audio::AudioSystem,
            ui_system: &mut crate::engine::ui::UISystem,
        ) {
        
        match message.message
        {
            MessageType::SpecificActorMessage(message) =>
            {
                match message {
                    SpecificActorMessage::FlagMessage(message) =>
                    {
                        match message
                        {
                            FlagMessage::SetFlagStatus(team, status) =>
                            {
                                if self.owned_by_team == team
                                {
                                    match status {
                                        FlagStatus::OnTheBase =>
                                        {
                                            self.set_flag_on_base_status();
                                        }
                                        FlagStatus::Missed(pos) =>
                                        {
                                            self.set_flag_missed_status(pos);
                                        }
                                        FlagStatus::Captured(captured_by) =>
                                        {
                                            self.set_flag_captured_status(captured_by, engine_handle);
                                        }
                                    }
                                }
                            }
                            FlagMessage::SetTargetPosition(position) =>
                            {
                                self.target_position = position;
                            }
                            
                            _ => {}
                        }
                    }
                    SpecificActorMessage::SessionControllerMessage(message) =>
                    {
                        match message {
                            SessionControllerMessage::NewSessionStarted(_) =>
                            {
                                self.set_flag_on_base_status();
                            }
                            SessionControllerMessage::TeamWin(team) =>
                            {
                                match team {
                                    Team::Red =>
                                    {
                                        todo!("play red win effect")
                                    }
                                    Team::Blue =>
                                    {
                                        todo!("play blue win effect")
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
                    CommonActorsMessages::SetTransform(tr) =>
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
                            AreaMessages::ActorEnterArea(id) =>
                            {
                                engine_handle.send_direct_message(
                                    id,
                                    Message {
                                        from: self.id.expect("Flag have not ActorID"),
                                        message: MessageType::SpecificActorMessage(
                                            SpecificActorMessage::FlagMessage(
                                                FlagMessage::YouTryingToGetFlag(
                                                    self.owned_by_team,
                                                    self.status
                                                )
                                            )
                                        )
                                    }
                                );
                            }
                            AreaMessages::ActorIsContainedInsideArea(id) =>
                            {
                                todo!("write logic of resending message after some time inside")
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

