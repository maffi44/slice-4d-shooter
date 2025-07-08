mod client_server_protocol;

use blink_alloc::UnsafeGlobalBlinkAlloc;

use std::{
    collections::HashMap,
    env,
    net::{
        Ipv4Addr,
        SocketAddr,
        SocketAddrV4
    },
    process::exit,
    str::FromStr, sync::{Arc, Mutex},
    time::{
        Duration,
        Instant
    }
};
use matchmaking_server_protocol::{
    GameServerMatchmakingServerProtocol,
    GameServerMessage, MatchmakingServerMessageToGameServer
};
use client_server_protocol::{
    BonusSpotStatus, ClientMessage, FlagStatus, NetMessageToPlayer, NetMessageToServer, RemoteMessage, ServerMessage, Team
};

use fyrox_core::{
    futures::SinkExt,
    rand::seq::SliceRandom
};
use matchbox_signaling::SignalingServer;
use matchbox_socket::{
    PeerId,
    PeerState::{Connected, Disconnected},
    RtcIceServerConfig,
    WebRtcChannel,
    WebRtcSocket
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt}, net::TcpListener, runtime::{
        Builder,
        Runtime
    }, sync::mpsc::{
        channel,
        Receiver,
        Sender
    }, task::JoinHandle
};

use glam::FloatExt;

use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;


#[derive(Clone, Debug)]
struct GameServerConfig {
    signaling_port: u16,
    game_severs_min_port_for_signaling_servers: u16,
    game_severs_max_port_for_signaling_servers: u16,
    game_severs_min_port_for_tcp_listener: u16,
    game_severs_max_port_for_tcp_listener: u16,
    matchmaking_server_ip: Ipv4Addr,
    matchmaking_server_port: u16,
    max_players: u32,
    game_server_index: u16,
    ice_urls: Vec<String>,
    username: Option<String>,
    credential: Option<String>
}


impl GameServerConfig {
    fn new(args: Vec<String>) -> Result<Self, &'static str> {
        if args.len() < 5 {
            return Err(
                "Usage: game_server
    <signaling_port>
    <game_severs_min_port_for_signaling_servers>
    <game_severs_max_port_for_signaling_servers>
    <game_severs_min_port_for_tcp_listener>
    <game_severs_max_port_for_tcp_listener>
    <matchmaking_server_ip>
    <matchmaking_server_port>
    <max_players_per_game_session>
    <ice_servers_urls>
    <turn_server_username>
    <turn_server_credential>\n"
            );
        }

        let signaling_port = args[1]
            .parse()
            .map_err(|_| "Invalid signaling port")?;

        let game_severs_min_port_for_signaling_servers = args[2]
            .parse()
            .map_err(|_| "Invalid game_severs_min_port_for_signaling_servers")?;
        
        let game_severs_max_port_for_signaling_servers = args[3]
            .parse()
            .map_err(|_| "Invalid game_severs_max_port_for_signaling_servers")?;
        
        let game_severs_min_port_for_tcp_listener = args[4]
            .parse()
            .map_err(|_| "Invalid game_severs_min_port_for_tcp_listener")?;
        
        let game_severs_max_port_for_tcp_listener = args[5]
            .parse()
            .map_err(|_| "Invalid game_severs_max_port_for_tcp_listener")?;

        
        let matchmaking_server_ip = Ipv4Addr::from_str(&args[6])
            .expect("Invalid matchmaking server ip address format");
        
        let matchmaking_server_port = args[7]
            .parse()
            .map_err(|_| "Invalid matchmaking server port")?;

        let max_players = args[8]
            .parse()
            .map_err(|_| "Invalid matchmaking server port")?;

        let ice_urls: Vec<String> = if args.len() > 9
        {
            args[9]
                .split("|")
                .into_iter()
                .filter_map(|s|if s != "" {Some(s.to_string())} else {None})
                .collect()
        }
        else
        {
            Vec::with_capacity(0)
        };

        let username = if args.len() > 10
        {
            if args[10] == ""
            {
                None
            }
            else
            {
                Some(args[10].clone())
            }
        }
        else
        {
            None
        };

        let credential = if args.len() > 11
        {
            if args[11] == ""
            {
                None
            }
            else
            {
                Some(args[11].clone())
            }
        }
        else
        {
            None
        };

        let config = GameServerConfig {
            signaling_port,
            game_severs_min_port_for_signaling_servers,
            game_severs_max_port_for_signaling_servers,
            game_severs_min_port_for_tcp_listener,
            game_severs_max_port_for_tcp_listener,
            matchmaking_server_ip,
            matchmaking_server_port,
            max_players,
            game_server_index: signaling_port,
            ice_urls,
            username,
            credential
        };

        println!("Game server config is: {:?}", config);

        Ok(config)
    }
}

#[global_allocator]
static GLOBAL_ALLOC: UnsafeGlobalBlinkAlloc = unsafe {
    UnsafeGlobalBlinkAlloc::new()
};
    

fn main() -> Result<(), ()> {
    let args: Vec<String> = env::args().collect();

    let config = match GameServerConfig::new(args) {
        Ok(cfg) => cfg,
        Err(e) => {
            println!("bad args");
            eprintln!("Error: {}", e);
            exit(1);
        }
    };

    println!("succesfully load config");

    let runtime = Arc::new(
        Builder::new_current_thread()
            .enable_all()
            .worker_threads(1)
            .build()
            .unwrap()
    );

    runtime.block_on(async_main(runtime.clone(), config))
}


