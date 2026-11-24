// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2026  Timofei Molokov

// The game-session networking of `SessionController` (moved out of the former `NetSystem`):
// the connection flow (matchmaking request over WebSocket, then the game server over WebRTC),
// decoding server messages into actor messages, the connection-status UI and the session (server) time.
// The transport itself is `NetSystem` (connection handles, raw packets).

use crate::engine::shared_systems::resources::ResourceLoader;
use std::time::Duration;

use glam::{Vec3, Vec4};
use web_time::Instant;

use client_server_protocol::{
    ClientMessage,
    NetMessageToPlayer,
    NetMessageToServer,
    RemoteCommand,
    RemoteMessage,
    ServerMessage,
};

use matchmaking_server_protocol::{
    ClientMatchmakingServerProtocol,
    GameType,
    GameVersion,
    MatchmakingServerMessage,
};

use crate::{
    actor::{
        flag::{
            FlagMessage, FlagStatus
        },
        hole::Hole,
        main_player::PlayerMessage,
        move_w_bonus::{
            BonusSpotStatus, MoveWBonusSpotMessage
        },
        players_death_explosion::PlayersDeathExplosion,
        players_doll::{
            PlayerDoll, PlayerDollInputState, PlayersDollMessage
        },
        session_controller::{self, SessionControllerMessage},
        ActorID,
        ActorWrapper,
        CommonActorsMessage,
        Message,
        MessageType,
        SpecificActorMessage,
    },
    engine::{
        shared_systems::global_settings::ConnectionSettings,
        shared_systems::net::{
            ConnectionConfig,
            ConnectionError as NetConnectionError,
            ConnectionHandle,
            ConnectionState,
            DisconnectReason,
            NetSystem,
            Reliability,
        },
        shared_systems::EngineSharedSystems,
        shared_systems::ui::{
            UIElementType,
            UISystem
        },
        world::{
            messenger::{
                Command,
                CommandType,
                Messenger,
                OUTER_WORLD_SENDER_ID,
            },
            WorldContext,
        },
    },
    engine::transform::Transform
};


const VERSION: &str = env!("CARGO_PKG_VERSION");

const MATCHMAKING_CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const MATCHMAKING_CONNECTION_ATTEMPTS: u64 = 2;
const GAME_SERVER_CONNECTION_TIMEOUT_TICKS: u64 = 240;


#[derive(Debug)]
enum ConnectionError {
    #[allow(dead_code)] // payload kept for diagnostics
    WrongVersion(GameVersion),
    NoFreeServers,
    MatchmakingServerClientProtocolError,
    ConnectionLost,
    ConnectionClosedByServer,
    ConnectionTimeout,
}


/// The session's connection state machine (formerly `NetSystem`'s `ConnectionState`).
enum SessionState {
    WaitingForUsersRequest,
    ConnectionFailure(u32, ConnectionError),
    ConnectingToMatchmakingServer {
        connection: Option<ConnectionHandle>,
        request_sent: bool,
        attempts_left: u64,
    },
    ConnectingToGameServer {
        connection: Option<ConnectionHandle>,
        timeout_counter: u64,
        attempts_left: u64,
    },
    ConnectedToGameServer {
        connection: ConnectionHandle,
        players_id: Vec<u128>,
    },
}


/// Server time of the current game session (formerly in the shared `TimeSystem`).
struct SessionTime {
    server_time_at_join_ms: u128,
    joined_at: Instant,
}

impl SessionTime {
    fn server_time_ms(&self) -> u128 {
        self.joined_at.elapsed().as_millis() + self.server_time_at_join_ms
    }
}


pub struct NetSession {
    state: Option<SessionState>,
    game_server_url: Option<String>,
    session_time: Option<SessionTime>,
    current_visible_ui_elem: UIElementType,
    connection_status_visible: bool,
}


impl NetSession {

    pub fn new() -> Self {
        NetSession {
            state: Some(SessionState::WaitingForUsersRequest),
            game_server_url: None,
            session_time: None,
            current_visible_ui_elem: UIElementType::TitlePressPToPlayOnline,
            connection_status_visible: false,
        }
    }

    pub fn set_connection_status_visible(&mut self, visible: bool) {
        self.connection_status_visible = visible;
    }

    /// Server time of the session in milliseconds (0 if not joined).
    pub fn server_time(&self) -> u128 {
        match &self.session_time {
            Some(session_time) => session_time.server_time_ms(),
            None => 0u128,
        }
    }

    /// Sends a message to the game server (if connected).
    pub fn send_to_server(&self, message: NetMessageToServer, net: &mut NetSystem) {
        if let Some(SessionState::ConnectedToGameServer { connection, .. }) = &self.state {
            // a lost connection is handled in `tick`, where `receive` reports it
            let _ = net.send(
                connection,
                ClientMessage::MessageToServer(message).to_packet(),
                Reliability::Reliable,
            );
        }
    }

    /// Sends a message to all players through the game server (if connected).
    pub fn send_to_players(&self, message: NetMessageToPlayer, reliability: Reliability, net: &mut NetSystem) {
        if let Some(SessionState::ConnectedToGameServer { connection, .. }) = &self.state {
            // a lost connection is handled in `tick`, where `receive` reports it
            let _ = net.send(
                connection,
                ClientMessage::BoardcastMessageToPlayers(message).to_packet(),
                reliability,
            );
        }
    }

    /// Sends a message to one player through the game server (if connected).
    pub fn send_to_player(&self, message: NetMessageToPlayer, peer: u128, reliability: Reliability, net: &mut NetSystem) {
        if let Some(SessionState::ConnectedToGameServer { connection, .. }) = &self.state {
            // a lost connection is handled in `tick`, where `receive` reports it
            let _ = net.send(
                connection,
                ClientMessage::DirectMessageToPlayer(peer, message).to_packet(),
                reliability,
            );
        }
    }


