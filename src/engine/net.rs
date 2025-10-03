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

use std::time::Duration;

use fyrox_core::futures::{SinkExt, StreamExt};
use glam::{Vec3, Vec4};
use client_server_protocol::{ClientMessage, NetMessageToServer, ServerMessage, Team};

#[cfg(not(target_arch = "wasm32"))]
use tokio::runtime::Runtime;
use tokio::task::JoinHandle;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;

use tokio_tungstenite::{
    connect_async,
    tungstenite::error::Error
};

use matchbox_socket::{
    PeerId,
    PeerState,
    RtcIceServerConfig,
    WebRtcSocket
};

use matchmaking_server_protocol::{
    ClientMatchmakingServerProtocol, GameType, GameVersion, MatchmakingServerMessage
};

use client_server_protocol::{
    RemoteCommand,
    RemoteMessage,
    NetCommand,
    NetMessageToPlayer,
};

use crate::{
    actor::{
        flag::{
            FlagMessage, FlagStatus
        }, hole::Hole, main_player::{
            player_settings::PlayerSettings, PlayerMessage
        }, move_w_bonus::{
            BonusSpotStatus, MoveWBonusSpotMessage
        }, players_death_explosion::PlayersDeathExplosion, players_doll::{
            PlayerDoll, PlayerDollInputState, PlayersDollMessage
        }, session_controller::{self, SessionControllerMessage}, ActorWrapper, CommonActorsMessage, Message, MessageType, SpecificActorMessage
    },
    transform::Transform
};

use super::{
    audio::AudioSystem,
    engine_handle::{
        Command,
        CommandType,
        EngineHandle
    },
    input::ActionsFrameState,
    ui::{
        UIElementType,
        UISystem
    }
};


type Packet = Box<[u8]>;


#[derive(Debug)]
enum ConnectionError {
    WrongVersion(GameVersion),
    NoFreeServers,
    MatchmakingServerClientProtocolError,
    ConnectionLost(Error),
    ConnectionClosedByServer,
    ConnectionTimeout,
}

enum ConnectionState {
    WaitingForUsersRequest,
    ConnectionFailure(u32, ConnectionError),
    ConnectingToMatchmakingServer(Option<JoinHandle<Result<String, ConnectionError>>>, u64),
    ConnectingToGameServer(u64, u64, Option<WebRtcSocket>),
    ConnectedToGameServer(WebRtcSocket, PeerId, Vec<u128>),
}

struct ConnectionData {
    matchmaking_server_url: String,
    game_server_url: Option<String>,
    bash_and_turn_servers: Vec<String>,
    turn_server_username: Option<String>,
    turn_server_credential: Option<String>,

}

const VERSION: &str = env!("CARGO_PKG_VERSION");
    
pub struct NetSystem {
    connection_data: ConnectionData,
    connection_state: Option<ConnectionState>,

    player_settings: PlayerSettings,
    it_is_2d_3d_example: bool,
    current_visible_ui_elem: UIElementType,
    connection_status_visible: bool,
}

impl NetSystem {
    pub async fn new(
        settings: &PlayerSettings,
        it_is_2d_3d_example: bool,
        #[cfg(not(target_arch = "wasm32"))]
        async_runtime: &mut Runtime
    ) -> Self {

        let connection_data = ConnectionData {
            matchmaking_server_url: settings.matchmaking_server_url.clone(),
            bash_and_turn_servers: settings.bash_and_turn_servers.clone(),
            game_server_url: None,
            turn_server_username: Some(settings.turn_server_username.clone()),
            turn_server_credential: Some(settings.turn_server_credential.clone()),
        };

        

        NetSystem {
            connection_state: Some(ConnectionState::WaitingForUsersRequest),
            connection_data,

            player_settings: settings.clone(),
            it_is_2d_3d_example,
            current_visible_ui_elem: UIElementType::TitlePressPToPlayOnline,
            connection_status_visible: false,
        }
    }


    pub fn set_is_visible_for_connection_status(
        &mut self,
        connection_status_visible: bool
    )
    {
        self.connection_status_visible = connection_status_visible;
    }


