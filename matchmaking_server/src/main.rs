pub mod matchmaking_server_protocol;

use matchmaking_server_protocol::{
    GameServerMatchmakingServerProtocol,
    ClientMatchmakingServerProtocol,
    MatchmakingServerMessage,
    GameServerMessage,
    ClientMessage,
    GameVersion
};

use core::panic;
use std::{
    env, io::{Read, Write}, net::Ipv4Addr, os::linux::raw::stat, process::Stdio, str::FromStr, sync::Arc, time::Duration
};
use tokio::{
    fs::File, io::{
        AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader, Lines
    }, net::{TcpListener, TcpSocket, TcpStream}, process::{
        Child,
        ChildStderr,
        ChildStdout,
        Command
    }, runtime::Runtime, sync::{
        Mutex,
        MutexGuard
    }
};

use fyrox_core::futures::{SinkExt, StreamExt};
use serde_json::{
    Value,
    Map,
};
use tokio_tungstenite::accept_async;
use std::collections::HashMap;

#[derive(Clone)]
struct Config
{
    pub matchmaking_server_ip: Ipv4Addr,
    pub matchmaking_server_port_for_clients: u16,
    pub matchmaking_server_port_for_servers: u16,

    pub current_game_version: GameVersion,

    pub game_severs_public_ip: Ipv4Addr,

    pub game_severs_min_port_for_signaling_servers: u16,
    pub game_severs_max_port_for_signaling_servers: u16,
    
    pub game_severs_min_port_for_tcp_listener: u16,
    pub game_severs_max_port_for_tcp_listener: u16,

    pub game_servers_ice_config: GameServersIceConfig,

    pub max_game_sessions: u32,

    pub max_players_per_game_session: u32,
}

trait ToOption<T> {
    fn none_if_zero(self) -> Option<T>;
}

impl ToOption<String> for String {
    fn none_if_zero(self) -> Option<String>
    {
        if self == "" {return None;}

        Some(self)
    }
}

#[derive(Clone)]
struct GameServersIceConfig {
    urls: String,
    username: String,
    credential: String,
}

impl GameServersIceConfig {
    pub fn parse_json(
        object: &Map<String, Value>
    ) -> Self
    {
        let object = {
            object
                .get("game_severs_ice_config")
                .expect("ERROR: Have not game_severs_ice_config in matchmaking-server-config.json")
                .as_object()
                .expect("ERROR: game_severs_ice_config is not object value in matchmaking-server-config.json")
        };

        let urls = {
            object
                .get("urls")
                .expect("ERROR: Have not urls in game_severs_ice_config in matchmaking-server-config.json")
                .as_array()
                .expect("ERROR: urls is not array value in game_severs_ice_config in matchmaking-server-config.json")
                .into_iter()
                .map(|s| {
                    s
                        .as_str()
                        .expect("ERROR: urls members is not string value in game_severs_ice_config in matchmaking-server-config.json")
                        .to_string() + "|"
                })
                .collect::<String>()
                // .none_if_zero()
        };

        let username = {
            object
                .get("username")
                .expect("ERROR: Have not username in game_severs_ice_config in matchmaking-server-config.json")
                .as_str()
                .expect("ERROR: username is not string value in game_severs_ice_config in matchmaking-server-config.json")
                .to_string()
                // .none_if_zero()
        };
        
        let credential = {
            object
                .get("credential")
                .expect("ERROR: Have not credential in game_severs_ice_config in matchmaking-server-config.json")
                .as_str()
                .expect("ERROR: credential is not string value in game_severs_ice_config in matchmaking-server-config.json")
                .to_string()
                // .none_if_zero()
        };

        GameServersIceConfig {
            urls,
            username,
            credential,
        }        
    }
}


#[derive(Clone)]
struct GameServerInfo {
    players_amount_by_matchmaking_server: u32,
    players_amount_by_game_server: u32,
    max_amount_of_players: u32,
    game_server_game_version: GameVersion,
    
    game_server_ip_address: Ipv4Addr,
    game_server_main_port: u16,

    matchmaking_server_listener_port: u16,
    
    server_index: u16,
    game_server_pid: u32,
}