async fn async_main(
    runtime: Arc<Runtime>,
    config: GameServerConfig
) -> Result<(), ()> {
    
    let players_amount = Arc::new(Mutex::new(0u32));

    let (matchmaking_server_listener, matchmaking_server_listener_port) =
        create_matchmaking_server_listener(config.clone())
            .await
            .unwrap();

    runtime.spawn(listening_to_matchmaking_server(
        players_amount.clone(),
        matchmaking_server_listener
    ));

    let actual_signaling_server_port = Arc::new(Mutex::new(0));

    runtime.spawn(run_signaling_server(
        config.clone(),
        players_amount.clone(),
        actual_signaling_server_port.clone()
    ));

    let (mut webrtc_socket, socket_future) =
        matchbox_socket::WebRtcSocketBuilder::new(
            format!("ws://localhost:{}/", config.signaling_port)
        )
        .ice_server(RtcIceServerConfig {
            urls: config.ice_urls.clone(),
            username: config.username.clone(),
            credential: config.credential.clone(),
        })
        .reconnect_attempts(Some(3))
        .signaling_keep_alive_interval(Some(Duration::from_secs(3)))
        .add_reliable_channel()
        .add_unreliable_channel()
        .build();

    runtime.spawn(socket_future);

    println!("created webRTC socket, waiting for connection to signaling server");

    let instant = std::time::Instant::now();
    while webrtc_socket.id().is_none() {
        tokio::time::sleep(Duration::from_millis(30)).await;

        if instant.elapsed().as_millis() > 3000 {

            println!("fail to connect to signaling server");
            
            return Err(());
        }
    }

    println!(
        "game server is ready|{}|{}",
        *actual_signaling_server_port.lock().unwrap(),
        matchmaking_server_listener_port,
    );

    game_server_main_loop(
        webrtc_socket,
        config,
    ).await;

    Ok(())
}

async fn game_server_main_loop(
    mut webrtc_socket: WebRtcSocket,
    config: GameServerConfig,
) {
    let mut players_state = GameSessionState::new(&config); 

    let mut relaible_channel = webrtc_socket
        .take_channel(0)
        .unwrap();

    let mut unrelaible_channel = webrtc_socket
        .take_channel(1)
        .unwrap();

    let server_start_time = Instant::now();

    loop {
        let command = start_new_game_session(
            &mut webrtc_socket,
            &config,
            &mut relaible_channel,
            &mut unrelaible_channel,
            &mut players_state,
            &server_start_time,
        ).await;

        match command
        {
            Command::StartNewGameSession =>
            {
                continue;
            }
            Command::ShutDownServer(exit_code) =>
            {
                exit(exit_code);
            }
        }
    }

    
}

pub enum Command
{
    StartNewGameSession,
    ShutDownServer(i32),
}

pub struct Hole
{
    color: [f32;3],
    radius: f32,
    position: [f32;4],
    target_size: f32,
    explode_final_time: f32,
    target_size_reached: bool,
    explode_current_time: f32,
}

const EXPLODE_TIME: f32 = 0.25;

impl Hole
{
    pub fn new(
        radius: f32,
        position: [f32;4],
        color: [f32;3],
    ) -> Self
    {
        Hole {
            color,
            radius: 0.01_f32,
            position,
            target_size: radius,
            target_size_reached: false,
            explode_current_time: 0.0,
            explode_final_time: EXPLODE_TIME * (radius*0.3),
        }
    }

    pub fn update_hole(
        &mut self,
        delta: f32,
    ) -> bool
    {
        if self.target_size_reached {

            self.radius -= delta * 0.2;

            if self.radius <= 0.0
            {
                // return true if hole should be removeds
                return true;
            }
        } else {
            self.explode_current_time += delta;

            let explode_coeff = self.explode_current_time / self.explode_final_time;

            self.radius  = f32::lerp(
                0.0,
                self.target_size,
                explode_coeff.clamp(0.0, 1.0)
            );

            if self.explode_current_time >= self.explode_final_time {
                self.target_size_reached = true;
            }
        }

        // return true if hole should be removed
        false
    }
}

struct GameSessionState
{
    players: HashMap<u128, PlayerInfo>,
    red_team: HashMap<u128,()>,
    blue_team: HashMap<u128,()>,
    holes: Vec<Hole>,
    temporal_holes: Vec<Hole>,
    move_w_bonus: MoveWBonusSpot,
    red_flag: Flag,
    blue_flag: Flag,
    red_team_score: u32,
    blue_team_score: u32,
    previous_update_time: u128,
    game_state: GameState,
}

enum GameState
{
    Playing,
    RedWin(
        // time when team win
        u128
    ),
    BlueWin(
        // time when team win
        u128
    ),
}

#[derive(Clone, Copy)]
struct PlayerInfo
{
    peer_id: PeerId,
    team: Team,
    captured_flag: bool,
}

pub const MOVE_W_BONUS_RESPAWN_TIME: u128 = 30_000;
pub const FLAG_RESPAWN_TIME: u128 = 6_400;
pub const MAX_SCORE: u32 = 4;
pub const TIME_IN_SESSION_AFTER_WIN: u128 = 12_000;

impl GameSessionState {
    pub fn new(config: &GameServerConfig) -> Self
    {
        let players = HashMap::with_capacity(config.max_players as usize);
        let red_team = HashMap::with_capacity(config.max_players as usize);
        let blue_team = HashMap::with_capacity(config.max_players as usize);
        
        let red_flag = Flag {
            get_previous_status_time: 0u128,
            status: FlagStatus::OnTheBase,
            team: Team::Red
        };
        let blue_flag = Flag {
            get_previous_status_time: 0u128,
            status: FlagStatus::OnTheBase,
            team: Team::Blue
        };
        let move_w_bonus = MoveWBonusSpot {
            get_previouse_status_time: 0u128,
            status: BonusSpotStatus::BonusOnTheSpot
        };

        let holes = Vec::new();
        let temporal_holes = Vec::new();

        GameSessionState {
            holes,
            temporal_holes,
            players,
            red_team,
            red_flag,
            blue_team,
            blue_flag,
            move_w_bonus,
            red_team_score: 0u32,
            blue_team_score: 0u32,
            previous_update_time: 0u128,
            game_state: GameState::Playing,
        }
    }