    pub fn tick(
        &mut self,
        input: ActionsFrameState,
        engine_handle: &mut EngineHandle,
        #[cfg(not(target_arch = "wasm32"))]
        async_runtime: &mut Runtime,
        audio_system: &mut AudioSystem,
        ui_system: &mut UISystem,
    ) {

        match self.connection_state.take().expect("ERROR: connection state in Net system is None")
        {
            ConnectionState::WaitingForUsersRequest =>
            {
                self.connection_state = Some(
                    self.handle_waiting_for_user_input(
                        input,
                        async_runtime,
                        ui_system
                    )
                )
            }

            ConnectionState::ConnectionFailure(timer, reason) =>
            {
                self.connection_state = Some(
                    self.handle_connection_failure(
                        timer,
                        reason,
                        engine_handle,
                        ui_system
                    )
                )
            }

            ConnectionState::ConnectingToMatchmakingServer(
                game_server_url_promise,
                connection_attempts_counter
            ) =>
            {
                self.connection_state = Some(
                    self.handle_connecting_to_matchmaking_server_state(
                        game_server_url_promise,
                        connection_attempts_counter,
                        async_runtime,
                        ui_system
                    )
                );
            }

            ConnectionState::ConnectingToGameServer(
                connection_timeout_counter,
                connection_attempts_counter,
                webrtc_socket
            ) =>
            {
                self.connection_state = Some(
                    self.handle_connecting_to_game_server_state(
                        connection_timeout_counter,
                        connection_attempts_counter,
                        webrtc_socket,
                        async_runtime,
                        engine_handle,
                        ui_system
                    )
                );
            }

            ConnectionState::ConnectedToGameServer(webrtc_socket, server_id, players_id) =>
            {
                self.connection_state = Some(
                    self.handle_connected_to_game_server_state(
                        webrtc_socket,
                        server_id,
                        players_id,
                        engine_handle,
                        audio_system,
                        ui_system
                    )
                );
            }
        }
    }


    fn handle_connection_failure(
        &mut self,
        mut timer: u32,
        reason: ConnectionError,
        engine_handle: &mut EngineHandle,
        ui_system: &mut UISystem,
    ) -> ConnectionState
    {
        *ui_system.get_mut_ui_element(&self.current_visible_ui_elem)
            .get_ui_data_mut()
            .get_is_visible_mut() = false;

        match &reason {
            ConnectionError::WrongVersion(_) =>
            {
                *ui_system.get_mut_ui_element(&UIElementType::TitleConnectionFailedOldVersion)
                    .get_ui_data_mut()
                    .get_is_visible_mut() = true;
                
                self.current_visible_ui_elem = UIElementType::TitleConnectionFailedOldVersion;
            },

            ConnectionError::NoFreeServers =>
            {
                *ui_system.get_mut_ui_element(&UIElementType::TitleConnectionFailedServerIsFull)
                    .get_ui_data_mut()
                    .get_is_visible_mut() = true;
                
                self.current_visible_ui_elem = UIElementType::TitleConnectionFailedServerIsFull;
            },

            ConnectionError::MatchmakingServerClientProtocolError =>
            {
                *ui_system.get_mut_ui_element(&UIElementType::TitleConnectionFailedServerError)
                    .get_ui_data_mut()
                    .get_is_visible_mut() = true;
                
                self.current_visible_ui_elem = UIElementType::TitleConnectionFailedServerError;
            },

            ConnectionError::ConnectionTimeout =>
            {
                *ui_system.get_mut_ui_element(&UIElementType::TitleConnectionFailedServerNotFound)
                    .get_ui_data_mut()
                    .get_is_visible_mut() = true;
                
                self.current_visible_ui_elem = UIElementType::TitleConnectionFailedServerNotFound;
            }

            ConnectionError::ConnectionLost(e) =>
            {
                match e
                {
                    Error::ConnectionClosed => 
                    {
                        *ui_system.get_mut_ui_element(&UIElementType::TitleConnectionFailedLostConnection)
                            .get_ui_data_mut()
                            .get_is_visible_mut() = true;
                        
                        self.current_visible_ui_elem = UIElementType::TitleConnectionFailedLostConnection;
                    },
                    _ =>
                    {
                        *ui_system.get_mut_ui_element(&UIElementType::TitleConnectionFailedServerNotFound)
                            .get_ui_data_mut()
                            .get_is_visible_mut() = true;
                        
                        self.current_visible_ui_elem = UIElementType::TitleConnectionFailedServerNotFound;
                    }
                }

            },

            ConnectionError::ConnectionClosedByServer =>
            {
                *ui_system.get_mut_ui_element(&UIElementType::TitleConnectionFailedLostConnection)
                    .get_ui_data_mut()
                    .get_is_visible_mut() = true;
                
                self.current_visible_ui_elem = UIElementType::TitleConnectionFailedLostConnection;
            },
        }

        timer -= 1;

        if timer == 0u32
        {
            engine_handle.send_boardcast_message(
                Message {
                    from: 0u128,
                    remote_sender: false,
                    message: MessageType::CommonActorsMessages(
                        CommonActorsMessage::ClientDisconnectedFromGameServer
                    )
                }
            );

            ConnectionState::WaitingForUsersRequest
        }
        else
        {
            ConnectionState::ConnectionFailure(timer, reason)
        }
    }

        
    fn handle_waiting_for_user_input(
        &mut self,
        input: ActionsFrameState,
        async_runtime: &Runtime,
        ui_system: &mut UISystem,
    ) -> ConnectionState
    {
        *ui_system.get_mut_ui_element(&self.current_visible_ui_elem)
            .get_ui_data_mut()
            .get_is_visible_mut() = false;

        *ui_system.get_mut_ui_element(&UIElementType::TitlePressPToPlayOnline)
            .get_ui_data_mut()
            .get_is_visible_mut() = true && self.connection_status_visible;
        
        self.current_visible_ui_elem = UIElementType::TitlePressPToPlayOnline;

        if input.connect_to_server.is_action_just_pressed()
        {
            ConnectionState::ConnectingToMatchmakingServer(None, 2)
        }
        else
        {
            ConnectionState::WaitingForUsersRequest
        }
    }


