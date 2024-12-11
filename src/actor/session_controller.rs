use client_server_protocol::Team;

use crate::{
    engine::{effects::EffectsSystem, engine_handle::EngineHandle, time::TimeSystem},
    transform::Transform
};

use super::{
    flag::FlagStatus, move_w_bonus::BonusSpotStatus, Actor, ActorID, Message, MessageType, SpecificActorMessage
};

pub const DEFAULT_TEAM: Team = Team::Red;

#[derive(Clone)]
pub enum SessionControllerMessage
{
    JoinedToSession(
        // your team
        Team,
        // red flag status
        FlagStatus,
        // blue flag status
        FlagStatus,
        // bonus spot status
        BonusSpotStatus,
        // red team score
        u32,
        // blue team score
        u32,
    ),
    NewSessionStarted(Team),
    SetScore(
        // red team score
        u32,
        // blue team score
        u32
    ),
    TeamWin(Team)
}


pub struct SessionController
{
    transform: Transform,
    id: Option<ActorID>,
    red_team_score: u32,
    blue_team_score: u32,
    your_team: Team,
}


impl SessionController
{
    pub fn new() -> Self
    {
        SessionController {
            transform: Transform::new(),
            id: None,
            red_team_score: 0u32,
            blue_team_score: 0u32,
            your_team: DEFAULT_TEAM,
        }
    }
}


impl Actor for SessionController
{
    fn get_mut_transform(&mut self) -> &mut Transform {
        &mut self.transform
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
            time_system: &TimeSystem,
            effects_system: &mut EffectsSystem,
        ) {
        
        match message.message {
            MessageType::SpecificActorMessage(message) =>
            {
                match message
                {
                    SpecificActorMessage::SessionControllerMessage(message) =>
                    {
                        match message {
                            SessionControllerMessage::NewSessionStarted(team) =>
                            {
                                self.blue_team_score = 0u32;
                                self.red_team_score = 0u32;
                                self.your_team = team;

                                todo!("set new ui score")
                            }

                            SessionControllerMessage::SetScore(new_red_team_score, new_blue_team_score) =>
                            {
                                if new_red_team_score > self.red_team_score
                                {
                                    match self.your_team {
                                        Team::Red =>
                                        {
                                            todo!("play good sound")
                                        }
                                        Team::Blue =>
                                        {
                                            todo!("play sad sound")
                                        }
                                    }
                                }
                                if new_blue_team_score > self.blue_team_score
                                {
                                    match self.your_team {
                                        Team::Red =>
                                        {
                                            todo!("play sad sound")
                                        }
                                        Team::Blue =>
                                        {
                                            todo!("play good sound")
                                        }
                                    }
                                }
                                self.red_team_score = new_red_team_score;
                                self.blue_team_score = new_blue_team_score;

                                todo!("set new ui score")
                            }

                            SessionControllerMessage::TeamWin(win_team) =>
                            {
                                match win_team
                                {
                                    Team::Red =>
                                    {
                                        match self.your_team {
                                            Team::Red =>
                                            {
                                                todo!("play good sound")
                                            }
                                            Team::Blue =>
                                            {
                                                todo!("play sad sound")
                                            }
                                        }
                                        todo!("Show red win ui")
                                    }
                                    Team::Blue =>
                                    {
                                        match self.your_team {
                                            Team::Red =>
                                            {
                                                todo!("play sad sound")
                                            }
                                            Team::Blue =>
                                            {
                                                todo!("play good sound")
                                            }
                                        }
                                        todo!("Show blue win ui")
                                    }
                                }
                            }

                            SessionControllerMessage::JoinedToSession(
                                your_team,
                                red_flag_status,
                                blue_flag_status,
                                bonus_spot_status,
                                red_team_score,
                                blue_team_score,
                            ) =>
                            {
                                println!("Joined to session");
                                
                                self.your_team = your_team;
                                self.red_team_score = red_team_score;
                                self.blue_team_score = blue_team_score;

                                todo!("set ui joined to team red or blue");
                                todo!("set ui score");
                            }
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}