    pub fn check_if_player_has_flag(&self, id: u128) -> bool
    {
        match self.players.get(&id)
        {
            Some(player_info) =>
            {
                player_info.captured_flag
            }
            None =>
            {
                println!(
                    "ERROR: checking the flag on a non-existent player"
                );

                false
            }
        }
    }

    pub fn update_items(
        &mut self,
        server_start_time: &Instant,
        relaible_channel: &mut WebRtcChannel,
    ) {
        let current_time = server_start_time.elapsed().as_millis();
        
        let delta = {
            current_time
            -
            self.previous_update_time
        };

        self.previous_update_time = current_time;

        // update move w bonus
        match self.move_w_bonus.status
        {
            BonusSpotStatus::BonusCollected(_) =>
            {
                if current_time - self.move_w_bonus.get_previouse_status_time
                    >=
                    MOVE_W_BONUS_RESPAWN_TIME
                {
                    self.set_new_bonus_status_and_send_update_to_players(
                        server_start_time,
                        0,
                        BonusSpotStatus::BonusOnTheSpot,
                        relaible_channel
                    );
                }
            }
            BonusSpotStatus::BonusOnTheSpot => {}
        }

        // update red flag
        match self.red_flag.status
        {
            FlagStatus::Droped(_) =>
            {
                if current_time - self.red_flag.get_previous_status_time
                    >=
                    FLAG_RESPAWN_TIME
                {
                    self.set_new_flag_status_and_send_update_to_players(
                        server_start_time,
                        Team::Red,
                        FlagStatus::OnTheBase,
                        relaible_channel
                    );
                }
            }
            FlagStatus::Captured(_) => {}
            FlagStatus::OnTheBase   => {}
        }

        // update blue flag
        match self.blue_flag.status
        {
            FlagStatus::Droped(_) =>
            {
                if current_time - self.blue_flag.get_previous_status_time
                    >=
                    FLAG_RESPAWN_TIME
                {
                    self.set_new_flag_status_and_send_update_to_players(
                        server_start_time,
                        Team::Blue,
                        FlagStatus::OnTheBase,
                        relaible_channel
                    );
                }
            }
            FlagStatus::Captured(_) => {}
            FlagStatus::OnTheBase   => {}
        }

        let delta = delta as f32 / 1000.0;

        while let Some(mut hole) = self.holes.pop()
        {
            let should_remove = hole.update_hole(delta);

            if should_remove
            {
                drop(hole);
            }
            else
            {
                self.temporal_holes.push(hole);
            }
        }

        self.holes.swap_with_slice(&mut self.temporal_holes);
    }

    pub fn set_new_bonus_status_and_send_update_to_players(
        &mut self,
        server_start_time: &Instant,
        index: usize,
        new_status: BonusSpotStatus,
        relaible_channel: &mut WebRtcChannel,
    ) {
        self.move_w_bonus
            .get_previouse_status_time =
            server_start_time.elapsed().as_millis();
        
        self.move_w_bonus.status = new_status;

        for (_, player_info) in &self.players
        {
            relaible_channel.send(
                ServerMessage::NetMessageToPlayer(
                    0u128,
                    NetMessageToPlayer::RemoteBoardCastMessage(
                        client_server_protocol::RemoteMessage::SetMoveWBonusStatus(
                            index as u32,
                            new_status,
                        )
                    )
                ).to_packet(),
                player_info.peer_id
            );
        }
    }

    pub fn set_new_flag_status_and_send_update_to_players(
        &mut self,
        server_start_time: &Instant,
        flag_team: Team,
        new_status: FlagStatus,
        relaible_channel: &mut WebRtcChannel,
    ) {
        match flag_team
        {
            Team::Red =>
            {
                self.red_flag
                    .get_previous_status_time =
                    server_start_time.elapsed().as_millis();
                
                self.red_flag.status = new_status;
            }
            Team::Blue =>
            {
                self.blue_flag
                    .get_previous_status_time =
                    server_start_time.elapsed().as_millis();
                
                self.blue_flag.status = new_status;
            }
        }
        
        match new_status {
            FlagStatus::Captured(id) =>
            {
                self.players.get_mut(&id).unwrap().captured_flag = true;
            }
            _ => {
                match flag_team {
                    Team::Red =>
                    {
                        for (id, _) in &self.blue_team
                        {
                            self.players.get_mut(id).unwrap().captured_flag = false;
                        } 
                    }
                    Team::Blue =>
                    {
                        for (id, _) in &self.red_team
                        {
                            self.players.get_mut(id).unwrap().captured_flag = false;
                        } 
                    }
                }
            }
        }

        for (_, player_info) in &self.players
        {
            relaible_channel.send(
                ServerMessage::NetMessageToPlayer(
                    0u128,
                    NetMessageToPlayer::RemoteBoardCastMessage(
                        client_server_protocol::RemoteMessage::SetFlagStatus(
                            flag_team,
                            new_status
                        )
                    )
                ).to_packet(),
                player_info.peer_id
            );
        }
    }