type GameServersState = Arc<Mutex<HashMap<u16,GameServerInfo>>>;

async fn handle_client_connection(
    stream: tokio::net::TcpStream,
    state: GameServersState,
    mut config: Config,
    async_rutime: Arc<Runtime>,
)
{
    let ws_stream = accept_async(stream).await.unwrap();
    let (mut sender_to_client, mut receiver_from_client) = ws_stream.split();

    while let Some(Ok(msg)) = receiver_from_client.next().await {
        let message =
            alkahest::deserialize::<ClientMatchmakingServerProtocol, ClientMatchmakingServerProtocol>(&msg.into_data());

        if message.is_ok()
        {
            let message = message.unwrap();

            match message
            {
                ClientMatchmakingServerProtocol::ClientMessage(client_message) =>
                {
                    match client_message {
                        ClientMessage::RequestToConnectToGameServer(clients_game_version) => {

                            // uodate current game version and max game sessions amount

                            println!("INFO: Client is requesting to connect to a game server");

                            let clients_game_version = GameVersion::from(clients_game_version);

                            let mut locked_state = state.lock().await;
                            
                            let finded_server = locked_state.values_mut().find(
                                |server_info| {
                                    println!(
                                        "[{}] server has {} players by matchmaking server, max players per server is {}",
                                        server_info.server_index,
                                        server_info.players_amount_by_matchmaking_server,
                                        server_info.max_amount_of_players
                                    );

                                    server_info.players_amount_by_matchmaking_server < server_info.max_amount_of_players
                                    &&
                                    clients_game_version == server_info.game_server_game_version
                                }
                            );

                            match finded_server {

                                Some(server_info) => {

                                    println!("INFO: Free game server is finded, send to the client server's addres");

                                    server_info.players_amount_by_matchmaking_server += 1;

                                    let message = ClientMatchmakingServerProtocol::MatchmakingServerMessage(
                                        MatchmakingServerMessage::GameServerAddress((
                                            server_info.game_server_ip_address.octets(),
                                            server_info.game_server_main_port
                                        ))
                                    );

                                    let message: Vec<u8> = message.to_packet();

                                    sender_to_client
                                        .send(tokio_tungstenite::tungstenite::Message::binary( message))
                                        .await
                                        .unwrap();

                                    continue ;
                                }

                                None => {

                                    println!("INFO: Free game server is not finded, creating new one");

                                    // update config to change max_game_sessions, max_players_per_session and current_game_version dynamically
                                    config = load_config().await;

                                    if clients_game_version != config.current_game_version
                                    {
                                        println!("WARNING: Client's game version is not correct");

                                        let message = ClientMatchmakingServerProtocol::MatchmakingServerMessage(
                                            MatchmakingServerMessage::WrongGameVersionCorrectIs(config.current_game_version.clone().into())
                                        );

                                        let message: Vec<u8> = message.to_packet();

                                        sender_to_client
                                            .send(tokio_tungstenite::tungstenite::Message::binary(message))
                                            .await
                                            .unwrap();

                                        return ;
                                    }
                                    println!("INFO: Client's game version is correct");

                                    let free_port = get_free_server_port(
                                        &mut locked_state,
                                        config.max_game_sessions,
                                        config.game_severs_min_port_for_signaling_servers,
                                        config.game_severs_max_port_for_signaling_servers,
                                    );

                                    match free_port {
                                        Some(new_port) =>
                                        {
                                            let server_info = spawn_game_server(
                                                new_port,
                                                &config,
                                                async_rutime.clone(),
                                                state.clone(),
                                            ).await;

                                            if server_info.is_err()
                                            {
                                                println!("WARNING: Can not create new game server because is out of the limit");

                                                let message = ClientMatchmakingServerProtocol::MatchmakingServerMessage(
                                                    MatchmakingServerMessage::NoFreeServers
                                                );
            
                                                let message: Vec<u8> = message.to_packet();
            
                                                sender_to_client
                                                    .send(tokio_tungstenite::tungstenite::Message::binary( message))
                                                    .await
                                                    .unwrap();
            
                                                return ;
                                            }

                                            let server_info = server_info.unwrap();
        
                                            println!("INFO: New game server is successfully created, send to the client server's addres");
        
                                            locked_state.insert(server_info.server_index, server_info.clone());
        
                                            let message = ClientMatchmakingServerProtocol::MatchmakingServerMessage(
                                                MatchmakingServerMessage::GameServerAddress((
                                                    server_info.game_server_ip_address.octets(),
                                                    server_info.game_server_main_port
                                                ))
                                            );
        
                                            let message: Vec<u8> = message.to_packet();
        
                                            sender_to_client
                                                .send(tokio_tungstenite::tungstenite::Message::binary( message))
                                                .await
                                                .unwrap();
        
                                            continue ;
                                        }
                                        None =>
                                        {
                                            println!("WARNING: Can not create new game server because is out of the limit");

                                            let message = ClientMatchmakingServerProtocol::MatchmakingServerMessage(
                                                MatchmakingServerMessage::NoFreeServers
                                            );
        
                                            let message: Vec<u8> = message.to_packet();
        
                                            sender_to_client
                                                .send(tokio_tungstenite::tungstenite::Message::binary( message))
                                                .await
                                                .unwrap();
        
                                            return ;
                                        }
                                    }
                                }
                            }
                        }
                    }
                },
                _ => {
                    println!("WARNING: Incorrect request to the matchmaking server on client's port ");
                    return ;
                }
            }
        }
    }
}