    fn handle_connecting_to_matchmaking_server_state(
        &mut self,
        game_server_url_promise:  Option<JoinHandle<Result<String, ConnectionError>>>,
        connection_attempts_counter: u64,
        async_runtime: &mut Runtime,
        ui_system: &mut UISystem,
    ) -> ConnectionState
    {
        *ui_system.get_mut_ui_element(&self.current_visible_ui_elem)
            .get_ui_data_mut()
            .get_is_visible_mut() = false;

        *ui_system.get_mut_ui_element(&UIElementType::TitleConnectingToServer)
            .get_ui_data_mut()
            .get_is_visible_mut() = true;
        
        self.current_visible_ui_elem = UIElementType::TitleConnectingToServer;

        match game_server_url_promise {
            Some(promise) =>
            {
                if promise.is_finished() {
                    let connection_to_matchmaking_result =
                        async_runtime.block_on(promise);

                    match connection_to_matchmaking_result {
                        Ok(connection_result) =>
                        {
                            match connection_result {
                                Ok(game_server_url) =>
                                {
                                    println!("got the url of game server: {}", game_server_url);
                                    self.connection_data.game_server_url = Some(game_server_url);
                                    
                                    return ConnectionState::ConnectingToGameServer(
                                        240,
                                        connection_attempts_counter,
                                        None
                                    );
                                }
                                Err(e) =>
                                {
                                    if connection_attempts_counter > 0
                                    {
                                        println!("WARNING: Can't connect to game server: {:?}, trying to reconnect", e);

                                        return ConnectionState::ConnectingToMatchmakingServer(
                                            None,
                                            connection_attempts_counter - 1
                                        );
                                    }
                                    else
                                    {
                                        return ConnectionState::ConnectionFailure(300, e);
                                    }
                                }
                            }
                        }
                        Err(e) =>
                        {
                            panic!("ERROR: connection to matchmaking server async task error: {}", e)
                        }
                    }

                }
                else
                {
                    return ConnectionState::ConnectingToMatchmakingServer(
                        Some(promise),
                        connection_attempts_counter
                    );
                }
            }
            None =>
            {
                let game_server_url_promise =
                    Some(async_runtime.spawn(

                        get_game_server_url(
                            self.connection_data.matchmaking_server_url.clone(),
                            self.it_is_2d_3d_example
                        )
                    ));
                
                return ConnectionState::ConnectingToMatchmakingServer(
                    game_server_url_promise,
                    connection_attempts_counter
                );
            }
        }
    }