    pub fn add_score_for_team_and_send_upadate_for_players(
        &mut self,
        score_for_team: Team,
        server_start_time: &Instant,
        relaible_channel: &mut WebRtcChannel,
    )
    {
        match self.game_state
        {
            GameState::BlueWin(_) => {}

            GameState::RedWin(_) => {}

            GameState::Playing =>
            {
                match score_for_team {
                    Team::Red =>
                    {
                        self.red_team_score += 1;
                    }
                    Team::Blue =>
                    {
                        self.blue_team_score += 1;
                    }
                }
        
                for (_, player_info) in &self.players
                {
                    relaible_channel.send(
                        ServerMessage::NetMessageToPlayer(
                            0u128,
                            NetMessageToPlayer::RemoteBoardCastMessage(
                                RemoteMessage::UpdateTeamsScore(
                                    self.red_team_score,
                                    self.blue_team_score,
                                )
                            )
                        ).to_packet(),
                        player_info.peer_id
                    );
                }
        
                if self.red_team_score >= MAX_SCORE
                {
                    self.game_state = GameState::RedWin(
                        server_start_time.elapsed().as_millis()
                    );

                    for (_, player_info) in &self.players
                    {
                        relaible_channel.send(
                            ServerMessage::NetMessageToPlayer(
                                0u128,
                                NetMessageToPlayer::RemoteBoardCastMessage(
                                    RemoteMessage::TeamWin(
                                        Team::Red
                                    )
                                )
                            ).to_packet(),
                            player_info.peer_id
                        );
                    }
                }
                else if self.blue_team_score >= MAX_SCORE
                {
                    self.game_state = GameState::BlueWin(
                        server_start_time.elapsed().as_millis()
                    );

                    for (_, player_info) in &self.players
                    {
                        relaible_channel.send(
                            ServerMessage::NetMessageToPlayer(
                                0u128,
                                NetMessageToPlayer::RemoteBoardCastMessage(
                                    RemoteMessage::TeamWin(
                                        Team::Blue
                                    )
                                )
                            ).to_packet(),
                            player_info.peer_id
                        );
                    }
                }
            }
        }
    }

    pub fn spawn_new_hole(
        &mut self,
        position: [f32; 4],
        radius: f32,
        color: [f32; 3],
    )
    {
        self.holes.push(Hole::new(radius, position, color));
    }

    pub fn add_player(
        &mut self,
        id: u128,
        player_info: PlayerInfo,
    )
    {
        self.players.insert(id, player_info);

        match player_info.team
        {
            Team::Red =>
            {
                self.red_team.insert(id, ());
            }
            Team::Blue =>
            {
                self.blue_team.insert(id, ());
            }
        }

    }

    pub fn remove_player(
        &mut self,
        id: u128,
    )
    {
        self.players.remove(&id);
        self.red_team.remove(&id);
        self.blue_team.remove(&id);
    }
}

struct Flag
{
    get_previous_status_time: u128,
    status: FlagStatus,
    team: Team,
}

struct MoveWBonusSpot
{
    get_previouse_status_time: u128,
    status: BonusSpotStatus,
}

async fn start_new_game_session(
    webrtc_socket: &mut WebRtcSocket,
    config: &GameServerConfig,
    relaible_channel: &mut WebRtcChannel,
    unrelaible_channel: &mut WebRtcChannel,
    game_session_state: &mut GameSessionState,
    server_start_time: &Instant,
) -> Command
{
    game_session_state.game_state = GameState::Playing;
    println!("New game session started!");

    let mut idle_timer: Option<Instant> = None;

    if webrtc_socket.any_channel_closed() {
        println!("ERROR: game server's WebRTC connection unexpectedly closed, server will shut down immediately");
        return Command::ShutDownServer(1);
    }

    init_game_session(
        game_session_state,
        relaible_channel,
        &server_start_time
    );

    loop {

        if webrtc_socket.any_channel_closed() {
            println!("ERROR: game server's WebRTC connection unexpectedly closed, server will shut down immediately");
            return Command::ShutDownServer(1);
        }

        // shutdown the game server if no players on the server for more than 3 minutes
        if webrtc_socket.connected_peers().count() == 0 {
            if idle_timer.is_some() {
                if idle_timer.unwrap().elapsed().as_secs() > 180 {

                println!("INFO: no players on the game server, server is shuting down");
                return Command::ShutDownServer(0);
                    
                }
            } else {
                idle_timer = Some(Instant::now());
            }
        } else {
            idle_timer = None;
        }

        let peers_state = webrtc_socket.update_peers();

        for (id, state) in peers_state {
            match state {
                Connected =>
                {
                    println!("player {} is connected to p2p network", id.0.as_u128());
                    handle_player_connection(
                        config,
                        &server_start_time,
                        relaible_channel,
                        game_session_state,
                        id,
                    ).await
                }
                Disconnected =>
                {
                    println!("player {} is disconnected to p2p network", id.0.as_u128());
                    handle_player_disconnection(
                        config,
                        &server_start_time,
                        relaible_channel,
                        game_session_state,
                        id
                    ).await
                }
            }
        }

        let recieved_messages = unrelaible_channel.receive();

        for (from_player, packet) in recieved_messages {
            
            process_player_message(
                &server_start_time,
                unrelaible_channel,
                game_session_state,
                from_player,
                packet
            );
        }

        let recieved_messages = relaible_channel.receive();
        
        for (from_player, packet) in recieved_messages {
            
            process_player_message(
                &server_start_time,
                relaible_channel,
                game_session_state,
                from_player,
                packet
            );
        }

        game_session_state.update_items(&server_start_time, relaible_channel);

        match game_session_state.game_state
        {
            GameState::Playing => {}
            GameState::BlueWin(win_time) =>
            {
                let time_since_win =
                    server_start_time.elapsed().as_millis()
                    -
                    win_time;
                
                if time_since_win > TIME_IN_SESSION_AFTER_WIN
                {
                    return Command::StartNewGameSession;
                }
            }
            GameState::RedWin(win_time) =>
            {
                let time_since_win =
                    server_start_time.elapsed().as_millis()
                    -
                    win_time;
                
                if time_since_win > TIME_IN_SESSION_AFTER_WIN
                {
                    return Command::StartNewGameSession;
                }
            }
        }

        tokio::time::sleep(Duration::from_millis(16)).await;
    }
}