fn get_free_server_port(
    locked_state:  &mut MutexGuard<'_, HashMap<u16, GameServerInfo>>,
    max_game_sessions: u32,
    min_port: u16,
    max_port: u16,
) -> Option<u16>
{
    if locked_state.len() >= max_game_sessions as usize {
        return None;
    }

    for port in min_port..=max_port {
        if !locked_state.contains_key(&port) {
            return Some(port);
        }
    }
    None
}


async fn spawn_game_server(
    port: u16,
    config: &Config,
    async_rutime: Arc<Runtime>,
    state: GameServersState,
) -> Result<GameServerInfo, ()>
{
    let server_process = Command::new("./game_server")
        .arg(port.to_string())
        .arg(config.game_severs_min_port_for_signaling_servers.to_string())
        .arg(config.game_severs_max_port_for_signaling_servers.to_string())
        .arg(config.game_severs_min_port_for_tcp_listener.to_string())
        .arg(config.game_severs_max_port_for_tcp_listener.to_string())
        .arg("127.0.0.1") //here will be config.matchmaking_server_ip.to_string()
        .arg(config.matchmaking_server_port_for_servers.to_string())
        .arg(config.max_players_per_game_session.to_string())
        .arg(config.game_servers_ice_config.urls.clone())
        .arg(config.game_servers_ice_config.username.clone())
        .arg(config.game_servers_ice_config.credential.clone())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    let mut server_process = match server_process {
        Ok(server_process) => server_process,
        Err(e) => return Err(()), 
    };

    let server_stdout = server_process.stdout.take().unwrap();
    let mut server_stderr = server_process.stderr.take().unwrap();
    let game_server_pid = server_process.id().unwrap();

    let mut server_stdout_reader = BufReader::new(server_stdout).lines();

    while let Ok(Some(line)) = server_stdout_reader.next_line().await {
        if line.contains("game server is ready") {

            let lines: Vec<&str> = line.split("|").collect();

            let game_server_main_port: u16 = lines[1].parse().unwrap();
            let matchmaking_server_listener_port: u16 = lines[2].parse().unwrap();

            println!("INFO: spawn new game server [{}] on {} port", game_server_main_port, game_server_main_port);

            async_rutime.spawn(
                keep_server_process(
                    server_process,
                    server_stdout_reader,
                    server_stderr,
                    game_server_main_port,
                    state,
                )
            );

            return Ok(GameServerInfo {
                game_server_ip_address: config.game_severs_public_ip,
                players_amount_by_matchmaking_server: 1_u32,
                players_amount_by_game_server: 0_u32,
                max_amount_of_players: config.max_players_per_game_session,
                game_server_game_version: config.current_game_version,
                game_server_main_port,
                matchmaking_server_listener_port,
                server_index: game_server_main_port,
                game_server_pid,
            });
        }
        else
        {

            println!("INFO: [{} (possible index)] server stdout is: {}", port, line);
            continue ;
        }
    }

    let mut errors = String::new();

    server_stderr.read_to_string(&mut errors).await.unwrap();

    panic!("ERROR: [{}] server spawned, but not ready, the server's Stderr is: {}", port, errors)
}


