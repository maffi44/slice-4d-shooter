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

use client_server_protocol::Team;
use fyrox_sound::source::Status;
use glam::{Vec3, Vec4};

use crate::{
    engine::{audio::Sound, effects::EffectsSystem, engine_handle::EngineHandle, time::TimeSystem, ui::{UIElementType, UISystem}},
    transform::Transform
};

use super::{
    flag::FlagStatus, main_player::{PlayerMessage, BLUE_TEAM_COLOR, RED_TEAM_COLOR}, move_w_bonus::BonusSpotStatus, Actor, ActorID, CommonActorsMessage, Message, MessageType, SpecificActorMessage
};

pub const DEFAULT_TEAM: Team = Team::Blue;

pub const SHOW_TEAM_BACKLIGHT_TIME: f32 = 4.0;
pub const SHOW_TEAM_JOIN_TITLE_TIME: f32 = 5.0;
pub const SHOW_TEAM_WIN_TITLE_TIME: f32 = 15.0;

pub const UI_ELEM_FADE_IN_SPEED: f32 = 3.0;
pub const UI_ELEM_FADE_OUT_SPEED: f32 = 3.0;

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
    show_red_team_backlight_timer: f32,
    show_blue_team_backlight_timer: f32,
    show_red_team_win_title_timer: f32,
    show_blue_team_win_title_timer: f32,
    show_join_red_team_title_timer: f32,
    show_join_blue_team_title_timer: f32,

    red_flag_base_position: Vec4,
    blue_flag_base_position: Vec4,

    observer_mode: bool,
}


impl SessionController
{
    pub fn new(
        ui_system: &mut UISystem,
        red_flag_base_position: Vec4,
        blue_flag_base_position: Vec4,
        observer_mode: bool,
    ) -> Self
    {
        let session_controller = SessionController {
            transform: Transform::new(),
            id: None,
            red_team_score: 4u32,
            blue_team_score: 4u32,
            your_team: DEFAULT_TEAM,
            show_red_team_backlight_timer: 0.0,
            show_blue_team_backlight_timer: 0.0,
            show_red_team_win_title_timer: 0.0,
            show_blue_team_win_title_timer: 0.0,
            show_join_red_team_title_timer: 0.0,
            show_join_blue_team_title_timer: 0.0,
            red_flag_base_position,
            blue_flag_base_position,
            observer_mode,
        };

        session_controller.set_score_ui(ui_system);

        session_controller
    }