fn init_game_session(
    game_session_state: &mut GameSessionState,
    relaible_channel:&mut WebRtcChannel,
    server_start_time: &Instant,
)
{
    for (_, player) in &mut game_session_state.players
    {
        player.captured_flag = false;
    }

    game_session_state.blue_flag.status = FlagStatus::OnTheBase;
    game_session_state.blue_flag.get_previous_status_time = 0u128;

    game_session_state.red_flag.status = FlagStatus::OnTheBase;
    game_session_state.red_flag.get_previous_status_time = 0u128;

    game_session_state.move_w_bonus.status = BonusSpotStatus::BonusOnTheSpot;
    game_session_state.move_w_bonus.get_previouse_status_time = 0u128;
    
    game_session_state.red_team_score = 0u32;
    game_session_state.blue_team_score = 0u32;
    
    game_session_state.holes.clear();
    
    shuffle_teams(game_session_state);


    update_states_for_players(
        game_session_state,
        relaible_channel,
        server_start_time
    );
}

fn update_states_for_players(
    game_session_state: &GameSessionState,
    relaible_channel: &mut WebRtcChannel,
    server_start_time: &Instant,
)
{
    for (_, player_info) in &game_session_state.players
    {
        relaible_channel.send(
            ServerMessage::NewSessionStarted(
                server_start_time.elapsed().as_millis(),
                player_info.team
            ).to_packet(),
            player_info.peer_id
        );
    }
}

use rand::{rng, Rng};

fn shuffle_teams(players_state: &mut GameSessionState)
{
    players_state.red_team.clear();    
    players_state.blue_team.clear();

    let mut keys = Vec::with_capacity(players_state.players.len());

    for key in players_state.players.keys()
    {
        keys.push(*key);
    }

    let mut rng = fyrox_core::rand::thread_rng();

    keys.shuffle(&mut rng);

    let mut team = Team::Red;
    for key in keys
    {
        let player_info = players_state.players
            .get_mut(&key)
            .unwrap();

        player_info.team = team;

        match team {
            Team::Red =>
            {
                players_state.red_team.insert(key, ());
                team = Team::Blue;
            }
            Team::Blue =>
            {
                players_state.blue_team.insert(key, ());
                team = Team::Red;
            }
        }
    }
}

fn choose_team_for_new_player(
    game_session_state: &GameSessionState
) -> Team
{
    if game_session_state.blue_team.len() >
        game_session_state.red_team.len()
    {
        return Team::Red;
    }
    if game_session_state.blue_team.len() <
        game_session_state.red_team.len()
    {
        return Team::Blue;
    }
    let mut rng = rng();

    if rng.random_bool(0.5)
    {
        return Team::Red;
    }
    else
    {
        return Team::Blue;    
    }
}



async fn handle_player_connection(
    config: &GameServerConfig,
    server_start_time: &Instant,
    channel: &mut WebRtcChannel,
    game_session_state: &mut GameSessionState,
    connected_player_id: PeerId,
) {
    let new_player_team = choose_team_for_new_player(game_session_state);

    channel.send(
        ServerMessage::JoinTheMatch(
            server_start_time.elapsed().as_millis(),
            new_player_team,
            game_session_state.red_flag.status,
            game_session_state.blue_flag.status,
            game_session_state.move_w_bonus.status,
            game_session_state.red_team_score,
            game_session_state.blue_team_score,
        ).to_packet(),
        connected_player_id
    );

    for hole in &game_session_state.holes
    {
        channel.send(
            ServerMessage::NetMessageToPlayer(
                0u128,
                NetMessageToPlayer::RemoteCommand(
                    client_server_protocol::RemoteCommand::SpawnHole(
                        hole.position,
                        hole.radius,
                        hole.color,
                        hole.target_size_reached,
                        hole.target_size,
                        hole.explode_current_time,
                        hole.explode_final_time,
                    )
                )
            ).to_packet(),
            connected_player_id
        );
    }
    
    for (_ , player_id) in game_session_state.players.iter() {
        channel.send(
            ServerMessage::PlayerConnected(
                connected_player_id.0.as_u128()
            ).to_packet(),
            player_id.peer_id
        );

        channel.send(
            ServerMessage::PlayerConnected(
                player_id.peer_id.0.as_u128()
            ).to_packet(),
            connected_player_id
        );
    }

    game_session_state.add_player(
        connected_player_id.0.as_u128(),
        PlayerInfo {
            peer_id: connected_player_id,
            team: new_player_team,
            captured_flag: false,
        }
    );
}


