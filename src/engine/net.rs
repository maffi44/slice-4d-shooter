use std::time::Duration;

use fyrox_core::futures::{SinkExt, StreamExt};
use glam::{Vec3, Vec4};
use client_server_protocol::{ClientMessage, NetMessageToServer, ServerMessage};

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
    ClientMatchmakingServerProtocol,
    MatchmakingServerMessage,
    GameVersion
};

use client_server_protocol::{
    RemoteCommand,
    RemoteMessage,
    NetCommand,
    NetMessageToPlayer,
};

use crate::{
    actor::{
        flag::{FlagMessage, FlagStatus}, hole::Hole, move_w_bonus::{BonusSpotStatus, MoveWBonusSpotMessage}, main_player::{player_settings::PlayerSettings, PlayerMessage}, players_death_explosion::PlayersDeathExplosion, players_doll::{
            PlayerDollInputState, PlayersDoll, PlayersDollMessage}, session_controller::SessionControllerMessage, ActorWrapper, CommonActorsMessage, Message, MessageType, SpecificActorMessage
    },
    transform::{self, Transform}
};

use super::{
    audio::AudioSystem,
    engine_handle::{
        Command,
        CommandType,
        EngineHandle
    }
};

use alkahest::{alkahest, Serialize};

type Packet = Box<[u8]>;


#[derive(Debug)]
enum ConnectionError {
    WrongVersion(GameVersion),
    NoFreeServers,
    MatchmakingServerClientProtocolError,
    ConnectionLost(Error),
    ConnectionClosedByServer,
}

enum ConnectionState {
    ConnectingToMatchmakingServer(Option<JoinHandle<Result<String, ConnectionError>>>),
    ConnectingToGameServer(u64, Option<WebRtcSocket>),
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
    w_levels: Vec<f32>,
    players_friction_on_air: f32,
}

impl NetSystem {
    pub async fn new(
        settings: &PlayerSettings,
        w_levels: &Vec<f32>,
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

        let game_server_url_promise = Some(
            async_runtime.spawn(
                get_game_server_url(
                    connection_data.matchmaking_server_url.clone(),
                    0_u64
                )
            )
        );

        NetSystem {
            connection_state: Some(ConnectionState::ConnectingToMatchmakingServer(game_server_url_promise)),
            connection_data,

            player_settings: settings.clone(),
            w_levels: w_levels.clone(),
            players_friction_on_air: settings.friction_on_air
        }
    }


    pub fn tick(
        &mut self,
        engine_handle: &mut EngineHandle,
        #[cfg(not(target_arch = "wasm32"))]
        async_runtime: &mut Runtime,
        audio_system: &mut AudioSystem
    ) {

        match self.connection_state.take().expect("ERROR: connection state in Net system is None")
        {
            ConnectionState::ConnectingToMatchmakingServer(game_server_url_promise) =>
            {
                self.connection_state = Some(
                    self.handle_connecting_to_matchmaking_server_state(
                        game_server_url_promise,
                        async_runtime,
                    )
                );
            }
            ConnectionState::ConnectingToGameServer(delay_counter, webrtc_socket) =>
            {
                self.connection_state = Some(
                    self.handle_connecting_to_game_server_state(
                        delay_counter,
                        webrtc_socket,
                        async_runtime,
                        engine_handle,
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
                    )
                );
            }
        }
    }