async fn keep_server_process(
    server_process: Child,
    mut server_stdout_reader: Lines<BufReader<ChildStdout>>,
    mut server_stderr: ChildStderr,
    server_index: u16,
    state: GameServersState,
) {
    while let Some(line) = server_stdout_reader.next_line().await.unwrap() {
        println!("INFO: [{}] server stdout is: {}", server_index, line);
        continue ;
    }

    let mut locked_state = state.lock().await;

    locked_state.remove(&server_index);

    let mut err = String::new();

    server_stderr.read_to_string(&mut err).await.unwrap();

    println!("{}", err);
}

fn main() {
    let args = std::env::args().collect();
    read_args(&args);

    let runtime = Arc::new(
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .worker_threads(1)
            .build()
            .unwrap()
    );

    runtime.block_on(async_main(runtime.clone()));
}

async fn async_main(
    async_runtime: Arc<Runtime>
) {
    let config = load_config().await;

    let game_servers_state = Arc::new(Mutex::new(HashMap::<u16, GameServerInfo>::new()));
    
    let clients_listener = TcpListener::bind(
        &("0.0.0.0:".to_string() + &config.matchmaking_server_port_for_clients.to_string())
    ).await.unwrap();

    async_runtime.spawn(check_game_servers_status(
        game_servers_state.clone(),
        async_runtime.clone(),
    ));
    
    loop
    {
        match clients_listener.accept().await
        {
            Ok((stream, _)) => {
                async_runtime.spawn(
                    handle_client_connection(
                        stream,
                        game_servers_state.clone(),
                        config.clone(),
                        async_runtime.clone(),
                    )
                );
            }
            Err(e) => {
                panic!("ERROR: clients listener error, err: {}", e)
            }
        }  
    }
}


async fn check_game_servers_status(
    game_servers_state: GameServersState,
    async_runtime: Arc<Runtime>,
)
{
    loop
    {
        let mut locked_state = game_servers_state
            .lock()
            .await;

        let mut game_servers_list_to_be_stopped = Vec::new();

        for (game_server_index, game_server_info) in locked_state.iter_mut()
        {
            let mut stop_server = false;

            let stream = match tokio::time::timeout(
                Duration::from_millis(300),
                TcpStream::connect(("127.0.0.1", game_server_info.matchmaking_server_listener_port))
            ).await
            {
                Ok(Ok(mut stream)) => {
                    Some(stream)
                },
                _ => None,
            };

            match stream {
                Some(mut stream) =>
                {
                    let message = GameServerMatchmakingServerProtocol::MatchmakingServerMessageToGameServer(
                        matchmaking_server_protocol::MatchmakingServerMessageToGameServer::GiveMePlayersAmount
                    );

                    match stream.write_all(&message.to_packet()).await
                    {
                        Ok(_) =>
                        {
                            let mut buf = Vec::new();

                            match tokio::time::timeout(
                                Duration::from_millis(300),
                                stream.read_buf(&mut buf)
                            ).await
                            {
                                Ok(Ok(_)) => {

                                    let message = alkahest::deserialize::<GameServerMatchmakingServerProtocol, GameServerMatchmakingServerProtocol>(&buf);

                                    match message
                                    {
                                        Ok(message) =>
                                        {
                                            match message
                                            {
                                                GameServerMatchmakingServerProtocol::GameServerMessage(message) =>
                                                {
                                                    match message {
                                                        GameServerMessage::PlayersAmoutIs(players_amount) =>
                                                        {
                                                            game_server_info.players_amount_by_game_server = players_amount;

                                                            println!("[{}] game server has {} of {} players", game_server_info.server_index, players_amount, game_server_info.max_amount_of_players);

                                                        }
                                                    }
                                                }
                                                GameServerMatchmakingServerProtocol::MatchmakingServerMessageToGameServer(_) =>
                                                {
                                                    println!("ERROR, matchmaking server recieved MatchmakingServerMessageToGameServer from [{}] game server, this server will be stoped", game_server_info.server_index);
                                                    
                                                    stop_server = true
                                                }
                                            }
                                        }
                                        Err(_) =>
                                        {
                                            println!("[{}] game server will stopped because of message deserialization error", game_server_index);
                                            stop_server = true
                                        }
                                    }
                                },

                                _ => {
                                    println!("[{}] game server will stopped because of answer timeout", game_server_index);
                                    stop_server = true;
                                }
                            };
                        }
                        Err(_) =>
                        {
                            println!("[{}] game server will stopped because of send message to game server error", game_server_index);
                            stop_server = true;

                        }
                    }
                }
                None =>
                {
                    println!("[{}] game server will stopped because of tcp connect to game server timeout", game_server_index);
                    stop_server = true;
                }
            }
            
            if stop_server
            {
                game_servers_list_to_be_stopped.push((*game_server_index, game_server_info.game_server_pid));
            }
        }

        while let Some((game_server_index, game_server_pid)) = game_servers_list_to_be_stopped.pop()
        {
            locked_state.remove(&game_server_index);
            println!("[{}] game server is stopped", game_server_index);
            
            async_runtime.spawn(stop_game_server(game_server_pid));
        }

        let mut total_players_amount = 0u32;

        for (_, game_server_info) in locked_state.iter_mut()
        {
            game_server_info.players_amount_by_matchmaking_server = game_server_info.players_amount_by_game_server;

            total_players_amount += game_server_info.players_amount_by_game_server;
        }

        println!("{} game servers is running with {} player in total", locked_state.len(), total_players_amount);
        
        tokio::time::sleep(Duration::from_secs(4)).await;
    }
}