fn make_teams_equal(
    game_session_state: &mut GameSessionState,
    channel: &mut WebRtcChannel,
)
{
    let difference =
        game_session_state.red_team.len() as i32 -
        game_session_state.blue_team.len() as i32;
    
    if difference.abs() > 1
    {
        if difference < 0
        {
            let mut keys = game_session_state
                .blue_team
                .keys();

            let mut key = keys
                .next()
                .unwrap()
                .clone();

            if game_session_state.check_if_player_has_flag(key)
            {
                key = keys
                    .next()
                    .unwrap()
                    .clone();
            }

            game_session_state
                .blue_team
                .remove(&key)
                .unwrap();

            game_session_state
                .red_team
                .insert(key, ());

            game_session_state
                .players
                .get_mut(&key)
                .unwrap()
                .team =
                Team::Red;
            
            channel.send(
                ServerMessage::NetMessageToPlayer(
                    0u128,
                    NetMessageToPlayer::RemoteBoardCastMessage(
                        RemoteMessage::SetNewTeam(Team::Red)
                    )
                ).to_packet(),
                game_session_state
                    .players
                    .get(&key)
                    .unwrap()
                    .peer_id,
            );
        }
        else
        {
            let mut keys = game_session_state
                .red_team
                .keys();

            let mut key = keys
                .next()
                .unwrap()
                .clone();

            if game_session_state.check_if_player_has_flag(key)
            {
                key = keys
                    .next()
                    .unwrap()
                    .clone();
            }

            game_session_state
                .red_team
                .remove(&key)
                .unwrap();

            game_session_state
                .blue_team
                .insert(key, ());

            game_session_state
                .players
                .get_mut(&key)
                .unwrap()
                .team =
                Team::Blue;
            
            channel.send(
                ServerMessage::NetMessageToPlayer(
                    0u128,
                    NetMessageToPlayer::RemoteBoardCastMessage(
                        RemoteMessage::SetNewTeam(Team::Blue)
                    )
                ).to_packet(),
                game_session_state
                    .players
                    .get(&key)
                    .unwrap()
                    .peer_id,
            );
        }
    }
}

async fn handle_player_disconnection(
    config: &GameServerConfig,
    server_start_time: &Instant,
    relaible_channel: &mut WebRtcChannel,
    game_session_state: &mut GameSessionState,
    disconnected_player_id: PeerId
) {
    if game_session_state.check_if_player_has_flag(
        disconnected_player_id.0.as_u128()
    )
    {
        match game_session_state.players
            .get(&disconnected_player_id.0.as_u128())
            .unwrap()
            .team
        {
            Team::Red =>
            {
                game_session_state.set_new_flag_status_and_send_update_to_players(
                    server_start_time,
                    Team::Blue,
                    FlagStatus::OnTheBase,
                    relaible_channel
                );
            }
            Team::Blue =>
            {
                game_session_state.set_new_flag_status_and_send_update_to_players(
                    server_start_time,
                    Team::Red,
                    FlagStatus::OnTheBase,
                    relaible_channel
                );

            }
        }
    }

    let disconnected_player =
        game_session_state.players.remove(&disconnected_player_id.0.as_u128());
    
    if disconnected_player.is_some()
    {
        let disconnected_player = disconnected_player.unwrap();

        game_session_state.red_team.remove(&disconnected_player_id.0.as_u128());
        game_session_state.blue_team.remove(&disconnected_player_id.0.as_u128());

        make_teams_equal(game_session_state, relaible_channel);

        for (_ , player_id) in game_session_state.players.iter() {
            relaible_channel.send(
                ServerMessage::PlayerDisconnected(
                    disconnected_player_id.0.as_u128()
                ).to_packet(),
                player_id.peer_id
            );
        }
    }
    else
    {
        println!("ERROR: disconected player is not exist in game_session_state!");
    }    
}