    fn handle_connecting_to_matchmaking_server_state(
        &mut self,
        game_server_url_promise:  Option<JoinHandle<Result<String, ConnectionError>>>,
        async_runtime: &mut Runtime,
    ) -> ConnectionState
    {
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
                                    return ConnectionState::ConnectingToGameServer(0, None);
                                }
                                Err(e) =>
                                {
                                    println!("WARNING: Can't connect to game server: {:?}, trying to reconnect", e);
                                    return ConnectionState::ConnectingToMatchmakingServer(None);
                                }
                            }
                        }
                        Err(e) =>
                        {
                            panic!("ERROR: connection to matchmaking server async task error: {}", e)
                        }
                    }

                } else {
                    return ConnectionState::ConnectingToMatchmakingServer(Some(promise));
                }
            }
            None =>
            {
                let game_server_url_promise =
                    Some(async_runtime.spawn(
                        get_game_server_url(
                            self.connection_data.matchmaking_server_url.clone(),
                            1_u64
                        )
                    ));
                
                return ConnectionState::ConnectingToMatchmakingServer(game_server_url_promise);
            }
        }
    }


    fn handle_connecting_to_game_server_state(
        &mut self,
        mut delay_counter: u64,
        webrtc_socket: Option<WebRtcSocket>,
        async_runtime: &mut Runtime,
        engine_handle: &mut EngineHandle,
    ) -> ConnectionState
    {
        if delay_counter > 0
        {
            delay_counter -= 1;

            return  ConnectionState::ConnectingToGameServer(delay_counter, webrtc_socket);
        }

        match webrtc_socket {
            Some(mut webrtc_socket) =>
            {
                if webrtc_socket.any_channel_closed() {

                    println!("WARNING: WebRTC connection is closed, trying to reconnect");
                    return ConnectionState::ConnectingToGameServer(90, None);
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
                                return ConnectionState::ConnectingToGameServer(90, None);
                            }
                        }   
                    }
                }

                return ConnectionState::ConnectingToGameServer(0, Some(webrtc_socket));
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
                    .signaling_keep_alive_interval(Some(Duration::from_secs(10)))
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

                return ConnectionState::ConnectingToGameServer(0, Some(webrtc_socket));
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
    ) -> ConnectionState
    {
        if webrtc_socket.any_channel_closed() {
            println!("WARNING: WebRTC connection is closed, trying to reconnect");
            return ConnectionState::ConnectingToGameServer(90, None);
        }

        if let Ok(vec) = webrtc_socket.try_update_peers() {
            for (peer_id, peer_state) in vec {
                match peer_state {
                    PeerState::Connected => {
                        panic!("BUG: Catched host connsection during connected to game server state. This can't be happening in client-server net arch");
                    }
                    PeerState::Disconnected => {
                        println!("WARNING: connection to game server is lost, trying to reconnect");
                        return ConnectionState::ConnectingToGameServer(90, None);
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
                            &self.w_levels,
                            self.players_friction_on_air,
                        );
                    }

                    ServerMessage::NewSessionStarted(
                        server_time,
                        your_team,
                    ) =>
                    {
                        // engine_handle.send_command(Command {
                        //     sender: 0_u128,
                        //     command_type: CommandType::NetCommand(
                        //         NetCommand::SetServerTime(server_time)   
                        //     )
                        // });
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
                        panic!("ERROR: recieved NewSessionStarted message from ureliable channel")
                    }

                    ServerMessage::JoinTheMatch(_,_,_,_,_,_,_) =>
                    {
                        panic!("ERROR: recieved JoinTheMatch message from ureliable channel")
                    }

                    ServerMessage::PlayerConnected(player_id) =>
                    {
                        panic!("ERROR: recieved PlayerConnected message from ureliable channel")
                    }

                    ServerMessage::PlayerDisconnected(player_id) =>
                    {
                        panic!("ERROR: recieved PlayerDisconnected message from ureliable channel")
                    }
                    
                    ServerMessage::NetMessageToPlayer(from_player, message) => {
                        process_message(
                            from_player,
                            message,
                            engine_handle,
                            audio_system,
                            &self.player_settings,
                            &self.w_levels,
                            self.players_friction_on_air,
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
                        packet.clone(),
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
                        packet.clone(),
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
                        packet.clone(),
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
                        packet.clone(),
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
                        packet.clone(),
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
    w_levels: &Vec<f32>,
    players_friction_on_air: f32,
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

                RemoteCommand::SpawnPlayerDeathExplode(pos) =>
                {
                    let position = Vec4::from_array(pos);

                    let player_death_explode = PlayersDeathExplosion::new(position);

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

                    let players_doll = PlayersDoll::new(
                        message_from_peer_id,
                        player_sphere_radius,
                        transform,
                        is_alive,
                        audio_system,
                        player_settings.clone(),
                        w_levels.clone(),
                        team,
                        players_friction_on_air
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
    wait_time_in_secs: u64,
) -> Result<String, ConnectionError>
{
    if wait_time_in_secs > 0_u64
    {
        tokio::time::sleep(Duration::from_secs(wait_time_in_secs)).await
    }

    let connection_result =
        connect_async(matchmaking_server_url)
        .await;

    match connection_result
    {
        Ok((mut ws_stream, _)) =>
        {
            let version = GameVersion::from(VERSION);

            let message = ClientMatchmakingServerProtocol::ClientMessage(
                matchmaking_server_protocol::ClientMessage::RequestToConnectToGameServer(
                    version.into()
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