    /// Called from `SessionController::tick` every tick.
    pub fn tick(
        &mut self,
        session_controller_id: ActorID,
        it_is_2d_3d_example: bool,
        world: &mut WorldContext,
        shared: &mut EngineSharedSystems,
    ) {
        let state = self.state.take().expect("ERROR: session state is None");

        let new_state = match state {
            SessionState::WaitingForUsersRequest => {
                self.handle_waiting_for_user_input(shared)
            }

            SessionState::ConnectionFailure(timer, reason) => {
                self.handle_connection_failure(timer, reason, world.messenger, &mut shared.ui)
            }

            SessionState::ConnectingToMatchmakingServer { connection, request_sent, attempts_left } => {
                self.handle_connecting_to_matchmaking_server(
                    connection,
                    request_sent,
                    attempts_left,
                    it_is_2d_3d_example,
                    shared,
                )
            }

            SessionState::ConnectingToGameServer { connection, timeout_counter, attempts_left } => {
                self.handle_connecting_to_game_server(
                    connection,
                    timeout_counter,
                    attempts_left,
                    session_controller_id,
                    world,
                    shared,
                )
            }

            SessionState::ConnectedToGameServer { connection, players_id } => {
                self.handle_connected_to_game_server(
                            connection,
                    players_id,
                    it_is_2d_3d_example,
                    world,
                    shared,
                )
            }
        };

        self.state = Some(new_state);
    }


    fn hide_current_ui_elem(&mut self, ui_system: &mut UISystem) {
        *ui_system.get_mut_ui_element(&self.current_visible_ui_elem)
            .get_ui_data_mut()
            .get_is_visible_mut() = false;
    }

    fn show_ui_elem(&mut self, ui_elem: UIElementType, ui_system: &mut UISystem) {
        *ui_system.get_mut_ui_element(&ui_elem)
            .get_ui_data_mut()
            .get_is_visible_mut() = true;

        self.current_visible_ui_elem = ui_elem;
    }


    fn handle_connection_failure(
        &mut self,
        mut timer: u32,
        reason: ConnectionError,
        messenger: &mut Messenger,
        ui_system: &mut UISystem,
    ) -> SessionState
    {
        self.hide_current_ui_elem(ui_system);

        let ui_elem = match &reason {
            ConnectionError::WrongVersion(_) => UIElementType::TitleConnectionFailedOldVersion,
            ConnectionError::NoFreeServers => UIElementType::TitleConnectionFailedServerIsFull,
            ConnectionError::MatchmakingServerClientProtocolError => UIElementType::TitleConnectionFailedServerError,
            ConnectionError::ConnectionTimeout => UIElementType::TitleConnectionFailedServerNotFound,
            ConnectionError::ConnectionLost => UIElementType::TitleConnectionFailedServerNotFound,
            ConnectionError::ConnectionClosedByServer => UIElementType::TitleConnectionFailedLostConnection,
        };

        self.show_ui_elem(ui_elem, ui_system);

        timer -= 1;

        if timer == 0u32
        {
            messenger.send_broadcast_message(
                Message {
                    from: OUTER_WORLD_SENDER_ID,
                    remote_sender: false,
                    message: MessageType::CommonActorsMessages(
                        CommonActorsMessage::ClientDisconnectedFromGameServer
                    )
                }
            );

            SessionState::WaitingForUsersRequest
        }
        else
        {
            SessionState::ConnectionFailure(timer, reason)
        }
    }


    fn handle_waiting_for_user_input(
        &mut self,
        shared: &mut EngineSharedSystems,
    ) -> SessionState
    {
        self.hide_current_ui_elem(&mut shared.ui);

        *shared.ui.get_mut_ui_element(&UIElementType::TitlePressPToPlayOnline)
            .get_ui_data_mut()
            .get_is_visible_mut() = self.connection_status_visible;

        self.current_visible_ui_elem = UIElementType::TitlePressPToPlayOnline;

        if shared.input.actions().connect_to_server.is_action_just_pressed()
        {
            SessionState::ConnectingToMatchmakingServer {
                connection: None,
                request_sent: false,
                attempts_left: MATCHMAKING_CONNECTION_ATTEMPTS,
            }
        }
        else
        {
            SessionState::WaitingForUsersRequest
        }
    }