fn process_player_message(
    server_start_time: &Instant,
    channel: &mut WebRtcChannel,
    game_session_state: &mut GameSessionState,
    from_player: PeerId,
    packet: Box<[u8]>,
) {
    if let Some(message) = ClientMessage::from_packet(packet) {
        match message
        {
            ClientMessage::DirectMessageToPlayer(to_player, message) => {
                let player_info = game_session_state.players.get(&to_player);

                // match &message
                // {
                //     NetMessageToPlayer::RemoteBoardCastMessage(message) =>
                //     {
                //         match message
                //         {
                //             RemoteMessage::SpawnHoleGunShotActor(
                //                 position,
                //                 _shoooted_from,
                //                 radius,
                //                 color,
                //                 _charging_volume_area
                //             ) =>
                //             {
                //                 let hole = Hole::new(*radius, *position, *color);

                //                 game_session_state.holes.push(hole);
                //             }
                //             _ => {}
                //         }
                //     }
                //     _ => {}
                // }

                if player_info.is_some() {
                    channel.send(
                        ServerMessage::NetMessageToPlayer(
                            from_player.0.as_u128(),
                            message
                        ).to_packet(),
                        player_info.unwrap().peer_id
                    );
                } else {
                    channel.send(
                        ServerMessage::PlayerDisconnected(
                            to_player
                        ).to_packet(),
                        from_player
                    );
                }
            }

            ClientMessage::BoardcastMessageToPlayers(message) => {
                for (index, player_info) in game_session_state.players.iter() {
                    if *index != from_player.0.as_u128() {
                        channel.send(
                            ServerMessage::NetMessageToPlayer(
                                from_player.0.as_u128(),
                                message.clone()
                            ).to_packet(),
                            player_info.peer_id
                        );
                    }
                }
            }

            ClientMessage::MessageToServer(message) =>
            {
                match message {
                    NetMessageToServer::TryToGetOpponentsFlag(
                        time_of_attempt,
                    ) =>
                    {
                        let team_trying_to_captured_flag = {
                            if game_session_state.red_team.contains_key(&from_player.0.as_u128())
                            {
                                Team::Red
                            }
                            else if game_session_state.blue_team.contains_key(&from_player.0.as_u128())
                            {
                                Team::Blue
                            }
                            else
                            {
                                // this case means that the player whose message was recieved
                                // by the server is already disconnected
                                return;    
                            }
                        };

                        match team_trying_to_captured_flag
                        {
                            Team::Red =>
                            {
                                match game_session_state.blue_flag.status
                                {
                                    FlagStatus::Captured(_) => {}

                                    _ =>
                                    {
                                        if time_of_attempt > game_session_state.blue_flag.get_previous_status_time
                                        {
                                            game_session_state.set_new_flag_status_and_send_update_to_players(
                                                server_start_time,
                                                Team::Blue,
                                                FlagStatus::Captured(from_player.0.as_u128()),
                                                channel
                                            );
                                        }
                                    }
                                }
                            }

                            Team::Blue =>
                            {
                                match game_session_state.red_flag.status
                                {
                                    FlagStatus::Captured(_) => {}

                                    _ =>
                                    {
                                        if time_of_attempt > game_session_state.red_flag.get_previous_status_time
                                        {
                                            game_session_state.set_new_flag_status_and_send_update_to_players(
                                                server_start_time,
                                                Team::Red,
                                                FlagStatus::Captured(from_player.0.as_u128()),
                                                channel
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }

                    NetMessageToServer::TryToReturnMyFlag(
                        time_of_attempt,
                    ) =>
                    {
                        let team_trying_to_return_flag = {
                            if game_session_state.red_team.contains_key(&from_player.0.as_u128())
                            {
                                Team::Red
                            }
                            else if game_session_state.blue_team.contains_key(&from_player.0.as_u128())
                            {
                                Team::Blue
                            }
                            else
                            {
                                // this case means that the player whose message was recieved
                                // by the server is already disconnected
                                return;    
                            }
                        };

                        match team_trying_to_return_flag
                        {
                            Team::Red =>
                            {
                                if time_of_attempt > game_session_state.red_flag.get_previous_status_time
                                {
                                    match game_session_state.red_flag.status
                                    {
                                        FlagStatus::Droped(_) =>
                                        {
                                            game_session_state.set_new_flag_status_and_send_update_to_players(
                                                server_start_time,
                                                Team::Red,
                                                FlagStatus::OnTheBase,
                                                channel,
                                            );
                                        }
                                        _ => {}
                                    }
                                }
                            }

                            Team::Blue =>
                            {
                                if time_of_attempt > game_session_state.blue_flag.get_previous_status_time
                                {
                                    match game_session_state.blue_flag.status
                                    {
                                        FlagStatus::Droped(_) =>
                                        {
                                            game_session_state.set_new_flag_status_and_send_update_to_players(
                                                server_start_time,
                                                Team::Blue,
                                                FlagStatus::OnTheBase,
                                                channel,
                                            );
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }

                    NetMessageToServer::TryToGetScore(
                        time_of_attempt
                    ) =>
                    {
                        let team_trying_to_get_score = {
                            if game_session_state.red_team.contains_key(&from_player.0.as_u128())
                            {
                                Team::Red
                            }
                            else if game_session_state.blue_team.contains_key(&from_player.0.as_u128())
                            {
                                Team::Blue
                            }
                            else
                            {
                                // this case means that the player whose message was recieved
                                // by the server is already disconnected
                                return;    
                            }
                        };

                        match team_trying_to_get_score
                        {
                            Team::Red =>
                            {
                                if time_of_attempt > game_session_state.blue_flag.get_previous_status_time
                                {
                                    match game_session_state.blue_flag.status
                                    {
                                        FlagStatus::Captured(captured_by) =>
                                        {
                                            if from_player.0.as_u128() == captured_by
                                            {
                                                game_session_state.set_new_flag_status_and_send_update_to_players(
                                                    server_start_time,
                                                    Team::Blue,
                                                    FlagStatus::OnTheBase,
                                                    channel,
                                                );

                                                game_session_state.add_score_for_team_and_send_upadate_for_players(
                                                    Team::Red,
                                                    server_start_time,
                                                    channel,
                                                );
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }

                            Team::Blue =>
                            {
                                if time_of_attempt > game_session_state.red_flag.get_previous_status_time
                                {
                                    match game_session_state.red_flag.status
                                    {
                                        FlagStatus::Captured(captured_by) =>
                                        {
                                            if from_player.0.as_u128() == captured_by
                                            {
                                                game_session_state.set_new_flag_status_and_send_update_to_players(
                                                    server_start_time,
                                                    Team::Red,
                                                    FlagStatus::OnTheBase,
                                                    channel,
                                                );

                                                game_session_state.add_score_for_team_and_send_upadate_for_players(
                                                    Team::Blue,
                                                    server_start_time,
                                                    channel,
                                                );
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }

                    NetMessageToServer::TryToGetMoveWBonus(
                        time_of_attempt,
                        bonus_spot_index,
                    ) =>
                    {
                        match game_session_state.move_w_bonus.status {
                            BonusSpotStatus::BonusOnTheSpot =>
                            {
                                if game_session_state.move_w_bonus.get_previouse_status_time <
                                    time_of_attempt
                                {
                                    game_session_state.set_new_bonus_status_and_send_update_to_players(
                                        server_start_time,
                                        bonus_spot_index as usize,
                                        BonusSpotStatus::BonusCollected(from_player.0.as_u128()),
                                        channel
                                    );
                                }
                            }
                            BonusSpotStatus::BonusCollected(_) => {}
                        }
                    }

                    NetMessageToServer::DropedFlag(
                        team,
                        position,
                        droped_in_space,
                    ) =>
                    {
                        match team
                        {
                            Team::Red =>
                            {
                                match game_session_state.red_flag.status
                                {
                                    FlagStatus::Captured(id) =>
                                    {
                                        if id == from_player.0.as_u128()
                                        {
                                            if droped_in_space
                                            {
                                                game_session_state.set_new_flag_status_and_send_update_to_players(
                                                    server_start_time,
                                                    team,
                                                    FlagStatus::OnTheBase,
                                                    channel
                                                );
                                            }
                                            else
                                            {
                                                game_session_state.set_new_flag_status_and_send_update_to_players(
                                                    server_start_time,
                                                    team,
                                                    FlagStatus::Droped(position),
                                                    channel
                                                );
                                            }
                                                            }
                                    }
                                    _ => {}
                                }
                            }
                            Team::Blue =>
                            {
                                match game_session_state.blue_flag.status
                                {
                                    FlagStatus::Captured(id) =>
                                    {
                                        if id == from_player.0.as_u128()
                                        {
                                            if droped_in_space
                                            {
                                                game_session_state.set_new_flag_status_and_send_update_to_players(
                                                    server_start_time,
                                                    team,
                                                    FlagStatus::OnTheBase,
                                                    channel
                                                );
                                            }
                                            else
                                            {
                                                game_session_state.set_new_flag_status_and_send_update_to_players(
                                                    server_start_time,
                                                    team,
                                                    FlagStatus::Droped(position),
                                                    channel
                                                );
                                            }
                                                            }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

enum PlayerAmountChangedEvent
{
    PlayerConnected(PeerId),
    PlayerDisconnected(PeerId),
}


async fn create_matchmaking_server_listener(config: GameServerConfig) -> Option<(TcpListener, u16)> {
    for port in config.game_severs_min_port_for_tcp_listener..=config.game_severs_max_port_for_tcp_listener
    {
        match TcpListener::bind(("0.0.0.0", port)).await
        {    
            Ok(listener) => return Some((listener, port)),
            Err(_) => continue,
        }
    }

    None
}


async fn listening_to_matchmaking_server(
    players_amount: Arc<Mutex<u32>>,
    tcp_listener: TcpListener,
)
{
    while let Ok((mut stream, _)) = tcp_listener.accept().await
    {
        let mut buf = Vec::new();

        stream.read_buf(&mut buf).await.unwrap();

        let message = alkahest::deserialize::<GameServerMatchmakingServerProtocol, GameServerMatchmakingServerProtocol>(&buf);

        match message
        {
            Ok(message) =>
            {
                match message {
                    GameServerMatchmakingServerProtocol::MatchmakingServerMessageToGameServer(msg) =>
                    {
                        match msg {
                            MatchmakingServerMessageToGameServer::GiveMePlayersAmount =>
                            {
                                let message = GameServerMatchmakingServerProtocol::GameServerMessage(
                                    GameServerMessage::PlayersAmoutIs(*players_amount.lock().unwrap())
                                );

                                stream.write_all(&message.to_packet()).await.unwrap();
                            }
                        }
                    }

                    GameServerMatchmakingServerProtocol::GameServerMessage(_) =>
                    {
                        println!("ERROR: game server recieved GameServerMessage message by matchmaking server listener");

                        continue;
                    }
                }
            }
            Err(_) =>
            {
                println!("ERROR: game server recieved unexpected message by matchmaking server listener");

                continue;
            }
        }
    }

    exit(1);
}

async fn  run_signaling_server(
    config: GameServerConfig,
    players_amount: Arc<Mutex<u32>>,
    actual_signaling_server_port: Arc<Mutex<u16>>,
) {

    let players_amount_1 = players_amount.clone();
    let players_amount_2 = players_amount.clone();
    let players_amount_3 = players_amount.clone();

    let max_players = config.max_players;

    *actual_signaling_server_port.lock().unwrap() = config.signaling_port;

    let server = 
        SignalingServer::client_server_builder(
            SocketAddr::V4(
                SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, config.signaling_port)
            )
        )

        .on_connection_request(move |connection| {
            if *players_amount_1.lock().unwrap() >= max_players {
                Ok(false)
            } else {
                Ok(true)
            }
        })

        .on_host_connected(
            |_| println!("host connected")
        )

        .on_client_connected(move |id| {
            *players_amount_2.lock().unwrap() += 1;
        })

        .on_client_disconnected(move |id| {
            *players_amount_3.lock().unwrap() -= 1;
        })

        .build();
    
    println!("start signaling server");

    if server.serve().await.is_err()
    {
        for port in (config.game_severs_min_port_for_signaling_servers..=config.game_severs_max_port_for_signaling_servers)
        {
            *actual_signaling_server_port.lock().unwrap() = port;

            let players_amount_1 = players_amount.clone();
            let players_amount_2 = players_amount.clone();
            let players_amount_3 = players_amount.clone();

            let server = 
                SignalingServer::client_server_builder(
                    SocketAddr::V4(
                        SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port)
                    )
                )

                .on_connection_request(move |connection| {
                    if *players_amount_1.lock().unwrap() >= max_players {
                        Ok(false)
                    } else {
                        Ok(true)
                    }
                })

                .on_host_connected(
                    |_| println!("host connected")
                )

                .on_client_connected(move |id| {
                    *players_amount_2.lock().unwrap() += 1;
                })

                .on_client_disconnected(move |id| {
                    *players_amount_3.lock().unwrap() -= 1;
                })

                .build();

            if server.serve().await.is_err()
            {
                continue;
            }
            else
            {
                exit(0);
  
            }
        }

        exit(1);
    }
}