    fn handle_connecting_to_game_server_state(
        &mut self,
        mut connection_timeout_counter: u64,
        connection_attempts_counter: u64,
        webrtc_socket: Option<WebRtcSocket>,
        async_runtime: &mut Runtime,
        engine_handle: &mut EngineHandle,
        ui_system: &mut UISystem,
    ) -> ConnectionState
    {
        *ui_system.get_mut_ui_element(&self.current_visible_ui_elem)
            .get_ui_data_mut()
            .get_is_visible_mut() = false;

        *ui_system.get_mut_ui_element(&UIElementType::TitleConnectingToServer)
            .get_ui_data_mut()
            .get_is_visible_mut() = true;
        
        self.current_visible_ui_elem = UIElementType::TitleConnectingToServer;

        if connection_timeout_counter > 0
        {
            connection_timeout_counter -= 1;
        }
        else
        {
            if connection_attempts_counter > 0
            {
                return ConnectionState::ConnectingToMatchmakingServer(
                    None,
                    connection_attempts_counter
                );
            }
            else
            {
                return  ConnectionState::ConnectionFailure(300, ConnectionError::ConnectionLost(Error::ConnectionClosed));
            }
        }

        match webrtc_socket {
            Some(mut webrtc_socket) =>
            {
                if webrtc_socket.any_channel_closed() {

                    return ConnectionState::ConnectionFailure(300, ConnectionError::ConnectionClosedByServer);
                }
        
                if let Ok(vec) = webrtc_socket.try_update_peers() {
                    for (peer_id, peer_state) in vec {
                        match peer_state {
                            PeerState::Connected => {
        
                                engine_handle.send_command(Command {
                                    sender: 0_u128,
                                    command_type: CommandType::NetCommand(
                                        NetCommand::NetSystemIsConnectedAndGetNewPeerID(
                                            webrtc_socket
                                                .id()
                                                .expect("ERROR: registrated peer (game server) connection, but the game client still does not have id in the p2p network")
                                                .0
                                                .as_u128()
                                        )
                                    ),
                                });

                                let server_id = peer_id;
                                let players_id = Vec::new();

                                println!("INFO: Connected to the game server");
                                return ConnectionState::ConnectedToGameServer(webrtc_socket, server_id, players_id);
                            }
                            PeerState::Disconnected => {

                                println!("WARNING: connection to game server is lost, trying to reconnect");
                                return ConnectionState::ConnectionFailure(300, ConnectionError::ConnectionClosedByServer);
                            }
                        }   
                    }
                }

                return ConnectionState::ConnectingToGameServer(
                    connection_timeout_counter,
                    connection_attempts_counter,
                    Some(webrtc_socket)
                );
            }
            None =>
            {
                let (webrtc_socket, socket_future) =
                    matchbox_socket::WebRtcSocketBuilder::new(
                        self
                            .connection_data.game_server_url
                            .as_ref()
                            .expect("ERROR: Have not game server url during connecting to game server state")
                            .clone()
                    )
                    .reconnect_attempts(Some(3))
                    .signaling_keep_alive_interval(Some(Duration::from_secs(1)))
                    .ice_server(RtcIceServerConfig {
                        urls: self.connection_data.bash_and_turn_servers.clone(),
                        username: self.connection_data.turn_server_username.clone(),
                        credential: self.connection_data.turn_server_credential.clone(),
                    })
                    // .ice_server(RtcIceServerConfig::default())
                    .add_reliable_channel()
                    .add_unreliable_channel()
                    .build();

                #[cfg(target_arch = "wasm32")]
                {
                    let promise = wasm_bindgen_futures::future_to_promise(async {
                        let _ = socket_future.await;

                        Result::Ok(JsValue::null())
                    });

                    let _ = wasm_bindgen_futures::JsFuture::from(promise);
                }
                #[cfg(not(target_arch = "wasm32"))]
                async_runtime.spawn(socket_future);
                
                println!("INFO: Connecting to the game server");

                return ConnectionState::ConnectingToGameServer(
                    connection_timeout_counter,
                    connection_attempts_counter,
                    Some(webrtc_socket)
                );
            }
        }
    }