    fn handle_connecting_to_matchmaking_server(
        &mut self,
        connection: Option<ConnectionHandle>,
        mut request_sent: bool,
        attempts_left: u64,
        it_is_2d_3d_example: bool,
        shared: &mut EngineSharedSystems,
    ) -> SessionState
    {
        self.hide_current_ui_elem(&mut shared.ui);
        self.show_ui_elem(UIElementType::TitleConnectingToServer, &mut shared.ui);

        let connection = match connection {
            Some(connection) => connection,
            None => {
                let connection = shared.net.connect(ConnectionConfig::WebSocket {
                    url: shared.global_settings.connection.matchmaking_server_url.clone(),
                    connect_timeout: MATCHMAKING_CONNECT_TIMEOUT,
                });

                return SessionState::ConnectingToMatchmakingServer {
                    connection: Some(connection),
                    request_sent: false,
                    attempts_left,
                };
            }
        };

        if !request_sent && shared.net.state(&connection) == ConnectionState::Connected {
            let version = GameVersion::from(VERSION);

            let game_type = match it_is_2d_3d_example {
                false => GameType::Slice4DShooter,
                true => GameType::Slice3DExample,
            };

            let request = ClientMatchmakingServerProtocol::ClientMessage(
                matchmaking_server_protocol::ClientMessage::RequestToConnectToGameServer(
                    version.into(),
                    game_type,
                )
            ).to_packet();

            // if sending fails, `receive` below reports the disconnect
            request_sent = shared.net.send(&connection, request.into(), Reliability::Reliable).is_ok();
        }

        // The server may answer and close the connection at once (e.g. a wrong version):
        // `receive` returns the answer first and reports the disconnect only after it.
        let result: Option<Result<String, ConnectionError>> = match shared.net.receive(&connection) {
            Ok(mut packets) => {
                packets.next().map(|packet| parse_matchmaking_server_response(&packet))
            }

            Err(NetConnectionError::Connecting) => None,

            Err(NetConnectionError::Disconnected(reason)) => {
                Some(Err(match reason {
                    DisconnectReason::Timeout => ConnectionError::ConnectionTimeout,
                    DisconnectReason::ClosedByRemote => ConnectionError::ConnectionClosedByServer,
                    _ => ConnectionError::ConnectionLost,
                }))
            }
        };

        match result {
            None => {
                SessionState::ConnectingToMatchmakingServer {
                    connection: Some(connection),
                    request_sent,
                    attempts_left,
                }
            }

            Some(result) => {
                match result {
                    Ok(game_server_url) => {
                        log::info!("got the url of game server: {}", game_server_url);
                        self.game_server_url = Some(game_server_url);

                        SessionState::ConnectingToGameServer {
                            connection: None,
                            timeout_counter: GAME_SERVER_CONNECTION_TIMEOUT_TICKS,
                            attempts_left,
                        }
                    }
                    Err(e) => {
                        if attempts_left > 0 {
                            log::warn!("Can't connect to game server: {:?}, trying to reconnect", e);

                            SessionState::ConnectingToMatchmakingServer {
                                connection: None,
                                request_sent: false,
                                attempts_left: attempts_left - 1,
                            }
                        } else {
                            SessionState::ConnectionFailure(300, e)
                        }
                    }
                }
            }
        }
    }


    fn handle_connecting_to_game_server(
        &mut self,
        connection: Option<ConnectionHandle>,
        mut timeout_counter: u64,
        attempts_left: u64,
        session_controller_id: ActorID,
        world: &mut WorldContext,
        shared: &mut EngineSharedSystems,
    ) -> SessionState
    {
        self.hide_current_ui_elem(&mut shared.ui);
        self.show_ui_elem(UIElementType::TitleConnectingToServer, &mut shared.ui);

        if timeout_counter > 0 {
            timeout_counter -= 1;
        } else {
            if attempts_left > 0 {
                return SessionState::ConnectingToMatchmakingServer {
                    connection: None,
                    request_sent: false,
                    attempts_left,
                };
            } else {
                return SessionState::ConnectionFailure(300, ConnectionError::ConnectionClosedByServer);
            }
        }

        let connection = match connection {
            Some(connection) => connection,
            None => {
                let connection_settings: &ConnectionSettings = &shared.global_settings.connection;

                let connection = shared.net.connect(ConnectionConfig::WebRtc {
                    url: self.game_server_url
                        .as_ref()
                        .expect("ERROR: Have not game server url during connecting to game server state")
                        .clone(),
                    ice_servers: connection_settings.ice_servers.clone(),
                    turn_server_username: Some(connection_settings.turn_server_username.clone()),
                    turn_server_credential: Some(connection_settings.turn_server_credential.clone()),
                });

                log::info!("Connecting to the game server");

                return SessionState::ConnectingToGameServer {
                    connection: Some(connection),
                    timeout_counter,
                    attempts_left,
                };
            }
        };

        match shared.net.state(&connection) {
            ConnectionState::Connecting => {
                SessionState::ConnectingToGameServer {
                    connection: Some(connection),
                    timeout_counter,
                    attempts_left,
                }
            }

            ConnectionState::Connected => {
                let new_id = shared.net
                    .local_peer_id(&connection)
                    .expect("ERROR: registrated peer (game server) connection, but the game client still does not have id in the p2p network");

                // The main player's actor id becomes its network peer id.
                // The player changes its own id (ChangeMyId) when it gets this message.
                world.messenger.send_direct_message(
                    world.main_actor_id,
                    Message {
                        from: session_controller_id,
                        remote_sender: false,
                        message: MessageType::SpecificActorMessage(
                            SpecificActorMessage::PlayerMessage(
                                PlayerMessage::ChangeYourId(new_id)
                            )
                        )
                    }
                );

                log::info!("Connected to the game server");

                SessionState::ConnectedToGameServer {
                    connection,
                    players_id: Vec::new(),
                }
            }

            ConnectionState::Disconnected(_) => {
                log::warn!("connection to game server is lost, trying to reconnect");

                SessionState::ConnectionFailure(300, ConnectionError::ConnectionClosedByServer)
            }
        }
    }


