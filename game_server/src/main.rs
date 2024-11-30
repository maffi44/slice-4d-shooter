mod client_server_protocol;

use std::{
    collections::{btree_set::Difference, HashMap}, env, net::{
        Ipv4Addr,
        SocketAddr,
        SocketAddrV4
    },
    process::exit,
    str::FromStr, sync::Arc,
    time::{
        Duration,
        Instant
    }
};
use matchmaking_server_protocol::{
    GameServerMatchmakingServerProtocol,
    GameServerMessage
};
use client_server_protocol::{BonusSpotStatus, ClientMessage, FlagStatus, NetMessageToPlayer, ServerMessage, Team};

use fyrox_core::{futures::{sink::drain, stream::Next, SinkExt}, rand::seq::SliceRandom};
use matchbox_signaling::SignalingServer;
use matchbox_socket::{
    MultipleChannels,
    PeerId,
    PeerState::{Connected, Disconnected},
    RtcIceServerConfig,
    WebRtcChannel,
    WebRtcSocket
};
use tokio::{
    runtime::{
        Builder,
        Runtime
    },
    sync::mpsc::{
        channel,
        Receiver,
        Sender
    },
    task::JoinHandle
};

use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;


#[derive(Clone, Debug)]
struct GameServerConfig {
    signaling_port: u16,
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
                "Usage: game_server <signaling_port> <matchmaking_server_ip> <matchmaking_server_port> <ice_servers_urls> <turn_server_username> <turn_server_credential>"
            );
        }

        let signaling_port = args[1]
            .parse()
            .map_err(|_| "Invalid signaling port")?;
        
        let matchmaking_server_ip = Ipv4Addr::from_str(&args[2])
            .expect("Invalid matchmaking server ip address format");
        
        let matchmaking_server_port = args[3]
            .parse()
            .map_err(|_| "Invalid matchmaking server port")?;

        let max_players = args[4]
            .parse()
            .map_err(|_| "Invalid matchmaking server port")?;

        let ice_urls: Vec<String> = if args.len() > 5
        {
            args[5]
                .split("|")
                .into_iter()
                .filter_map(|s|if s != "" {Some(s.to_string())} else {None})
                .collect()
        }
        else
        {
            Vec::with_capacity(0)
        };

        let username = if args.len() > 6
        {
            if args[6] == ""
            {
                None
            }
            else
            {
                Some(args[6].clone())
            }
        }
        else
        {
            None
        };

        let credential = if args.len() > 7
        {
            if args[7] == ""
            {
                None
            }
            else
            {
                Some(args[7].clone())
            }
        }
        else
        {
            None
        };

        let config = GameServerConfig {
            signaling_port,
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


fn main() {
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

    runtime.block_on(async_main(runtime.clone(), config));
}


async fn async_main(
    runtime: Arc<Runtime>,
    config: GameServerConfig
) {
    
    let (sender_to_matchmaking_server, reciever) =
        channel::<GameServerMatchmakingServerProtocol>(10);

    let matchmaking_server_connect_handle =
        runtime.spawn(connect_to_matchmaking_server(
            config.matchmaking_server_ip.to_string(),
            config.matchmaking_server_port,
            reciever
        ));

    let (player_connected_event_sender, player_connected_event_reciever) =
        std::sync::mpsc::channel::<PeerId>();
    
    let (player_disconnected_event_sender, player_disconnected_event_reciever) =
        std::sync::mpsc::channel::<PeerId>();

    runtime.spawn(run_singnaling_server(
        config.signaling_port,
        config.max_players,
        player_connected_event_sender,
        player_disconnected_event_sender,
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
        }
    }

    println!("ready");

    sender_to_matchmaking_server.send(
        GameServerMatchmakingServerProtocol::GameServerMessage(
            GameServerMessage::ServerStarted(config.game_server_index)
        )
    ).await.unwrap();

    game_server_main_loop(
        webrtc_socket,
        sender_to_matchmaking_server,
        config,
        matchmaking_server_connect_handle,
        player_connected_event_reciever,
        player_disconnected_event_reciever,
    ).await;
}

async fn game_server_main_loop(
    mut webrtc_socket: WebRtcSocket<MultipleChannels>,
    mut sender_to_matchmaking_server: Sender<GameServerMatchmakingServerProtocol>,
    config: GameServerConfig,
    handle_to_matchmaking_server_connect: JoinHandle<()>,
    player_connected_event_reciever: std::sync::mpsc::Receiver<PeerId>,
    player_disconnected_event_reciever: std::sync::mpsc::Receiver<PeerId>,
) {
    let mut players_state = GameSessionState::new(&config); 

    let mut relaible_channel = webrtc_socket
        .take_channel(0)
        .unwrap();

    let mut unrelaible_channel = webrtc_socket
        .take_channel(1)
        .unwrap();

    loop {
        let command = start_new_game_session(
            &mut webrtc_socket,
            &mut sender_to_matchmaking_server,
            &config,
            &mut relaible_channel,
            &mut unrelaible_channel,
            &mut players_state
        ).await;

        match command
        {
            Command::StartNewGameSession =>
            {
                continue;
            }
            Command::ShutDownServer(exit_code) =>
            {
                shutdown_game_server(
                    sender_to_matchmaking_server,
                    config,
                    handle_to_matchmaking_server_connect,
                    exit_code
                ).await;
            }
        }
    }

    
}

pub enum Command
{
    StartNewGameSession,
    ShutDownServer(i32),
}

pub struct GameSessionState
{
    players: HashMap<u128, PlayerInfo>,
    red_team: HashMap<u128,()>,
    blue_team: HashMap<u128,()>,
    move_w_bonus: MoveWBonusSpot,
    red_flag: Flag,
    blue_flag: Flag,
    red_team_score: u32,
    blue_team_score: u32,
}

struct PlayerInfo
{
    peer_id: PeerId,
    team: Team,
    captured_flag: bool,
}

pub const MOVE_W_BONUS_RESPAWN_TIME: u128 = 20_000;
pub const FLAG_RESPAWN_TIME: u128 = 10_000;

impl GameSessionState {
    pub fn new(config: &GameServerConfig) -> Self
    {
        let players = HashMap::with_capacity(config.max_players as usize);
        let red_team = HashMap::with_capacity(config.max_players as usize);
        let blue_team = HashMap::with_capacity(config.max_players as usize);
        
        let red_flag = Flag {
            get_previouse_status_time: 0u128,
            status: FlagStatus::OnTheBase,
            team: Team::Red
        };
        let blue_flag = Flag {
            get_previouse_status_time: 0u128,
            status: FlagStatus::OnTheBase,
            team: Team::Blue
        };
        let move_w_bonus = MoveWBonusSpot {
            get_previouse_status_time: 0u128,
            status: BonusSpotStatus::BonusOnTheSpot
        };

        GameSessionState {
            players,
            red_team,
            blue_team,
            red_flag,
            blue_flag,
            move_w_bonus,
            red_team_score: 0u32,
            blue_team_score: 0u32,
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
        game_session_start_time: &Instant,
        relaible_channel: &mut WebRtcChannel,
    ) {
        let current_time = game_session_start_time.elapsed().as_millis();

        // update move w bonus
        match self.move_w_bonus.status
        {
            BonusSpotStatus::BonusCollected =>
            {
                if current_time - self.move_w_bonus.get_previouse_status_time
                    >=
                    MOVE_W_BONUS_RESPAWN_TIME
                {
                    self.set_new_bonus_status_and_send_update_to_players(
                        game_session_start_time,
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
            FlagStatus::Missed(_) =>
            {
                if current_time - self.red_flag.get_previouse_status_time
                    >=
                    FLAG_RESPAWN_TIME
                {
                    self.set_new_flag_status_and_send_update_to_players(
                        game_session_start_time,
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
            FlagStatus::Missed(_) =>
            {
                if current_time - self.blue_flag.get_previouse_status_time
                    >=
                    FLAG_RESPAWN_TIME
                {
                    self.set_new_flag_status_and_send_update_to_players(
                        game_session_start_time,
                        Team::Blue,
                        FlagStatus::OnTheBase,
                        relaible_channel
                    );
                }
            }
            FlagStatus::Captured(_) => {}
            FlagStatus::OnTheBase   => {}
        }
    }

    pub fn set_new_bonus_status_and_send_update_to_players(
        &mut self,
        game_session_start_time: &Instant,
        index: usize,
        new_status: BonusSpotStatus,
        relaible_channel: &mut WebRtcChannel,
    ) {
        self.move_w_bonus
            .get_previouse_status_time =
            game_session_start_time.elapsed().as_millis();
        
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
        game_session_start_time: &Instant,
        flag_team: Team,
        new_status: FlagStatus,
        relaible_channel: &mut WebRtcChannel,
    ) {
        match flag_team
        {
            Team::Red =>
            {
                self.red_flag
                    .get_previouse_status_time =
                    game_session_start_time.elapsed().as_millis();
                
                self.red_flag.status = new_status;
            }
            Team::Blue =>
            {
                self.blue_flag
                    .get_previouse_status_time =
                    game_session_start_time.elapsed().as_millis();
                
                self.blue_flag.status = new_status;
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
}

struct Flag
{
    get_previouse_status_time: u128,
    status: FlagStatus,
    team: Team,
}

struct MoveWBonusSpot
{
    get_previouse_status_time: u128,
    status: BonusSpotStatus,
}

async fn start_new_game_session(
    webrtc_socket: &mut WebRtcSocket<MultipleChannels>,
    sender_to_matchmaking_server: &mut Sender<GameServerMatchmakingServerProtocol>,
    config: &GameServerConfig,
    relaible_channel: &mut WebRtcChannel,
    unrelaible_channel: &mut WebRtcChannel,
    game_session_state: &mut GameSessionState
) -> Command
{
    let mut idle_timer: Option<Instant> = None;
    let game_session_start_time = std::time::Instant::now();

    if webrtc_socket.any_closed() {
        println!("ERROR: game server's WebRTC connection unexpectedly closed, server will shut down immediately");
        return Command::ShutDownServer(1);
    }

    init_game_session(
        game_session_state,
        relaible_channel,
        &game_session_start_time
    );

    loop {

        if webrtc_socket.any_closed() {
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
                        sender_to_matchmaking_server,
                        config,
                        &game_session_start_time,
                        relaible_channel,
                        game_session_state,
                        id,
                    ).await
                }
                Disconnected =>
                {
                    println!("player {} is disconnected to p2p network", id.0.as_u128());
                    handle_player_disconnection(
                        sender_to_matchmaking_server,
                        config,
                        &game_session_start_time,
                        relaible_channel,
                        game_session_state,
                        id
                    ).await
                }
            }
        }

        let recieved_messages = unrelaible_channel.receive();

        for (from_player, packet) in recieved_messages {
            
            proccess_player_message(
                unrelaible_channel,
                game_session_state,
                from_player,
                packet
            );
        }

        let recieved_messages = relaible_channel.receive();
        
        for (from_player, packet) in recieved_messages {
            
            proccess_player_message(
                relaible_channel,
                game_session_state,
                from_player,
                packet
            );
        }

        tokio::time::sleep(Duration::from_millis(2)).await;
    }
}


fn init_game_session(
    game_session_state: &mut GameSessionState,
    relaible_channel:&mut WebRtcChannel,
    game_session_start_time: &Instant,
)
{
    shuffle_teams(game_session_state);

    game_session_state.blue_flag.status = FlagStatus::OnTheBase;
    game_session_state.red_flag.status = FlagStatus::OnTheBase;
    game_session_state.move_w_bonus.status = BonusSpotStatus::BonusOnTheSpot;
    game_session_state.red_team_score = 0u32;
    game_session_state.blue_team_score = 0u32;

    update_states_for_players(
        game_session_state,
        relaible_channel,
        game_session_start_time
    );
}

fn update_states_for_players(
    game_session_state: &GameSessionState,
    relaible_channel: &mut WebRtcChannel,
    game_session_start_time: &Instant,
)
{
    for (_, player_info) in &game_session_state.players
    {
        relaible_channel.send(
            ServerMessage::NewSessionStarted(
                game_session_start_time.elapsed().as_millis(),
                player_info.team
            ).to_packet(),
            player_info.peer_id
        );
    }
}

use rand::{thread_rng, Rng};

fn shuffle_teams(players_state: &mut GameSessionState)
{
    players_state.red_team.clear();    
    players_state.blue_team.clear();

    let mut keys = Vec::with_capacity(players_state.players.len());

    for key in players_state.players.keys()
    {
        keys.push(*key);
    }

    let mut rng = thread_rng();

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
    let mut rng = thread_rng();

    if rng.gen_bool(0.5)
    {
        return Team::Red;
    }
    else
    {
        return Team::Blue;    
    }
}



async fn handle_player_connection(
    sender_to_matchmaking_server: &mut Sender<GameServerMatchmakingServerProtocol>,
    config: &GameServerConfig,
    game_session_start_time: &Instant,
    channel: &mut WebRtcChannel,
    game_session_state: &mut GameSessionState,
    connected_player_id: PeerId,
) {
    let new_player_team = choose_team_for_new_player(game_session_state);

    channel.send(
        ServerMessage::JoinTheMatch(
            game_session_start_time.elapsed().as_millis(),
            new_player_team,
            game_session_state.red_flag.status,
            game_session_state.blue_flag.status,
            game_session_state.move_w_bonus.status,
            game_session_state.red_team_score,
            game_session_state.blue_team_score,
        ).to_packet(),
        connected_player_id
    );
    
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

    sender_to_matchmaking_server.send(
        GameServerMatchmakingServerProtocol::GameServerMessage(
            GameServerMessage::PlayerConnected(config.game_server_index)
        )
    ).await.unwrap();

    game_session_state.players.insert(
        connected_player_id.0.as_u128(),
        PlayerInfo {
            peer_id: connected_player_id,
            team: new_player_team,
            captured_flag: false,
        }
    );
}


fn make_teams_equal(
    game_session_state: &mut GameSessionState
)
{
    let difference =
        game_session_state.red_team.len() as i32 -
        game_session_state.blue_team.len() as i32;
    
    if difference.abs() > 2
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
        }
    }
}

async fn handle_player_disconnection(
    sender_to_matchmaking_server: &mut Sender<GameServerMatchmakingServerProtocol>,
    config: &GameServerConfig,
    game_session_start_time: &Instant,
    relaible_channel: &mut WebRtcChannel,
    game_session_state: &mut GameSessionState,
    disconnected_player_id: PeerId
) {
    let disconnected_player =
        game_session_state.players.remove(&disconnected_player_id.0.as_u128());
    
    if disconnected_player.is_some()
    {
        let disconnected_player = disconnected_player.unwrap();

        match disconnected_player.team {
            Team::Red =>
            {
                game_session_state.red_team.remove(&disconnected_player_id.0.as_u128());
            }
            Team::Blue =>
            {
                game_session_state.blue_team.remove(&disconnected_player_id.0.as_u128());

            }
        }

        if game_session_state.check_if_player_has_flag(
            disconnected_player_id.0.as_u128()
        ) {
            match disconnected_player.team {
                Team::Red =>
                {
                    game_session_state.set_new_flag_status_and_send_update_to_players(
                        game_session_start_time,
                        Team::Blue,
                        FlagStatus::OnTheBase,
                        relaible_channel
                    );
                }
                Team::Blue =>
                {
                    game_session_state.set_new_flag_status_and_send_update_to_players(
                        game_session_start_time,
                        Team::Red,
                        FlagStatus::OnTheBase,
                        relaible_channel
                    );

                }
            }
        }
    }
    else
    {
        println!("ERROR: disconected player is not exist in game_session_state!");
    }

    make_teams_equal(game_session_state);

    for (_ , player_id) in game_session_state.players.iter() {
        relaible_channel.send(
            ServerMessage::PlayerDisconnected(
                disconnected_player_id.0.as_u128()
            ).to_packet(),
            player_id.peer_id
        );
    }

    sender_to_matchmaking_server.send(
        GameServerMatchmakingServerProtocol::GameServerMessage(
            GameServerMessage::PlayerDisconnected(config.game_server_index)
        )
    ).await.unwrap();
}


fn proccess_player_message(
    channel: &mut WebRtcChannel,
    players_state: &GameSessionState,
    from_player: PeerId,
    packet: Box<[u8]>,
) {
    if let Some(message) = ClientMessage::from_packet(packet) {
        match message {
            ClientMessage::DirectMessage(to_player, message) => {
                let peer_id = players_state.get(&to_player);

                if peer_id.is_some() {
                    channel.send(
                        ServerMessage::NetMessage(
                            from_player.0.as_u128(),
                            message
                        ).to_packet(),
                        *peer_id.unwrap()
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
            ClientMessage::BoardcastMessage(message) => {
                for (index, peer_id) in players_state.iter() {
                    if *index != from_player.0.as_u128() {
                        channel.send(
                            ServerMessage::NetMessage(
                                from_player.0.as_u128(),
                                message.clone()
                            ).to_packet(),
                            *peer_id
                        );
                    }
                }
            }
        }
    }
}


async fn shutdown_game_server(
    sender_to_matchmaking_server: Sender<GameServerMatchmakingServerProtocol>,
    config: GameServerConfig,
    handle_to_matchmaking_server_connect: JoinHandle<()>,
    exit_code: i32,
) -> ! 
{
    sender_to_matchmaking_server.send(
        GameServerMatchmakingServerProtocol::GameServerMessage(
            GameServerMessage::GameServerShutedDown(
                config.game_server_index
            )
        )
    ).await.unwrap();

    let timer = Instant::now();

    while !handle_to_matchmaking_server_connect.is_finished() {
        if timer.elapsed().as_secs() > 3 {
            exit(1)
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    exit(exit_code)
}

enum PlayerAmountChangedEvent
{
    PlayerConnected(PeerId),
    PlayerDisconnected(PeerId),
}


async fn run_singnaling_server(
    port: u16,
    max_players: u32,
    player_connected_event_sender: std::sync::mpsc::Sender<PeerId>,
    player_disconnected_event_sender: std::sync::mpsc::Sender<PeerId>,
) {

    // let active_connections = Arc::new(Mutex::new(HashMap::new()));
    // let players_amount = Arc::new(Mutex::new(0u32));

    // let players_amount_1 = players_amount.clone();
    // let players_amount_2 = players_amount.clone();

    let server = 
        SignalingServer::client_server_builder(
            SocketAddr::V4(
                SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port)
            )
        )

        .on_connection_request(move |connection| {
            println!("request connection: {:?}", connection);

            // let ip = connection.origin.ip();
            // let port = connection.origin.port();

            // if ip == IpAddr::V4(Ipv4Addr::LOCALHOST) {
            //     return Ok(true);
            // }

            // if active_connections.lock().unwrap().contains_key(&(ip, port)) {
            //     return Ok(true);
            // } else {
            //     active_connections.lock().unwrap().insert((ip,port), ());
            //     return Ok(false);
            // }
            Ok(true)

            // if *players_amount.lock().unwrap() >= max_players {
            //     Ok(false)
            // } else {
            //     Ok(true)
            // }
        })

        .on_host_connected(
            |_| println!("host connected")
        )

        .on_client_connected(move |id| {
            println!("player connected, id: {}", id.0.as_u128());
            // *players_amount_1.lock().unwrap() += 1;
            // match player_connected_event_sender.send(id) {
            //     Ok(_) => {}
            //     Err(_) =>
            //     {
            //         println!("ERROR: player connection event channel error");
            //         exit(1);
            //     }
            // }
        })

        .on_client_disconnected(move |id| {
            println!("player disconnected, id: {}", id.0.as_u128());
            // *players_amount_2.lock().unwrap() -= 1;
            // match player_disconnected_event_sender.send(id) {
            //     Ok(_) => {}
            //     Err(_) =>
            //     {
            //         println!("ERROR: player connection event channel error");
            //         exit(1);
            //     }
            // }
        })

        // .on_id_assignment(|(_socket, _id)| {})
        // .on_host_connected(|_id| {})
        // .on_host_disconnected(|_id| {})
        .cors()
        .trace()

        .build();
    
    println!("start signaling server");

    server.serve().await.unwrap()
}


async fn connect_to_matchmaking_server(
    matchmaking_server_ip: String,
    matchmaking_server_port: u16,
    mut reciever: Receiver<GameServerMatchmakingServerProtocol>
) {
    
    let server_url = format!("ws://{}:{}/", matchmaking_server_ip, matchmaking_server_port);

    println!("game server is connecting to matchaking server on {} addres", server_url);
    
    let (mut ws_stream, _) =
        connect_async(server_url)
        .await
        .expect("Failed to connect to matchmaking server");

    println!("game server is sucessfully connected to the matchmaking server");

    while let Some(message) = reciever.recv().await {

        let mut shutdown = false;

        match &message {
            GameServerMatchmakingServerProtocol::GameServerMessage(
                message
            ) => {
                match message {
                    GameServerMessage::GameServerShutedDown(index) => {
                        shutdown = true;
                    },
                    _ => {
                        shutdown = false;
                    }
                }
            }
        }
        
        ws_stream.send(Message::Binary(message.to_packet()))
            .await
            .unwrap();

        if shutdown {
            return ;
        }
        
        continue ;
    }
}