    pub fn set_score_ui(&self, ui: &mut UISystem)
    {
        match self.red_team_score
        {
            0 =>
            {
                let score_mark = ui.get_mut_ui_element(&UIElementType::FirstScoreMarkRed);
                *score_mark.get_ui_data_mut().get_is_visible_mut() = false;

                let score_mark = ui.get_mut_ui_element(&UIElementType::SecondScoreMarkRed);
                *score_mark.get_ui_data_mut().get_is_visible_mut() = false;

                let score_mark = ui.get_mut_ui_element(&UIElementType::ThirdScoreMarkRed);
                *score_mark.get_ui_data_mut().get_is_visible_mut() = false;

                let score_mark = ui.get_mut_ui_element(&UIElementType::FinalScoreMarkRed);
                *score_mark.get_ui_data_mut().get_is_visible_mut() = false;
            }

            1 =>
            {
                let score_mark = ui.get_mut_ui_element(&UIElementType::FirstScoreMarkRed);
                *score_mark.get_ui_data_mut().get_is_visible_mut() = true;

                let score_mark = ui.get_mut_ui_element(&UIElementType::SecondScoreMarkRed);
                *score_mark.get_ui_data_mut().get_is_visible_mut() = false;

                let score_mark = ui.get_mut_ui_element(&UIElementType::ThirdScoreMarkRed);
                *score_mark.get_ui_data_mut().get_is_visible_mut() = false;

                let score_mark = ui.get_mut_ui_element(&UIElementType::FinalScoreMarkRed);
                *score_mark.get_ui_data_mut().get_is_visible_mut() = false;
            }

            2 =>
            {
                let score_mark = ui.get_mut_ui_element(&UIElementType::FirstScoreMarkRed);
                *score_mark.get_ui_data_mut().get_is_visible_mut() = true;

                let score_mark = ui.get_mut_ui_element(&UIElementType::SecondScoreMarkRed);
                *score_mark.get_ui_data_mut().get_is_visible_mut() = true;

                let score_mark = ui.get_mut_ui_element(&UIElementType::ThirdScoreMarkRed);
                *score_mark.get_ui_data_mut().get_is_visible_mut() = false;

                let score_mark = ui.get_mut_ui_element(&UIElementType::FinalScoreMarkRed);
                *score_mark.get_ui_data_mut().get_is_visible_mut() = false;
            }

            3 =>
            {
                let score_mark = ui.get_mut_ui_element(&UIElementType::FirstScoreMarkRed);
                *score_mark.get_ui_data_mut().get_is_visible_mut() = true;

                let score_mark = ui.get_mut_ui_element(&UIElementType::SecondScoreMarkRed);
                *score_mark.get_ui_data_mut().get_is_visible_mut() = true;

                let score_mark = ui.get_mut_ui_element(&UIElementType::ThirdScoreMarkRed);
                *score_mark.get_ui_data_mut().get_is_visible_mut() = true;

                let score_mark = ui.get_mut_ui_element(&UIElementType::FinalScoreMarkRed);
                *score_mark.get_ui_data_mut().get_is_visible_mut() = false;
            }

            4 =>
            {
                let score_mark = ui.get_mut_ui_element(&UIElementType::FirstScoreMarkRed);
                *score_mark.get_ui_data_mut().get_is_visible_mut() = true;

                let score_mark = ui.get_mut_ui_element(&UIElementType::SecondScoreMarkRed);
                *score_mark.get_ui_data_mut().get_is_visible_mut() = true;

                let score_mark = ui.get_mut_ui_element(&UIElementType::ThirdScoreMarkRed);
                *score_mark.get_ui_data_mut().get_is_visible_mut() = true;

                let score_mark = ui.get_mut_ui_element(&UIElementType::FinalScoreMarkRed);
                *score_mark.get_ui_data_mut().get_is_visible_mut() = true;
            }

            _ => {panic!("ERROR: red team score > 4")}
        };

        match self.blue_team_score
        {
            0 =>
            {
                let score_mark = ui.get_mut_ui_element(&UIElementType::FirstScoreMarkBlue);
                *score_mark.get_ui_data_mut().get_is_visible_mut() = false;

                let score_mark = ui.get_mut_ui_element(&UIElementType::SecondScoreMarkBlue);
                *score_mark.get_ui_data_mut().get_is_visible_mut() = false;

                let score_mark = ui.get_mut_ui_element(&UIElementType::ThirdScoreMarkBlue);
                *score_mark.get_ui_data_mut().get_is_visible_mut() = false;

                let score_mark = ui.get_mut_ui_element(&UIElementType::FinalScoreMarkBlue);
                *score_mark.get_ui_data_mut().get_is_visible_mut() = false;
            }

            1 =>
            {
                let score_mark = ui.get_mut_ui_element(&UIElementType::FirstScoreMarkBlue);
                *score_mark.get_ui_data_mut().get_is_visible_mut() = true;

                let score_mark = ui.get_mut_ui_element(&UIElementType::SecondScoreMarkBlue);
                *score_mark.get_ui_data_mut().get_is_visible_mut() = false;

                let score_mark = ui.get_mut_ui_element(&UIElementType::ThirdScoreMarkBlue);
                *score_mark.get_ui_data_mut().get_is_visible_mut() = false;

                let score_mark = ui.get_mut_ui_element(&UIElementType::FinalScoreMarkBlue);
                *score_mark.get_ui_data_mut().get_is_visible_mut() = false;
            }

            2 =>
            {
                let score_mark = ui.get_mut_ui_element(&UIElementType::FirstScoreMarkBlue);
                *score_mark.get_ui_data_mut().get_is_visible_mut() = true;

                let score_mark = ui.get_mut_ui_element(&UIElementType::SecondScoreMarkBlue);
                *score_mark.get_ui_data_mut().get_is_visible_mut() = true;

                let score_mark = ui.get_mut_ui_element(&UIElementType::ThirdScoreMarkBlue);
                *score_mark.get_ui_data_mut().get_is_visible_mut() = false;

                let score_mark = ui.get_mut_ui_element(&UIElementType::FinalScoreMarkBlue);
                *score_mark.get_ui_data_mut().get_is_visible_mut() = false;
            }

            3 =>
            {
                let score_mark = ui.get_mut_ui_element(&UIElementType::FirstScoreMarkBlue);
                *score_mark.get_ui_data_mut().get_is_visible_mut() = true;

                let score_mark = ui.get_mut_ui_element(&UIElementType::SecondScoreMarkBlue);
                *score_mark.get_ui_data_mut().get_is_visible_mut() = true;

                let score_mark = ui.get_mut_ui_element(&UIElementType::ThirdScoreMarkBlue);
                *score_mark.get_ui_data_mut().get_is_visible_mut() = true;

                let score_mark = ui.get_mut_ui_element(&UIElementType::FinalScoreMarkBlue);
                *score_mark.get_ui_data_mut().get_is_visible_mut() = false;
            }

            4 =>
            {
                let score_mark = ui.get_mut_ui_element(&UIElementType::FirstScoreMarkBlue);
                *score_mark.get_ui_data_mut().get_is_visible_mut() = true;

                let score_mark = ui.get_mut_ui_element(&UIElementType::SecondScoreMarkBlue);
                *score_mark.get_ui_data_mut().get_is_visible_mut() = true;

                let score_mark = ui.get_mut_ui_element(&UIElementType::ThirdScoreMarkBlue);
                *score_mark.get_ui_data_mut().get_is_visible_mut() = true;

                let score_mark = ui.get_mut_ui_element(&UIElementType::FinalScoreMarkBlue);
                *score_mark.get_ui_data_mut().get_is_visible_mut() = true;
            }

            _ => {panic!("ERROR: blue team score > 4")}
        };
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

    fn tick(
            &mut self,
            physic_system: &crate::engine::physics::PhysicsSystem,
            engine_handle: &mut EngineHandle,
            audio_system: &mut crate::engine::audio::AudioSystem,
            ui_system: &mut UISystem,
            time_system: &mut TimeSystem,
            effects_system: &mut EffectsSystem,
            delta: f32
        ) {
        
        process_ui_animation(
            &mut self.show_red_team_backlight_timer,
            &UIElementType::RedTeamBacklight,
            delta,
            ui_system,
        );

        process_ui_animation(
            &mut self.show_blue_team_backlight_timer,
            &UIElementType::BlueTeamBacklight,
            delta,
            ui_system,
        );

        process_ui_animation(
            &mut self.show_red_team_win_title_timer,
            &UIElementType::RedTeamWinTitle,
            delta,
            ui_system,
        );

        process_ui_animation(
            &mut self.show_blue_team_win_title_timer,
            &UIElementType::BlueTeamWinTitle,
            delta,
            ui_system,
        );

        process_ui_animation(
            &mut self.show_join_red_team_title_timer,
            &UIElementType::JoinRedTeamTitle,
            delta,
            ui_system,
        );

        process_ui_animation(
            &mut self.show_join_blue_team_title_timer,
            &UIElementType::JoinBlueTeamTitle,
            delta,
            ui_system,
        );
        
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
                            SessionControllerMessage::NewSessionStarted(your_team) =>
                            {
                                self.blue_team_score = 0u32;
                                self.red_team_score = 0u32;
                                self.your_team = your_team;
                                self.show_blue_team_backlight_timer = 0.0;
                                self.show_red_team_backlight_timer = 0.0;
                                self.show_join_blue_team_title_timer = 0.0;
                                self.show_join_red_team_title_timer = 0.0;
                                self.show_blue_team_win_title_timer = 0.0;
                                self.show_red_team_win_title_timer = 0.0;

                                match your_team
                                {
                                    Team::Red =>
                                    {
                                        let elem = ui_system.get_mut_ui_element(&UIElementType::RedTeamBacklight);
                                        *elem.get_ui_data_mut().get_is_visible_mut() = true;
                                        
                                        let elem = ui_system.get_mut_ui_element(&UIElementType::JoinRedTeamTitle);
                                        *elem.get_ui_data_mut().get_is_visible_mut() = true;

                                        self.show_red_team_backlight_timer = SHOW_TEAM_JOIN_TITLE_TIME;
                                        self.show_join_red_team_title_timer = SHOW_TEAM_JOIN_TITLE_TIME;

                                    }

                                    Team::Blue =>
                                    {
                                        let elem = ui_system.get_mut_ui_element(&UIElementType::BlueTeamBacklight);
                                        *elem.get_ui_data_mut().get_is_visible_mut() = true;
                                        
                                        let elem = ui_system.get_mut_ui_element(&UIElementType::JoinBlueTeamTitle);
                                        *elem.get_ui_data_mut().get_is_visible_mut() = true;

                                        self.show_blue_team_backlight_timer = SHOW_TEAM_JOIN_TITLE_TIME;
                                        self.show_join_blue_team_title_timer = SHOW_TEAM_JOIN_TITLE_TIME;

                                    }
                                }

                                self.set_score_ui(ui_system);
                            }

                            SessionControllerMessage::SetScore(new_red_team_score, new_blue_team_score) =>
                            {
                                if new_red_team_score > self.red_team_score
                                {
                                    effects_system.spawn_wave(
                                        engine_handle,
                                        self.red_flag_base_position,
                                        vec![
                                            0.0,
                                            15.0,
                                        ],
                                        vec![
                                            RED_TEAM_COLOR,
                                            Vec3::ZERO
                                        ],
                                        vec![
                                            2.0,
                                        ]
                                    );

                                    let elem = ui_system.get_mut_ui_element(&UIElementType::RedTeamBacklight);
                                    *elem.get_ui_data_mut().get_is_visible_mut() = true;

                                    self.show_red_team_backlight_timer = SHOW_TEAM_BACKLIGHT_TIME;

                                    match self.your_team {
                                        Team::Red =>
                                        {
                                            audio_system.spawn_non_spatial_sound(
                                                Sound::GetScore,
                                                1.0,
                                                1.0,
                                                false,
                                                true,
                                                Status::Playing
                                            );
                                        }
                                        Team::Blue =>
                                        {
                                            if self.observer_mode
                                            {
                                                audio_system.spawn_non_spatial_sound(
                                                    Sound::GetScore,
                                                    1.0,
                                                    1.0,
                                                    false,
                                                    true,
                                                    Status::Playing
                                                );
                                            }
                                            else
                                            {
                                                audio_system.spawn_non_spatial_sound(
                                                    Sound::LooseScore,
                                                    1.0,
                                                    1.0,
                                                    false,
                                                    true,
                                                    Status::Playing
                                                ); 
                                            }
                                        }
                                    }
                                }
                                if new_blue_team_score > self.blue_team_score
                                {
                                    effects_system.spawn_wave(
                                        engine_handle,
                                        self.blue_flag_base_position,
                                        vec![
                                            0.0,
                                            15.0,
                                        ],
                                        vec![
                                            BLUE_TEAM_COLOR,
                                            Vec3::ZERO
                                        ],
                                        vec![
                                            2.0,
                                        ]
                                    );

                                    let elem = ui_system.get_mut_ui_element(&UIElementType::BlueTeamBacklight);
                                    *elem.get_ui_data_mut().get_is_visible_mut() = true;

                                    self.show_blue_team_backlight_timer = SHOW_TEAM_BACKLIGHT_TIME;

                                    match self.your_team {
                                        Team::Red =>
                                        {
                                            if self.observer_mode
                                            {
                                                audio_system.spawn_non_spatial_sound(
                                                    Sound::GetScore,
                                                    1.0,
                                                    1.0,
                                                    false,
                                                    true,
                                                    Status::Playing
                                                );
                                            }
                                            else
                                            {
                                                audio_system.spawn_non_spatial_sound(
                                                    Sound::LooseScore,
                                                    1.0,
                                                    1.0,
                                                    false,
                                                    true,
                                                    Status::Playing
                                                ); 
                                            }
                                        }
                                        Team::Blue =>
                                        {
                                            audio_system.spawn_non_spatial_sound(
                                                Sound::GetScore,
                                                1.0,
                                                1.0,
                                                false,
                                                true,
                                                Status::Playing
                                            );
                                        }
                                    }
                                }
                                self.red_team_score = new_red_team_score;
                                self.blue_team_score = new_blue_team_score;

                                self.set_score_ui(ui_system);
                            }

                            SessionControllerMessage::TeamWin(win_team) =>
                            {
                                self.show_blue_team_backlight_timer = 0.0;
                                self.show_red_team_backlight_timer = 0.0;
                                self.show_join_blue_team_title_timer = 0.0;
                                self.show_join_red_team_title_timer = 0.0;
                                self.show_blue_team_win_title_timer = 0.0;
                                self.show_red_team_win_title_timer = 0.0;

                                match win_team
                                {
                                    Team::Red =>
                                    {
                                        let elem = ui_system.get_mut_ui_element(&UIElementType::RedTeamBacklight);
                                        *elem.get_ui_data_mut().get_is_visible_mut() = true;
                                        
                                        let elem = ui_system.get_mut_ui_element(&UIElementType::RedTeamWinTitle);
                                        *elem.get_ui_data_mut().get_is_visible_mut() = true;

                                        self.show_red_team_backlight_timer = SHOW_TEAM_WIN_TITLE_TIME;
                                        self.show_red_team_win_title_timer = SHOW_TEAM_WIN_TITLE_TIME;

                                        match self.your_team {
                                            Team::Red =>
                                            {
                                                audio_system.spawn_non_spatial_sound(
                                                    Sound::TeamWin,
                                                    1.0,
                                                    1.0,
                                                    false,
                                                    true,
                                                    Status::Playing
                                                );  
                                            }
                                            Team::Blue =>
                                            {
                                                if self.observer_mode
                                                {
                                                    audio_system.spawn_non_spatial_sound(
                                                        Sound::TeamWin,
                                                        1.0,
                                                        1.0,
                                                        false,
                                                        true,
                                                        Status::Playing
                                                    );
                                                }
                                                else
                                                {
                                                    audio_system.spawn_non_spatial_sound(
                                                        Sound::TeamLoose,
                                                        1.0,
                                                        1.0,
                                                        false,
                                                        true,
                                                        Status::Playing
                                                    );
                                                }
                                            }
                                        }
                                    }
                                    Team::Blue =>
                                    {
                                        let elem = ui_system.get_mut_ui_element(&UIElementType::BlueTeamBacklight);
                                        *elem.get_ui_data_mut().get_is_visible_mut() = true;
                                        
                                        let elem = ui_system.get_mut_ui_element(&UIElementType::BlueTeamWinTitle);
                                        *elem.get_ui_data_mut().get_is_visible_mut() = true;

                                        self.show_blue_team_backlight_timer = SHOW_TEAM_WIN_TITLE_TIME;
                                        self.show_blue_team_win_title_timer = SHOW_TEAM_WIN_TITLE_TIME;

                                        match self.your_team {
                                            Team::Red =>
                                            {
                                                if self.observer_mode
                                                {
                                                    audio_system.spawn_non_spatial_sound(
                                                        Sound::TeamWin,
                                                        1.0,
                                                        1.0,
                                                        false,
                                                        true,
                                                        Status::Playing
                                                    );
                                                }
                                                else
                                                {
                                                    audio_system.spawn_non_spatial_sound(
                                                        Sound::TeamLoose,
                                                        1.0,
                                                        1.0,
                                                        false,
                                                        true,
                                                        Status::Playing
                                                    );
                                                } 
                                            }
                                            Team::Blue =>
                                            {
                                                audio_system.spawn_non_spatial_sound(
                                                    Sound::TeamWin,
                                                    1.0,
                                                    1.0,
                                                    false,
                                                    true,
                                                    Status::Playing
                                                );
                                            }
                                        }
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
                                
                                self.show_blue_team_backlight_timer = 0.0;
                                self.show_red_team_backlight_timer = 0.0;
                                self.show_join_blue_team_title_timer = 0.0;
                                self.show_join_red_team_title_timer = 0.0;
                                self.show_blue_team_win_title_timer = 0.0;
                                self.show_red_team_win_title_timer = 0.0;

                                self.your_team = your_team;
                                self.red_team_score = red_team_score;
                                self.blue_team_score = blue_team_score;

                                match your_team
                                {
                                    Team::Red =>
                                    {
                                        let elem = ui_system.get_mut_ui_element(&UIElementType::RedTeamBacklight);
                                        *elem.get_ui_data_mut().get_is_visible_mut() = true;
                                        
                                        let elem = ui_system.get_mut_ui_element(&UIElementType::JoinRedTeamTitle);
                                        *elem.get_ui_data_mut().get_is_visible_mut() = true;
                                        
                                        self.show_red_team_backlight_timer = SHOW_TEAM_JOIN_TITLE_TIME;
                                        self.show_join_red_team_title_timer = SHOW_TEAM_JOIN_TITLE_TIME;

                                    }

                                    Team::Blue =>
                                    {
                                        let elem = ui_system.get_mut_ui_element(&UIElementType::BlueTeamBacklight);
                                        *elem.get_ui_data_mut().get_is_visible_mut() = true;
                                        
                                        let elem = ui_system.get_mut_ui_element(&UIElementType::JoinBlueTeamTitle);
                                        *elem.get_ui_data_mut().get_is_visible_mut() = true;

                                        self.show_blue_team_backlight_timer = SHOW_TEAM_JOIN_TITLE_TIME;
                                        self.show_join_blue_team_title_timer = SHOW_TEAM_JOIN_TITLE_TIME;

                                    }
                                }
                                self.set_score_ui(ui_system);
                            }
                        }
                    }

                    SpecificActorMessage::PlayerMessage(message) =>
                    {
                        match message 
                        {
                            PlayerMessage::SetNewTeam(team) =>
                            {
                                self.your_team = team;

                                match team {
                                    Team::Red =>
                                    {
                                        let elem = ui_system.get_mut_ui_element(&UIElementType::RedTeamBacklight);
                                        *elem.get_ui_data_mut().get_is_visible_mut() = true;
                                        
                                        let elem = ui_system.get_mut_ui_element(&UIElementType::JoinRedTeamTitle);
                                        *elem.get_ui_data_mut().get_is_visible_mut() = true;

                                        self.show_red_team_backlight_timer = SHOW_TEAM_JOIN_TITLE_TIME;
                                        self.show_join_red_team_title_timer = SHOW_TEAM_JOIN_TITLE_TIME;
                                    }
                                    Team::Blue =>
                                    {
                                        let elem = ui_system.get_mut_ui_element(&UIElementType::BlueTeamBacklight);
                                        *elem.get_ui_data_mut().get_is_visible_mut() = true;
                                        
                                        let elem = ui_system.get_mut_ui_element(&UIElementType::JoinBlueTeamTitle);
                                        *elem.get_ui_data_mut().get_is_visible_mut() = true;

                                        self.show_red_team_backlight_timer = SHOW_TEAM_JOIN_TITLE_TIME;
                                        self.show_join_red_team_title_timer = SHOW_TEAM_JOIN_TITLE_TIME;
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
                match message
                {
                    CommonActorsMessage::ClientDisconnectedFromGameServer =>
                    {
                        self.show_blue_team_backlight_timer = 0.0;
                        self.show_red_team_backlight_timer = 0.0;
                        self.show_join_blue_team_title_timer = 0.0;
                        self.show_join_red_team_title_timer = 0.0;
                        self.show_blue_team_win_title_timer = 0.0;
                        self.show_red_team_win_title_timer = 0.0;
                        self.red_team_score = 4;
                        self.blue_team_score = 4;
                        self.set_score_ui(ui_system);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

fn process_ui_animation(
    timer: &mut f32,
    ui_elem_type: &UIElementType,
    delta: f32,
    ui_system: &mut UISystem
)
{
    if *timer > 0.0
        {
            *timer -= delta;

            let ui_elem = ui_system.get_mut_ui_element(ui_elem_type);

            let transparency = ui_elem.get_ui_data().get_transparency();

            if transparency < 1.0
            {
                ui_elem.get_ui_data_mut().set_transparency(
                    transparency + (UI_ELEM_FADE_IN_SPEED * delta)
                );
            }
            else
            {
                ui_elem.get_ui_data_mut().set_transparency(
                    1.0
                );
            }
        }
        else {
            *timer = 0.0;

            let ui_elem = ui_system.get_mut_ui_element(ui_elem_type);
            let transparency = ui_elem.get_ui_data().get_transparency();

            if transparency > 0.0
            {
                ui_elem.get_ui_data_mut().set_transparency(
                    transparency - (UI_ELEM_FADE_OUT_SPEED * delta)
                );
            }
            else
            {
                ui_elem.get_ui_data_mut().set_transparency(
                    0.0
                );

                *ui_elem.get_ui_data_mut().get_is_visible_mut() = false;
            }
        }
}