    fn handle_connected_to_game_server(
        &mut self,
        connection: ConnectionHandle,
        mut players_id: Vec<u128>,
        it_is_2d_3d_example: bool,
        world: &mut WorldContext,
        shared: &mut EngineSharedSystems,
    ) -> SessionState
    {
        self.hide_current_ui_elem(&mut shared.ui);

        if players_id.len() > 0 {
            self.show_ui_elem(UIElementType::TitleConnectedToServer, &mut shared.ui);
        } else {
            self.show_ui_elem(UIElementType::TitleConnectedToServerAndWaitingForOthers, &mut shared.ui);
        }

        // Packets that arrived before a disconnect are processed first; the error comes after them.
        let lost_connection = match shared.net.receive(&connection) {
            Ok(packets) => {
                for packet in packets {

                    if let Some(message) = ServerMessage::from_packet(packet) {
                        match message
                        {
                            ServerMessage::JoinTheMatch(
                                millis_from_server_start,
                                your_team,
                                red_flag_status,
                                blue_flag_status,
                                bonus_spot_status,
                                red_team_score,
                                blue_team_score,
                            ) => {
                                self.session_time = Some(SessionTime {
                                    server_time_at_join_ms: millis_from_server_start,
                                    joined_at: Instant::now(),
                                });

                                world.messenger.send_command(Command {
                                    sender: OUTER_WORLD_SENDER_ID,
                                    command_type: CommandType::RemoveAllHolesAndEffects
                                });

                                world.messenger.send_broadcast_message(
                                    Message {
                                        from: OUTER_WORLD_SENDER_ID,
                                        remote_sender: true,
                                        message: MessageType::SpecificActorMessage(
                                            SpecificActorMessage::SessionControllerMessage(
                                                SessionControllerMessage::JoinedToSession(
                                                    your_team,
                                                    FlagStatus::from(red_flag_status),
                                                    FlagStatus::from(blue_flag_status),
                                                    BonusSpotStatus::from(bonus_spot_status),
                                                    red_team_score,
                                                    blue_team_score,
                                                )
                                            )
                                        )
                                    }
                                );
                            }

                            ServerMessage::PlayerConnected(player_id) => {
                                world.messenger.send_broadcast_message(
                                    Message {
                                        from: OUTER_WORLD_SENDER_ID,
                                        remote_sender: false,
                                        message: MessageType::SpecificActorMessage(
                                            SpecificActorMessage::PlayerMessage(
                                                PlayerMessage::NewPeerConnected(player_id)
                                            )
                                        )
                                    }
                                );

                                players_id.push(player_id);
                            }

                            ServerMessage::PlayerDisconnected(player_id) => {
                                if let Some(index) = players_id.iter().position(|stored_peer| *stored_peer == player_id) {
                                    players_id.remove(index);

                                    world.messenger.send_command(Command {
                                        sender: OUTER_WORLD_SENDER_ID,
                                        command_type: CommandType::RemoveActor(player_id)
                                    });
                                }
                            }

                            ServerMessage::NetMessageToPlayer(from_player, message) => {
                                process_message(
                                    from_player,
                                    message,
                                    world.messenger,
                                    &mut shared.resources,
                                    it_is_2d_3d_example,
                                );
                            }

                            ServerMessage::NewSessionStarted(
                                _server_time,
                                your_team,
                            ) =>
                            {
                                world.messenger.send_command(Command {
                                    sender: OUTER_WORLD_SENDER_ID,
                                    command_type: CommandType::RemoveAllHolesAndEffects
                                });

                                world.messenger.send_broadcast_message(
                                    Message {
                                        from: OUTER_WORLD_SENDER_ID,
                                        remote_sender: true,
                                        message: MessageType::SpecificActorMessage(
                                            SpecificActorMessage::SessionControllerMessage(
                                                SessionControllerMessage::NewSessionStarted(your_team)
                                            )
                                        )
                                    }
                                );
                            }
                        }
                    }
                }
                None
            }
            Err(error) => Some(error),
        };

        if let Some(error) = lost_connection {
            log::warn!("connection to the game server is lost: {:?}", error);

            world.messenger.send_broadcast_message(
                Message {
                    from: OUTER_WORLD_SENDER_ID,
                    remote_sender: false,
                    message: MessageType::SpecificActorMessage(
                        SpecificActorMessage::SessionControllerMessage(
                            SessionControllerMessage::NewSessionStarted(
                                session_controller::DEFAULT_TEAM
                            )
                        )
                    )
                }
            );

            world.messenger.send_broadcast_message(
                Message {
                    from: OUTER_WORLD_SENDER_ID,
                    remote_sender: false,
                    message: MessageType::CommonActorsMessages(
                        CommonActorsMessage::ClientDisconnectedFromGameServer
                    )
                }
            );

            return SessionState::ConnectionFailure(820, ConnectionError::ConnectionClosedByServer);
        }

        SessionState::ConnectedToGameServer { connection, players_id }
    }
}


fn parse_matchmaking_server_response(packet: &[u8]) -> Result<String, ConnectionError>
{
    match alkahest::deserialize::<ClientMatchmakingServerProtocol, ClientMatchmakingServerProtocol>(packet)
    {
        Ok(ClientMatchmakingServerProtocol::MatchmakingServerMessage(message)) =>
        {
            match message
            {
                MatchmakingServerMessage::GameServerAddress((ip, port)) =>
                {
                    Ok(format!(
                        "ws://{}.{}.{}.{}:{}/",
                        ip[0], ip[1], ip[2], ip[3], port
                    ))
                }
                MatchmakingServerMessage::GameServerAddressThroughProxy((proxy_ip, proxy_port, game_port)) =>
                {
                    Ok(format!(
                        "ws://{}.{}.{}.{}:{}/ws/{}",
                        proxy_ip[0], proxy_ip[1], proxy_ip[2], proxy_ip[3], proxy_port, game_port
                    ))
                }
                MatchmakingServerMessage::NoFreeServers =>
                {
                    Err(ConnectionError::NoFreeServers)
                }
                MatchmakingServerMessage::WrongGameVersionCorrectIs(correct_game_version) =>
                {
                    Err(ConnectionError::WrongVersion(correct_game_version.into()))
                }
            }
        }
        _ =>
        {
            Err(ConnectionError::MatchmakingServerClientProtocolError)
        }
    }
}


