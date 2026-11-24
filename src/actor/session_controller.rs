// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2026  Timofei Molokov

mod net_session;

use crate::engine::shared_systems::resources::{ResourceLoader, Retention};
use fyrox_sound::buffer::SoundBufferResource;
use net_protocol::gameplay::{NetMessageToPlayer, NetMessageToServer, RemoteMessage, Team};
use fyrox_sound::source::Status;
use glam::{Vec3, Vec4};

use crate::{
    engine::{
        world::effects,
        shared_systems::net::Reliability,
        shared_systems::EngineSharedSystems,
        shared_systems::ui::{UIElementType, UISystem},
        world::{messenger::Messenger, WorldContext},
    },
    engine::transform::Transform
};

use self::net_session::NetSession;

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
    TeamWin(Team),
    GiveMeFlagsPostitions,
    ItIsFlagPosition(Team, Vec4),

    // Network data from other actors (V1: all game traffic goes through SessionController).
    // Messages that carry the session (server) time are stamped by SessionController.
    TryToGetOpponentsFlag,
    TryToGetScore,
    TryToReturnMyFlag,
    TryToGetMoveWBonus(
        // index of bonus spot
        u32
    ),
    SendToServer(NetMessageToServer),
    SendToPlayers(NetMessageToPlayer, Reliability),
    SendToPlayer(NetMessageToPlayer, ActorID, Reliability),
    SendPlayerDollState(
        // player's transform (serializable form)
        ([f32; 4], [f32; 16]),
        // simple input state (for extrapolation)
        (bool,bool,bool,bool,bool),
        // player's velocity
        [f32;4],
    ),
    ShowConnectionStatusUI,
}


/// Sends a message to the world's SessionController.
/// It's broadcast, so senders don't need to know the SessionController's id (and devices,
/// which aren't actors, can use it too). A world without a SessionController has no network.
pub fn send_to_session_controller(
    messenger: &mut Messenger,
    from: ActorID,
    message: SessionControllerMessage,
)
{
    messenger.send_broadcast_message(
        Message {
            from,
            remote_sender: false,
            message: MessageType::SpecificActorMessage(
                SpecificActorMessage::SessionControllerMessage(message)
            )
        }
    );
}


pub struct SessionController
{
    get_score_sound: Option<SoundBufferResource>,
    lost_score_sound: Option<SoundBufferResource>,
    team_lost_sound: Option<SoundBufferResource>,
    team_win_sound: Option<SoundBufferResource>,


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

    red_flag_base_position: Option<Vec4>,
    blue_flag_base_position: Option<Vec4>,

    observer_mode: bool,

    /// The game-session networking (formerly in NetSystem).
    net: NetSession,
    it_is_2d_3d_example: bool,
}