    fn handle_connected_to_game_server_state(
        &mut self,
        mut webrtc_socket: WebRtcSocket,
        server_id: PeerId,
        mut players_id: Vec<u128>,
        engine_handle: &mut EngineHandle,
        audio_system: &mut AudioSystem,
        ui_system: &mut UISystem,
    ) -> ConnectionState
    {
        *ui_system.get_mut_ui_element(&self.current_visible_ui_elem)
            .get_ui_data_mut()
            .get_is_visible_mut() = false;

        if players_id.len() > 0
        {
            *ui_system.get_mut_ui_element(&UIElementType::TitleConnectedToServer)
                .get_ui_data_mut()
                .get_is_visible_mut() = true;
        
            self.current_visible_ui_elem = UIElementType::TitleConnectedToServer;
        }
        else
        {
            *ui_system.get_mut_ui_element(&UIElementType::TitleConnectedToServerAndWaitingForOthers)
                .get_ui_data_mut()
                .get_is_visible_mut() = true;
        
            self.current_visible_ui_elem = UIElementType::TitleConnectedToServerAndWaitingForOthers;
        }
        
        if webrtc_socket.any_channel_closed() {

            engine_handle.send_boardcast_message(
                Message {
                    from: 0u128,
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

            engine_handle.send_boardcast_message(
                Message {
                    from: 0u128,
                    remote_sender: false,
                    message: MessageType::CommonActorsMessages(
                        CommonActorsMessage::ClientDisconnectedFromGameServer
                    )
                }
            );

            return ConnectionState::ConnectionFailure(820, ConnectionError::ConnectionClosedByServer);
        }

        if let Ok(peers) = webrtc_socket.try_update_peers() {
            for (peer_id, peer_state) in peers {
                match peer_state {
                    PeerState::Connected => {
                        panic!("BUG: Catched host connection during connected to game server state. This can't be happening in client-server net arch");
                    }
                    PeerState::Disconnected => {

                        engine_handle.send_boardcast_message(
                            Message {
                                from: 0u128,
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

                        engine_handle.send_boardcast_message(
                            Message {
                                from: 0u128,
                                remote_sender: false,
                                message: MessageType::CommonActorsMessages(
                                    CommonActorsMessage::ClientDisconnectedFromGameServer
                                )
                            }
                        );

                        return ConnectionState::ConnectionFailure(820, ConnectionError::ConnectionClosedByServer);
                    }
                }   
            }
        }

        for (_, packet) in webrtc_socket.channel_mut(0).receive() {

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
                        engine_handle.send_command(Command {
                            sender: 0_u128,
                            command_type: CommandType::NetCommand(
                                NetCommand::SetServerTime(millis_from_server_start)   
                            )
                        });
                        engine_handle.send_command(Command {
                            sender: 0_u128,
                            command_type: CommandType::RemoveAllHolesAndEffects
                        });
                        engine_handle.send_boardcast_message(
                            Message {
                                from: 0u128,
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
                        engine_handle.send_command(Command {
                            sender: 0_u128,
                            command_type: CommandType::NetCommand(
                                NetCommand::PeerConnected(player_id)
                            ),
                        });
                        players_id.push(player_id);
                    }

                    ServerMessage::PlayerDisconnected(player_id) => {
                        let mut index = 0usize;
                        let mut finded = false;
                        for stored_peer in players_id.iter() {
                            if *stored_peer == player_id {
                                finded = true;
                                break;
                            }
                            index += 1;
                        }

                        if finded {
                            players_id.remove(index);

                            engine_handle.send_command(Command {
                                sender: 0_u128,
                                command_type: CommandType::NetCommand(
                                    NetCommand::PeerDisconnected(player_id)
                                ),
                            });
                        }
                    }
                    
                    ServerMessage::NetMessageToPlayer(from_player, message) => {
                        process_message(
                            from_player,
                            message,
                            engine_handle,
                            audio_system,
                            &self.player_settings,
                            self.it_is_2d_3d_example,
                        );
                    }

                    ServerMessage::NewSessionStarted(
                        server_time,
                        your_team,
                    ) =>
                    {
                        engine_handle.send_command(Command {
                            sender: 0_u128,
                            command_type: CommandType::RemoveAllHolesAndEffects
                        });
                        engine_handle.send_boardcast_message(
                            Message {
                                from: 0u128,
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

        for (_, packet) in webrtc_socket.channel_mut(1).receive() {
            
            if let Some(message) = ServerMessage::from_packet(packet) {
                match message {

                    ServerMessage::NewSessionStarted(_,_) =>
                    {
                        eprintln!("ERROR: recieved NewSessionStarted message from unreliable channel")
                    }

                    ServerMessage::JoinTheMatch(_,_,_,_,_,_,_) =>
                    {
                        eprintln!("ERROR: recieved JoinTheMatch message from unreliable channel")
                    }

                    ServerMessage::PlayerConnected(player_id) =>
                    {
                        eprintln!("ERROR: recieved PlayerConnected message from unreliable channel")
                    }

                    ServerMessage::PlayerDisconnected(player_id) =>
                    {
                        eprintln!("ERROR: recieved PlayerDisconnected message from unreliable channel")
                    }
                    
                    ServerMessage::NetMessageToPlayer(from_player, message) => {
                        process_message(
                            from_player,
                            message,
                            engine_handle,
                            audio_system,
                            &self.player_settings,
                            self.it_is_2d_3d_example,
                        );
                    }
                }
            }
        }

        return ConnectionState::ConnectedToGameServer(webrtc_socket, server_id, players_id);
    }


    pub fn send_message_to_game_server(&mut self, message: NetMessageToServer) {
        match &mut self.connection_state
            .as_mut()
            .expect("ERROR: connection state in Net system is None")
        {
            ConnectionState::ConnectedToGameServer(webrtc_socket, server_id , players_id) =>
            {
                if webrtc_socket.any_channel_closed() {return;}
                
                let packet = ClientMessage::MessageToServer(message).to_packet();
        
                webrtc_socket
                    .channel_mut(0)
                    .send(
                        packet,
                        *server_id
                    );
            }
            _ => {}
        }
    }


    pub fn send_boardcast_message_reliable(&mut self, message: NetMessageToPlayer) {

        match &mut self.connection_state
            .as_mut()
            .expect("ERROR: connection state in Net system is None")
        {
            ConnectionState::ConnectedToGameServer(webrtc_socket, server_id , players_id) =>
            {
                if webrtc_socket.any_channel_closed() {return;}
                
                let packet = ClientMessage::BoardcastMessageToPlayers(message).to_packet();
        
                webrtc_socket
                    .channel_mut(0)
                    .send(
                        packet,
                        *server_id
                    );
            }
            _ => {}
        }
    }


    pub fn send_boardcast_message_unreliable(&mut self, message: NetMessageToPlayer) {

        match &mut self.connection_state
            .as_mut()
            .expect("ERROR: connection state in Net system is None")
        {
            ConnectionState::ConnectedToGameServer(webrtc_socket, server_id , players_id) =>
            {
                if webrtc_socket.any_channel_closed() {return;}

                let packet = ClientMessage::BoardcastMessageToPlayers(message).to_packet();

                webrtc_socket
                    .channel_mut(1)
                    .send(
                        packet,
                        *server_id
                    );
            }
            _ => {}
        }
    }


    pub fn send_direct_message_reliable(&mut self, message: NetMessageToPlayer, peer: u128) {

        match &mut self.connection_state
            .as_mut()
            .expect("ERROR: connection state in Net system is None")
        {
            ConnectionState::ConnectedToGameServer(webrtc_socket, server_id , players_id) =>
            {
                if webrtc_socket.any_channel_closed() {return;}
                
                let packet = ClientMessage::DirectMessageToPlayer(peer, message).to_packet();
        
                webrtc_socket
                    .channel_mut(0)
                    .send(
                        packet,
                        *server_id
                    );
            }
            _ => {}
        }
    }


    pub fn send_direct_message_unreliable(&mut self, message: NetMessageToPlayer, peer: u128) {
        
        match &mut self.connection_state
            .as_mut()
            .expect("ERROR: connection state in Net system is None")
        {
            ConnectionState::ConnectedToGameServer(webrtc_socket, server_id , players_id) =>
            {
                if webrtc_socket.any_channel_closed() {return;}

                let packet = ClientMessage::DirectMessageToPlayer(peer, message).to_packet();
        
                webrtc_socket
                    .channel_mut(1)
                    .send(
                        packet,
                        *server_id
                    );
            }
            _ => {}
        }
    }
}

fn process_message(
    message_from_peer_id: u128,
    message: NetMessageToPlayer,
    engine_handle: &mut EngineHandle,
    audio_system: &mut AudioSystem,
    player_settings: &PlayerSettings,
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
                    engine_handle.send_command(Command {
                        sender: 0u128,
                        command_type: CommandType::RemoveActor(actor_id)
                    })
                },

                RemoteCommand::SpawnPlayerDeathExplode(pos, team) =>
                {
                    let position = Vec4::from_array(pos);

                    let player_death_explode = PlayersDeathExplosion::new(position, team);

                    engine_handle.send_command(Command {
                        sender: 0u128,
                        command_type: CommandType::SpawnActor(
                            ActorWrapper::PlayersDeathExplosion(player_death_explode)
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
                        audio_system,
                        player_settings.clone(),
                        team,
                        it_is_2d_3d_example
                    );

                    let actor = ActorWrapper::PlayersDoll(players_doll);

                    engine_handle.send_command(Command {
                        sender: 0u128,
                        command_type: CommandType::SpawnActor(actor)
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

                    engine_handle.send_command(Command {
                        sender: 0u128,
                        command_type: CommandType::SpawnActor(actor)
                    })
                }
            }

        },

        NetMessageToPlayer::RemoteDirectMessage(actor_id, message) => {
            match message
            {
                RemoteMessage::AntiProjectionModeTurnedOn =>
                {
                    engine_handle.send_direct_message(
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
                    engine_handle.send_direct_message(
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
                    engine_handle.send_direct_message(
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
                    engine_handle.send_direct_message(
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
                    engine_handle.send_direct_message(
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
                    engine_handle.send_direct_message(
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
                    engine_handle.send_direct_message(
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
                    engine_handle.send_direct_message(
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
                    engine_handle.send_direct_message(
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

                    engine_handle.send_direct_message(
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
                    engine_handle.send_direct_message(
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
                    
                    engine_handle.send_direct_message(
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
                    engine_handle.send_direct_message(
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
                    engine_handle.send_direct_message(
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
                    engine_handle.send_direct_message(
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
                    shoooted_from,
                    radius,
                    color,
                    charging_volume_area
                ) => 
                {
                    engine_handle.send_direct_message(
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
                    shoooted_from,
                    radius,
                    color,
                    charging_volume_area
                ) =>
                {
                    engine_handle.send_direct_message(
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

                RemoteMessage::SetTransform(tr) =>
                {
                    let transform = Transform::from_serializable_transform(tr);

                    engine_handle.send_direct_message(
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
                    engine_handle.send_direct_message(
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
                    engine_handle.send_direct_message(
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
                    engine_handle.send_boardcast_message(
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
                    engine_handle.send_boardcast_message(
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
                    engine_handle.send_boardcast_message(
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
                    engine_handle.send_boardcast_message(
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
                    engine_handle.send_boardcast_message(
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
                    engine_handle.send_boardcast_message(
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
                    engine_handle.send_boardcast_message(
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
                    engine_handle.send_boardcast_message(
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
                    engine_handle.send_boardcast_message(
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

                    engine_handle.send_boardcast_message(
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
                    engine_handle.send_boardcast_message(
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
                    
                    engine_handle.send_boardcast_message(
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
                    engine_handle.send_boardcast_message(
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
                    engine_handle.send_boardcast_message(
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
                    engine_handle.send_boardcast_message(
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
                    shoooted_from,
                    radius,
                    color,
                    charging_volume_area
                ) =>
                {
                    engine_handle.send_boardcast_message(
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
                    shoooted_from,
                    radius,
                    color,
                    charging_volume_area
                ) =>
                {
                    engine_handle.send_boardcast_message(
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

                RemoteMessage::SetTransform(tr) =>
                {
                    let transform = Transform::from_serializable_transform(tr);

                    engine_handle.send_boardcast_message(
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
                    engine_handle.send_boardcast_message(
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
                    engine_handle.send_boardcast_message(
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

async fn get_game_server_url(
    matchmaking_server_url: String,
    it_is_2d_3d_example: bool,
) -> Result<String, ConnectionError>
{

    let connection_result = tokio::time::timeout(
        Duration::from_secs(5),
        connect_async(matchmaking_server_url)
    ).await;

    match connection_result
    {
        Ok(connection_result) =>
        {
            match connection_result
            {
                Ok((mut ws_stream, _)) =>
                {
                    let version = GameVersion::from(VERSION);
                    
                    let game_type = match it_is_2d_3d_example
                    {
                        false => GameType::Slice4DShooter,
                        true => GameType::Slice3DExample,
                    };

                    let message = ClientMatchmakingServerProtocol::ClientMessage(
                        matchmaking_server_protocol::ClientMessage::RequestToConnectToGameServer(
                            version.into(),
                            game_type,
                        )
                    ).to_packet();
        
                    let sending_result = ws_stream
                        .send(tokio_tungstenite::tungstenite::Message::binary(message.clone()))
                        .await;
                    
                    match sending_result
                    {
                        Ok(_) =>
                        {
                            let recieving_result = ws_stream.next().await;
        
                            if recieving_result.is_none()
                            {
                                return Err(ConnectionError::ConnectionClosedByServer);
                            }
        
                            match recieving_result.unwrap()
                            {
                                Ok(message) =>
                                {
                                    let deserializeing_result =
                                        alkahest::deserialize::<ClientMatchmakingServerProtocol, ClientMatchmakingServerProtocol>(&message.into_data());
                                    
                                    match deserializeing_result
                                    {
                                        Ok(message) =>
                                        {
                                            match message
                                            {
                                                ClientMatchmakingServerProtocol::MatchmakingServerMessage(message) =>
                                                {
                                                    match message
                                                    {
                                                        MatchmakingServerMessage::GameServerAddress((ip, port)) =>
                                                        {
                                                            let url = format!(
                                                                "ws://{}.{}.{}.{}:{}/",
                                                                ip[0], ip[1], ip[2], ip[3], port
                                                            );
                    
                                                            return Ok(url);
                                                        }
                                                        MatchmakingServerMessage::GameServerAddressThroughProxy((proxy_ip, proxy_port, game_port)) =>
                                                        {
                                                            let url = format!(
                                                                "ws://{}.{}.{}.{}:{}/ws/{}",
                                                                proxy_ip[0], proxy_ip[1], proxy_ip[2], proxy_ip[3], proxy_port, game_port
                                                            );

                                                            return Ok(url);
                                                        }
                                                        MatchmakingServerMessage::NoFreeServers =>
                                                        {
                                                            return Err(ConnectionError::NoFreeServers);
                                                        }
                                                        MatchmakingServerMessage::WrongGameVersionCorrectIs(correct_game_version) =>
                                                        {
                                                            return Err(ConnectionError::WrongVersion(correct_game_version.into()));
                                                        }
                                                    }
                                                }
                                                _ =>
                                                {
                                                    return Err(ConnectionError::MatchmakingServerClientProtocolError)
                                                }
                                            }
                                        }
                                        Err(_) =>
                                        {
                                            return Err(ConnectionError::MatchmakingServerClientProtocolError);
                                        }
                                    }
                                }
                                Err(e) =>
                                {
                                    return Err(ConnectionError::ConnectionLost(e));
                                }
                            }
                        }
                        Err(e) =>
                        {
                            return Err(ConnectionError::ConnectionLost(e));
                        }
                    }
                }
                Err(e) =>
                {
                    return Err(ConnectionError::ConnectionLost(e));
                }
            }
        }
        Err(_) =>
        {
            return Err(ConnectionError::ConnectionTimeout);
        }
    }
}