/// Converts a message relayed from another player into actor messages and commands (unchanged logic).
fn process_message(
    message_from_peer_id: u128,
    message: NetMessageToPlayer,
    messenger: &mut Messenger,
    resources: &mut ResourceLoader,
    it_is_2d_3d_example: bool,
) {
    match message
    {
        NetMessageToPlayer::RemoteCommand(command) =>
        {
            match command
            {
                RemoteCommand::RemoveActor(actor_id) =>
                {
                    messenger.send_command(Command {
                        sender: 0u128,
                        command_type: CommandType::RemoveActor(actor_id)
                    })
                },

                RemoteCommand::SpawnPlayerDeathExplode(pos, team) =>
                {
                    let position = Vec4::from_array(pos);

                    let player_death_explode = PlayersDeathExplosion::new(position, team);

                    messenger.send_command(Command {
                        sender: 0u128,
                        command_type: CommandType::SpawnActor(
                            Box::new(ActorWrapper::PlayersDeathExplosion(player_death_explode))
                        )
                    });
                },
                
                RemoteCommand::SpawnPlayersDollActor(
                    tr,
                    player_sphere_radius,
                    is_alive,
                    team
                ) =>
                {
                    let transform = Transform::from_serializable_transform(tr);

                    let players_doll = PlayerDoll::new(
                        message_from_peer_id,
                        player_sphere_radius,
                        transform,
                        is_alive,
                        resources,
                        team,
                        it_is_2d_3d_example
                    );

                    let actor = ActorWrapper::PlayersDoll(players_doll);

                    messenger.send_command(Command {
                        sender: 0u128,
                        command_type: CommandType::SpawnActor(Box::new(actor))
                    })
                }

                RemoteCommand::SpawnHole(
                    position,
                    radius,
                    color,
                    target_size_reached,
                    target_radius,
                    explode_current_time,
                    explode_final_time,
                ) =>
                {
                    let transform = Transform::from_position(Vec4::from_array(position));
                    let color = Vec3::from_array(color);

                    let hole = Hole::new(
                        transform,
                        radius,
                        color,
                        target_size_reached,
                        target_radius,
                        explode_current_time,
                        explode_final_time,
                    );

                    let actor = ActorWrapper::Hole(hole);

                    messenger.send_command(Command {
                        sender: 0u128,
                        command_type: CommandType::SpawnActor(Box::new(actor))
                    })
                }
            }

        },

        NetMessageToPlayer::RemoteDirectMessage(actor_id, message) => {
            match message
            {
                RemoteMessage::AntiProjectionModeTurnedOn =>
                {
                    messenger.send_direct_message(
                        actor_id,
                        Message {
                            from: message_from_peer_id,
                            remote_sender: true,
                            message: MessageType::SpecificActorMessage(
                                SpecificActorMessage::PlayerMessage(
                                    PlayerMessage::AntiProjectionModeTurnedOn
                                )
                            )
                        }
                    );
                }

                RemoteMessage::ScannerTurnedOn =>
                {
                    messenger.send_direct_message(
                        actor_id,
                        Message {
                            from: message_from_peer_id,
                            remote_sender: true,
                            message: MessageType::SpecificActorMessage(
                                SpecificActorMessage::PlayersDollMessage(
                                    PlayersDollMessage::ScannerTurnedOn
                                )
                            )
                        }
                    );
                }

                RemoteMessage::YouWasScanned =>
                {
                    messenger.send_direct_message(
                        actor_id,
                        Message {
                            from: message_from_peer_id,
                            remote_sender: true,
                            message: MessageType::SpecificActorMessage(
                                SpecificActorMessage::PlayerMessage(
                                    PlayerMessage::YouWasScanned
                                )
                            )
                        }
                    );
                }
                RemoteMessage::SpawnShotgunShot(
                    start_pos,
                    shot_dir ,
                    rng_seed,
                    damage_dealer_id,
                    damage_dealer_team,
                ) =>
                {
                    messenger.send_direct_message(
                        actor_id,
                        Message {
                            from: message_from_peer_id,
                            remote_sender: true,
                            message: MessageType::SpecificActorMessage(
                                SpecificActorMessage::PlayersDollMessage(
                                    PlayersDollMessage::SpawnShotgunShot(
                                        start_pos.into(),
                                        shot_dir.into(),
                                        rng_seed,
                                        damage_dealer_id,
                                        damage_dealer_team,
                                    )
                                )
                            )
                        }
                    );
                },
                RemoteMessage::TeamWin(team) =>
                {
                    messenger.send_direct_message(
                        actor_id,
                        Message {
                            from: message_from_peer_id,
                            remote_sender: true,
                            message: MessageType::SpecificActorMessage(
                                SpecificActorMessage::SessionControllerMessage(
                                    SessionControllerMessage::TeamWin(
                                        team
                                    )
                                )
                            )
                        }
                    );
                }

                RemoteMessage::UpdateTeamsScore(red_team_score, blue_team_score) =>
                {
                    messenger.send_direct_message(
                        actor_id,
                        Message {
                            from: message_from_peer_id,
                            remote_sender: true,
                            message: MessageType::SpecificActorMessage(
                                SpecificActorMessage::SessionControllerMessage(
                                    SessionControllerMessage::SetScore(
                                        red_team_score,
                                        blue_team_score
                                    )
                                )
                            )
                        }
                    );
                }

                RemoteMessage::SetFlagStatus(team, status) =>
                {
                    messenger.send_direct_message(
                        actor_id,
                        Message {
                            from: message_from_peer_id,
                            remote_sender: true,
                            message: MessageType::SpecificActorMessage(
                                SpecificActorMessage::FlagMessage(
                                    FlagMessage::SetFlagStatus(
                                        team,
                                        FlagStatus::from(status)
                                    )
                                )
                            )
                        }
                    );
                }

                RemoteMessage::SetMoveWBonusStatus(index, status) =>
                {
                    messenger.send_direct_message(
                        actor_id,
                        Message {
                            from: message_from_peer_id,
                            remote_sender: true,
                            message: MessageType::SpecificActorMessage(
                                SpecificActorMessage::MoveWBonusSpotMessage(
                                    MoveWBonusSpotMessage::SetBonusStatus(
                                        index,
                                        BonusSpotStatus::from(status)
                                    )
                                )
                            )
                        }
                    );
                }

                RemoteMessage::SetNewTeam(team) =>
                {
                    messenger.send_direct_message(
                        actor_id,
                        Message {
                            from: message_from_peer_id,
                            remote_sender: true,
                            message: MessageType::SpecificActorMessage(
                                SpecificActorMessage::PlayerMessage(
                                    PlayerMessage::SetNewTeam(team)
                                )
                            )
                        }
                    );
                }

                RemoteMessage::SetPlayerDollState(
                    tr,
                    input,
                    velocity,
                    time
                ) =>
                {
                    let transform = Transform::from_serializable_transform(tr);
                    let input_state = PlayerDollInputState::deserialize(input);
                    let velocity = Vec4::from_array(velocity);

                    messenger.send_direct_message(
                        actor_id,
                        Message {
                            from: message_from_peer_id,remote_sender: true,
                            message: MessageType::SpecificActorMessage(
                                SpecificActorMessage::PlayersDollMessage(
                                    PlayersDollMessage::SetInterploatedModelTargetState(
                                        transform,
                                        input_state,
                                        velocity,
                                        time,
                                    )
                                )
                            )
                        }
                    );
                }

                RemoteMessage::SpawnMachineGunShot(pos, is_miss) =>
                {
                    messenger.send_direct_message(
                        actor_id,
                        Message {
                            from: message_from_peer_id,
                            remote_sender: true,
                            message: MessageType::SpecificActorMessage(
                                SpecificActorMessage::PlayersDollMessage(
                                    PlayersDollMessage::SpawnMachineGunShot(
                                        Vec4::from_array(pos),
                                        is_miss
                                    )
                                )
                            )
                        }
                    );
                }

                RemoteMessage::PlayerRespawn(
                    tr,
                    input,
                    velocity,
                    team,
                ) =>
                {
                    let transform = Transform::from_serializable_transform(tr);
                    let input_state = PlayerDollInputState::deserialize(input);
                    let velocity = Vec4::from_array(velocity);
                    
                    messenger.send_direct_message(
                        actor_id,
                        Message {
                            from: message_from_peer_id,
                            remote_sender: true,
                            message: MessageType::SpecificActorMessage(
                                SpecificActorMessage::PlayersDollMessage(
                                    PlayersDollMessage::Respawn(
                                        transform,
                                        input_state,
                                        velocity,
                                        team,
                                    )
                                )
                            )
                        }
                    )
                }

                RemoteMessage::HoleGunStartCharging =>
                {
                    messenger.send_direct_message(
                        actor_id,
                        Message {
                            from: message_from_peer_id,
                            remote_sender: true,
                            message: MessageType::SpecificActorMessage(
                                SpecificActorMessage::PlayersDollMessage(
                                    PlayersDollMessage::HoleGunStartCharging
                                )
                            )
                        }
                    )
                }

                RemoteMessage::DieImmediately =>
                {
                    messenger.send_direct_message(
                        actor_id,
                        Message {
                            from: message_from_peer_id,
                            remote_sender: true,
                            message: MessageType::SpecificActorMessage(
                                SpecificActorMessage::PlayerMessage(
                                    PlayerMessage::DieImmediately
                                )
                            )
                        }
                    )
                },

                RemoteMessage::DieSlowly =>
                {
                    messenger.send_direct_message(
                        actor_id,
                        Message {
                            from: message_from_peer_id,
                            remote_sender: true,
                            message: MessageType::SpecificActorMessage(
                                SpecificActorMessage::PlayerMessage(
                                    PlayerMessage::DieSlowly
                                )
                            )
                        }
                    )
                }

                RemoteMessage::SpawHoleGunMissActor(
                    position,
                    _shoooted_from,
                    radius,
                    color,
                    charging_volume_area
                ) => 
                {
                    messenger.send_direct_message(
                        actor_id,
                        Message {
                            from: message_from_peer_id,
                            remote_sender: true,
                            message: MessageType::SpecificActorMessage(
                                SpecificActorMessage::PlayersDollMessage(
                                    PlayersDollMessage::SpawHoleGunMissActor(
                                        Vec4::from_array(position),
                                        radius,
                                        Vec3::from_array(color),
                                        charging_volume_area
                                    )
                                )
                            )
                        }
                    )
                },

                RemoteMessage::SpawnHoleGunShotActor(
                    position,
                    _shoooted_from,
                    radius,
                    color,
                    charging_volume_area
                ) =>
                {
                    messenger.send_direct_message(
                        actor_id,
                        Message {
                            from: message_from_peer_id,
                            remote_sender: true,
                            message: MessageType::SpecificActorMessage(
                                SpecificActorMessage::PlayersDollMessage(
                                    PlayersDollMessage::SpawnHoleGunShotActor(
                                        Vec4::from_array(position),
                                        radius,
                                        Vec3::from_array(color),
                                        charging_volume_area
                                    )
                                )
                            )
                        }
                    )
                },

                RemoteMessage::SpawnObstacleGunShotActor(
                    position,
                    _shoooted_from,
                    radius,
                    color,
                    charging_volume_area
                ) =>
                {
                    messenger.send_direct_message(
                        actor_id,
                        Message {
                            from: message_from_peer_id,
                            remote_sender: true,
                            message: MessageType::SpecificActorMessage(
                                SpecificActorMessage::PlayersDollMessage(
                                    PlayersDollMessage::SpawnObstacleGunShotActor(
                                        Vec4::from_array(position),
                                        radius,
                                        Vec3::from_array(color),
                                        charging_volume_area
                                    )
                                )
                            )
                        }
                    )
                },

                RemoteMessage::SetTransform(tr) =>
                {
                    let transform = Transform::from_serializable_transform(tr);

                    messenger.send_direct_message(
                        actor_id,
                        Message {
                            from: message_from_peer_id,
                            remote_sender: true,
                            message: MessageType::CommonActorsMessages(
                                CommonActorsMessage::SetTransform(transform)
                            )
                        }
                    )
                },

                RemoteMessage::DealDamageAndForce(
                    damage,
                    force,
                    impact_pos,
                    damage_by_team
                ) =>
                {
                    messenger.send_direct_message(
                        actor_id,
                        Message {
                            from: message_from_peer_id,
                            remote_sender: true,
                            message: MessageType::SpecificActorMessage(
                                SpecificActorMessage::PlayerMessage(
                                    PlayerMessage::GetDamageAndForce(
                                        damage,
                                        Vec4::from_array(force),
                                        Vec4::from_array(impact_pos),
                                        damage_by_team,
                                        message_from_peer_id,
                                    )
                                )
                            )
                        }
                    )
                },
                RemoteMessage::Enable(enable_state) => {
                    messenger.send_direct_message(
                        actor_id,
                        Message {
                            from: message_from_peer_id,
                            remote_sender: true,
                            message: MessageType::CommonActorsMessages(
                                CommonActorsMessage::Enable(enable_state)
                            )
                        }
                    )
                },
            }
        },

        NetMessageToPlayer::RemoteBoardCastMessage(message) => {
            match message
            {
                RemoteMessage::AntiProjectionModeTurnedOn =>
                {
                    messenger.send_broadcast_message(
                        Message {
                            from: message_from_peer_id,
                            remote_sender: true,
                            message: MessageType::SpecificActorMessage(
                                SpecificActorMessage::PlayerMessage(
                                    PlayerMessage::AntiProjectionModeTurnedOn
                                )
                            )
                        }
                    );
                }

                RemoteMessage::ScannerTurnedOn =>
                {
                    messenger.send_broadcast_message(
                        Message {
                            from: message_from_peer_id,
                            remote_sender: true,
                            message: MessageType::SpecificActorMessage(
                                SpecificActorMessage::PlayersDollMessage(
                                    PlayersDollMessage::ScannerTurnedOn
                                )
                            )
                        }
                    );
                }

                RemoteMessage::YouWasScanned =>
                {
                    messenger.send_broadcast_message(
                        Message {
                            from: message_from_peer_id,
                            remote_sender: true,
                            message: MessageType::SpecificActorMessage(
                                SpecificActorMessage::PlayerMessage(
                                    PlayerMessage::YouWasScanned
                                )
                            )
                        }
                    );
                },

                RemoteMessage::SpawnShotgunShot(
                    start_pos,
                    shot_dir ,
                    rng_seed,
                    damage_dealer_id,
                    damage_dealer_team,
                ) =>
                {
                    messenger.send_broadcast_message(
                        Message {
                            from: message_from_peer_id,
                            remote_sender: true,
                            message: MessageType::SpecificActorMessage(
                                SpecificActorMessage::PlayersDollMessage(
                                    PlayersDollMessage::SpawnShotgunShot(
                                        start_pos.into(),
                                        shot_dir.into(),
                                        rng_seed,
                                        damage_dealer_id,
                                        damage_dealer_team,
                                    )
                                )
                            )
                        }
                    );
                },
                RemoteMessage::TeamWin(team) =>
                {
                    messenger.send_broadcast_message(
                        Message {
                            from: message_from_peer_id,
                            remote_sender: true,
                            message: MessageType::SpecificActorMessage(
                                SpecificActorMessage::SessionControllerMessage(
                                    SessionControllerMessage::TeamWin(
                                        team
                                    )
                                )
                            )
                        }
                    );
                }

                RemoteMessage::UpdateTeamsScore(red_team_score, blue_team_score) =>
                {
                    messenger.send_broadcast_message(
                        Message {
                            from: message_from_peer_id,
                            remote_sender: true,
                            message: MessageType::SpecificActorMessage(
                                SpecificActorMessage::SessionControllerMessage(
                                    SessionControllerMessage::SetScore(
                                        red_team_score,
                                        blue_team_score
                                    )
                                )
                            )
                        }
                    );
                }

                RemoteMessage::SetFlagStatus(team, status) =>
                {
                    messenger.send_broadcast_message(
                        Message {
                            from: message_from_peer_id,
                            remote_sender: true,
                            message: MessageType::SpecificActorMessage(
                                SpecificActorMessage::FlagMessage(
                                    FlagMessage::SetFlagStatus(
                                        team,
                                        FlagStatus::from(status)
                                    )
                                )
                            )
                        }
                    );
                }

                RemoteMessage::SetMoveWBonusStatus(index, status) =>
                {
                    messenger.send_broadcast_message(
                        Message {
                            from: message_from_peer_id,
                            remote_sender: true,
                            message: MessageType::SpecificActorMessage(
                                SpecificActorMessage::MoveWBonusSpotMessage(
                                    MoveWBonusSpotMessage::SetBonusStatus(
                                        index,
                                        BonusSpotStatus::from(status)
                                    )
                                )
                            )
                        }
                    );
                }

                RemoteMessage::SetNewTeam(team) =>
                {
                    messenger.send_broadcast_message(
                        Message {
                            from: message_from_peer_id,
                            remote_sender: true,
                            message: MessageType::SpecificActorMessage(
                                SpecificActorMessage::PlayerMessage(
                                    PlayerMessage::SetNewTeam(team)
                                )
                            )
                        }
                    );
                }

                RemoteMessage::SetPlayerDollState(
                    transform,
                    input_state,
                    velocity,
                    time,
                ) =>
                {
                    let transform = Transform::from_serializable_transform(transform);
                    let input_state = PlayerDollInputState::deserialize(input_state);
                    let velocity = Vec4::from_array(velocity);

                    messenger.send_broadcast_message(
                        Message {
                            from: message_from_peer_id,
                            remote_sender: true,
                            message: MessageType::SpecificActorMessage(
                                SpecificActorMessage::PlayersDollMessage(
                                    PlayersDollMessage::SetInterploatedModelTargetState(
                                        transform,
                                        input_state,
                                        velocity,
                                        time,
                                    )
                                )
                            )
                        }
                    )
                }

                RemoteMessage::SpawnMachineGunShot(pos, is_miss) =>
                {
                    messenger.send_broadcast_message(
                        Message {
                            from: message_from_peer_id,
                            remote_sender: true,
                            message: MessageType::SpecificActorMessage(
                                SpecificActorMessage::PlayersDollMessage(
                                    PlayersDollMessage::SpawnMachineGunShot(
                                        Vec4::from_array(pos),
                                        is_miss
                                    )
                                )
                            )
                        }
                    );
                }
                RemoteMessage::PlayerRespawn(
                    tr,
                    input,
                    velocity,
                    team
                ) =>
                {
                    let transform = Transform::from_serializable_transform(tr);
                    let input_state = PlayerDollInputState::deserialize(input);
                    let velocity = Vec4::from_array(velocity);
                    
                    messenger.send_broadcast_message(
                        Message {
                            from: message_from_peer_id,
                            remote_sender: true,
                            message: MessageType::SpecificActorMessage(
                                SpecificActorMessage::PlayersDollMessage(
                                    PlayersDollMessage::Respawn(
                                        transform,
                                        input_state,
                                        velocity,
                                        team,
                                    )
                                )
                            )
                        }
                    )
                }
                
                RemoteMessage::DieImmediately =>
                {
                    messenger.send_broadcast_message(
                        Message {
                            from: message_from_peer_id,
                            remote_sender: true,
                            message: MessageType::SpecificActorMessage(
                                SpecificActorMessage::PlayerMessage(
                                    PlayerMessage::DieImmediately
                                )
                            )
                        }
                    )
                },

                RemoteMessage::DieSlowly =>
                {
                    messenger.send_broadcast_message(
                        Message {
                            from: message_from_peer_id,
                            remote_sender: true,
                            message: MessageType::SpecificActorMessage(
                                SpecificActorMessage::PlayerMessage(
                                    PlayerMessage::DieSlowly
                                )
                            )
                        }
                    )
                }

                RemoteMessage::HoleGunStartCharging =>
                {
                    messenger.send_broadcast_message(
                        Message {
                            from: message_from_peer_id,
                            remote_sender: true,
                            message: MessageType::SpecificActorMessage(
                                SpecificActorMessage::PlayersDollMessage(
                                    PlayersDollMessage::HoleGunStartCharging
                                )
                            )
                        }
                    )
                }

                RemoteMessage::SpawHoleGunMissActor(
                    position,
                    _shoooted_from,
                    radius,
                    color,
                    charging_volume_area
                ) =>
                {
                    messenger.send_broadcast_message(
                        Message {
                            from: message_from_peer_id,
                            remote_sender: true,
                            message: MessageType::SpecificActorMessage(
                                SpecificActorMessage::PlayersDollMessage(
                                    PlayersDollMessage::SpawHoleGunMissActor(
                                        Vec4::from_array(position),
                                        radius,
                                        Vec3::from_array(color),
                                        charging_volume_area
                                    )
                                )
                            )
                        }
                    )
                },

                RemoteMessage::SpawnHoleGunShotActor(
                    position,
                    _shoooted_from,
                    radius,
                    color,
                    charging_volume_area
                ) =>
                {
                    messenger.send_broadcast_message(
                        Message {
                            from: message_from_peer_id,
                            remote_sender: true,
                            message: MessageType::SpecificActorMessage(
                                SpecificActorMessage::PlayersDollMessage(
                                    PlayersDollMessage::SpawnHoleGunShotActor(
                                        Vec4::from_array(position),
                                        radius,
                                        Vec3::from_array(color),
                                        charging_volume_area
                                    )
                                )
                            )
                        }
                    )
                },

                RemoteMessage::SpawnObstacleGunShotActor(
                    position,
                    _shoooted_from,
                    radius,
                    color,
                    charging_volume_area
                ) =>
                {
                    messenger.send_broadcast_message(
                        Message {
                            from: message_from_peer_id,
                            remote_sender: true,
                            message: MessageType::SpecificActorMessage(
                                SpecificActorMessage::PlayersDollMessage(
                                    PlayersDollMessage::SpawnObstacleGunShotActor(
                                        Vec4::from_array(position),
                                        radius,
                                        Vec3::from_array(color),
                                        charging_volume_area
                                    )
                                )
                            )
                        }
                    )
                },

                RemoteMessage::SetTransform(tr) =>
                {
                    let transform = Transform::from_serializable_transform(tr);

                    messenger.send_broadcast_message(
                        Message {
                            from: message_from_peer_id,
                            remote_sender: true,
                            message: MessageType::CommonActorsMessages(
                                CommonActorsMessage::SetTransform(transform)
                            )
                        }
                    )
                },

                RemoteMessage::DealDamageAndForce(
                    damage,
                    force,
                    impact_pos,
                    damaged_by_team
                ) =>
                {
                    messenger.send_broadcast_message(
                        Message {
                            from: message_from_peer_id,
                            remote_sender: true,
                            message: MessageType::SpecificActorMessage(
                                SpecificActorMessage::PlayerMessage(
                                    PlayerMessage::GetDamageAndForce(
                                        damage,
                                        Vec4::from_array(force),
                                        Vec4::from_array(impact_pos),
                                        damaged_by_team,
                                        message_from_peer_id,
                                    )
                                )
                            )
                        }
                    )
                },

                RemoteMessage::Enable(enable_state) =>
                {
                    messenger.send_broadcast_message(
                        Message {
                            from: message_from_peer_id,
                            remote_sender: true,
                            message: MessageType::CommonActorsMessages(
                                CommonActorsMessage::Enable(enable_state)
                            )
                        }
                    )
                },
            }
        }
    }
}