async fn stop_game_server(pid: u32)
{
    let kill_cmd = Command::new("kill")
        .arg("-s")
        .arg("SIGKILL")
        .arg(pid.to_string())
        .spawn();

    if kill_cmd.is_err()
    {
        return;
    }

    let _ = kill_cmd.unwrap().wait().await;
}


async fn load_config() -> Config 
{
    let mut file = File::open("./matchmaking-server-config.json")
        .await
        .expect("ERROR: matchmaking-server-config.json file expected");

    let mut file_content = String::new();
    let config = match file.read_to_string(&mut file_content).await {
        Ok(_) => {
            let json_config = serde_json::from_str(&file_content)
                .expect("ERROR: can't parse matchmaking-server-config.json file");

            parse_json_matchmaking_config(json_config)
        },
        Err(e) => {
            panic!(
                "ERROR: the matchmaking-server-config.json cannot be loaded, err: {}",
                e.to_string()
            );
        }
    };

    return config;
}


fn parse_json_matchmaking_config(json_config: Value) -> Config
{
    let object = json_config
        .as_object()
        .expect("ERROR: Wrong JSON config format");

    let matchmaking_server_port_for_clients = {
        object
            .get("matchmaking_server_port_for_clients")
            .expect("ERROR: Have not matchmaking_server_port_for_clients in matchmaking-server-config.json")
            .as_i64()
            .expect("ERROR: matchmaking_server_port_for_clients is not number value in matchmaking-server-config.json")
            as u16
    };

    let matchmaking_server_port_for_servers = {
        object
            .get("matchmaking_server_port_for_servers")
            .expect("ERROR: Have not matchmaking_server_port_for_servers in matchmaking-server-config.json")
            .as_i64()
            .expect("ERROR: matchmaking_server_port_for_servers is not number value in matchmaking-server-config.json")
            as u16
    };

    let game_severs_public_ip = {
        object
            .get("game_severs_public_ip")
            .expect("ERROR: Have not game_severs_public_ip in matchmaking-server-config.json")
            .as_str()
            .expect("ERROR: game_severs_public_ip is not string value in matchmaking-server-config.json")
            .to_string()
    };

    let game_severs_public_ip = Ipv4Addr::from_str(&game_severs_public_ip)
        .expect("ERROR: wrong game_severs_public_ip ip address format");

    let matchmaking_server_ip = {
        object
            .get("matchmaking_server_ip")
            .expect("ERROR: Have not matchmaking_server_ip in matchmaking-server-config.json")
            .as_str()
            .expect("ERROR: matchmaking_server_ip is not string value in matchmaking-server-config.json")
            .to_string()
    };

    let matchmaking_server_ip = Ipv4Addr::from_str(&matchmaking_server_ip)
        .expect("ERROR: wrong matchmaking_server_ip ip address format");

    let game_severs_min_port_for_signaling_servers = {
        object
            .get("game_severs_min_port_for_signaling_servers")
            .expect("ERROR: Have not game_severs_min_port_for_signaling_servers in matchmaking-server-config.json")
            .as_i64()
            .expect("ERROR: game_severs_min_port_for_signaling_servers is not number value in matchmaking-server-config.json")
            as u16
    };

    let game_severs_max_port_for_signaling_servers = {
        object
            .get("game_severs_max_port_for_signaling_servers")
            .expect("ERROR: Have not game_severs_max_port_for_signaling_servers in matchmaking-server-config.json")
            .as_i64()
            .expect("ERROR: game_severs_max_port_for_signaling_servers is not number value in matchmaking-server-config.json")
            as u16
    };

    let game_severs_min_port_for_tcp_listener = {
        object
            .get("game_severs_min_port_for_tcp_listener")
            .expect("ERROR: Have not game_severs_min_port_for_tcp_listener in matchmaking-server-config.json")
            .as_i64()
            .expect("ERROR: game_severs_min_port_for_tcp_listener is not number value in matchmaking-server-config.json")
            as u16
    };

    let game_severs_max_port_for_tcp_listener = {
        object
            .get("game_severs_max_port_for_tcp_listener")
            .expect("ERROR: Have not game_severs_max_port_for_tcp_listener in matchmaking-server-config.json")
            .as_i64()
            .expect("ERROR: game_severs_max_port_for_tcp_listener is not number value in matchmaking-server-config.json")
            as u16
    };

    let max_game_sessions = {
        object
            .get("max_game_sessions")
            .expect("ERROR: Have not max_game_sessions in matchmaking-server-config.json")
            .as_i64()
            .expect("ERROR: max_game_sessions is not number value in matchmaking-server-config.json")
            as u32
    };

    let max_players_per_game_session = {
        object
            .get("max_players_per_game_session")
            .expect("ERROR: Have not max_players_per_game_session in matchmaking-server-config.json")
            .as_i64()
            .expect("ERROR: max_players_per_game_session is not number value in matchmaking-server-config.json")
            as u32
    };

    let current_game_version = {
        object
            .get("current_game_version")
            .expect("ERROR: Have not current_game_version in matchmaking-server-config.json")
            .as_str()
            .expect("ERROR: current_game_version is not string value in matchmaking-server-config.json")
    };

    let current_game_version = GameVersion::from(current_game_version);

    let game_servers_ice_config = GameServersIceConfig::parse_json(object);

    Config {
        matchmaking_server_ip,
        current_game_version,
        matchmaking_server_port_for_clients,
        matchmaking_server_port_for_servers,
        game_severs_public_ip,
        game_severs_min_port_for_signaling_servers,
        game_severs_max_port_for_signaling_servers,
        game_severs_min_port_for_tcp_listener,
        game_severs_max_port_for_tcp_listener,
        game_servers_ice_config,
        max_game_sessions,
        max_players_per_game_session,  
    }
}


pub fn read_args(args: &Vec<String>)
{
    for arg in args
    {
        match arg.as_str()
        {

            "--help" | "-help" | "help" | "-h" | "--usage" | "-usage" | "usage" =>
            {
                println!("Usage: ./matchmaking_server [OPTIONS]");
                println!();
                println!("  -v --v -version, --version,  Show current matchmaking server version");

                std::process::exit(0);
            }

            "-v" | "--v" | "-version" | "--version" =>
            {
                println!("Slice: 4D Shooter matchmaking server version: {}", env!("CARGO_PKG_VERSION"));
                
                std::process::exit(0);

            }

            _ => {}
        }
    }
}