impl SessionController
{
    pub fn new(
        resources: &mut ResourceLoader,
        it_is_2d_3d_example: bool,
    ) -> Self
    {
        let session_controller = SessionController {
            get_score_sound: resources.load_media(
                crate::embedded_asset!("sounds/get_score.wav"), Retention::Session,
            ),
            lost_score_sound: resources.load_media(
                crate::embedded_asset!("sounds/lost_score.wav"), Retention::Session,
            ),
            team_lost_sound: resources.load_media(
                crate::embedded_asset!("sounds/team_lost.wav"), Retention::Session,
            ),
            team_win_sound: resources.load_media(
                crate::embedded_asset!("sounds/team_win.wav"), Retention::Session,
            ),

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
            red_flag_base_position: None,
            blue_flag_base_position: None,

            // temp
            observer_mode: false,

            net: NetSession::new(),
            it_is_2d_3d_example,
        };

        

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


impl SessionController
{
    /// Network data sent by other actors. Returns true if the message was one of them.
    fn handle_outgoing_net_message(
        &mut self,
        from: ActorID,
        message: &SessionControllerMessage,
        shared: &mut EngineSharedSystems,
    ) -> bool
    {
        let server_time = self.net.server_time();

        match message
        {
            SessionControllerMessage::TryToGetOpponentsFlag =>
            {
                self.net.send_to_server(NetMessageToServer::TryToGetOpponentsFlag(server_time), &mut shared.net);
            }
            SessionControllerMessage::TryToGetScore =>
            {
                self.net.send_to_server(NetMessageToServer::TryToGetScore(server_time), &mut shared.net);
            }
            SessionControllerMessage::TryToReturnMyFlag =>
            {
                self.net.send_to_server(NetMessageToServer::TryToReturnMyFlag(server_time), &mut shared.net);
            }
            SessionControllerMessage::TryToGetMoveWBonus(index) =>
            {
                self.net.send_to_server(NetMessageToServer::TryToGetMoveWBonus(server_time, *index), &mut shared.net);
            }
            SessionControllerMessage::SendToServer(message) =>
            {
                self.net.send_to_server(message.clone(), &mut shared.net);
            }
            SessionControllerMessage::SendToPlayers(message, reliability) =>
            {
                self.net.send_to_players(message.clone(), *reliability, &mut shared.net);
            }
            SessionControllerMessage::SendToPlayer(message, peer, reliability) =>
            {
                self.net.send_to_player(message.clone(), *peer, *reliability, &mut shared.net);
            }
            SessionControllerMessage::SendPlayerDollState(transform, input_state, velocity) =>
            {
                self.net.send_to_players(
                    NetMessageToPlayer::RemoteDirectMessage(
                        from,
                        RemoteMessage::SetPlayerDollState(
                            transform.clone(),
                            *input_state,
                            *velocity,
                            server_time,
                        )
                    ),
                    Reliability::Unreliable,
                    &mut shared.net,
                );
            }
            SessionControllerMessage::ShowConnectionStatusUI =>
            {
                self.net.set_connection_status_visible(true);
            }
            _ => return false,
        }

        true
    }
}


impl Actor for SessionController
{
    fn on_world_started(
        &mut self,
        world: &mut WorldContext,
        shared: &mut EngineSharedSystems,
    )
    {
        let messenger = &mut *world.messenger;
        let ui_system = &mut shared.ui;

        messenger.send_broadcast_message(
            Message {
                from: self.id.unwrap(),
                remote_sender: false,
                message: MessageType::SpecificActorMessage(
                    SpecificActorMessage::SessionControllerMessage(
                        SessionControllerMessage::GiveMeFlagsPostitions
                    )
                )
            }
        );

        messenger.send_broadcast_message(
            Message {
                from: self.id.unwrap(),
                remote_sender: false,
                message: MessageType::SpecificActorMessage(
                    SpecificActorMessage::SessionControllerMessage(
                        SessionControllerMessage::NewSessionStarted(Team::Blue)
                    )
                )
            }
        );

        

        self.set_score_ui(ui_system);
    }

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
        world: &mut WorldContext,
        shared: &mut EngineSharedSystems,
    ) {
        let delta = world.time.delta();
        let ui_system = &mut shared.ui;

        
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

        self.net.tick(
            self.id.expect("SessionController have not ActorID"),
            self.it_is_2d_3d_example,
            world,
            shared,
        );
    }

    fn recieve_message(
        &mut self,
        message: &Message,
        world: &mut WorldContext,
        shared: &mut EngineSharedSystems,
    ) {
        if let MessageType::SpecificActorMessage(SpecificActorMessage::SessionControllerMessage(session_message)) = &message.message
        {
            if self.handle_outgoing_net_message(message.from, session_message, shared)
            {
                return;
            }
        }

        let messenger = &mut *world.messenger;
        let audio_system = &mut shared.audio;
        let ui_system = &mut shared.ui;

        
        match &message.message {
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
                                self.your_team = *your_team;
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
                                if *new_red_team_score > self.red_team_score
                                {
                                    effects::spawn_wave(
                                        messenger,
                                        self.red_flag_base_position.expect("CTF Session Controller haven't a flag position"),
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
                                                &self.get_score_sound,
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
                                                    &self.get_score_sound,
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
                                                    &self.lost_score_sound,
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
                                if *new_blue_team_score > self.blue_team_score
                                {
                                    effects::spawn_wave(
                                        messenger,
                                        self.blue_flag_base_position.expect("CTF Session Controller haven't a flag position"),
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
                                                    &self.get_score_sound,
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
                                                    &self.lost_score_sound,
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
                                                &self.get_score_sound,
                                                1.0,
                                                1.0,
                                                false,
                                                true,
                                                Status::Playing
                                            );
                                        }
                                    }
                                }
                                self.red_team_score = *new_red_team_score;
                                self.blue_team_score = *new_blue_team_score;

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
                                                    &self.team_win_sound,
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
                                                        &self.team_win_sound,
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
                                                        &self.team_lost_sound,
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
                                                        &self.team_win_sound,
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
                                                        &self.team_lost_sound,
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
                                                    &self.team_win_sound,
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
                                _red_flag_status,
                                _blue_flag_status,
                                _bonus_spot_status,
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

                                self.your_team = *your_team;
                                self.red_team_score = *red_team_score;
                                self.blue_team_score = *blue_team_score;

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

                            SessionControllerMessage::ItIsFlagPosition(team, position) =>
                            {
                                match team
                                {
                                    Team::Blue =>
                                    {
                                        self.blue_flag_base_position = Some(*position);
                                    }
                                    Team::Red =>
                                    {
                                        self.red_flag_base_position = Some(*position);
                                    }
                                }
                            }
                            SessionControllerMessage::GiveMeFlagsPostitions => {}
                            _ => {}
                        }
                    }

                    SpecificActorMessage::PlayerMessage(message) =>
                    {
                        match message 
                        {
                            PlayerMessage::SetNewTeam(team) =>
                            {
                                self.your_team = *team;

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