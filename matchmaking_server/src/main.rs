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
    fs::File,
    io::Read,
    net::Ipv4Addr,
    process::Stdio,
    str::FromStr,
    sync::Arc,
    time::Duration
};
use tokio::{
    io::{
        AsyncBufReadExt,
        AsyncReadExt,
        BufReader,
        Lines
    },
    process::{
        Child,
        ChildStderr,
        ChildStdout,
        Command
    },
    sync::{
        Mutex,
        MutexGuard
    },
    net::TcpListener,
    runtime::Runtime,
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
    pub game_severs_min_port: u16,
    pub game_severs_max_port: u16,
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
    
    game_server_ip_address: Ipv4Addr,
    game_server_port: u16,
    
    server_index: u16,
}

type GameServersState = Arc<Mutex<HashMap<u16,GameServerInfo>>>;

async fn handle_client_connection(
    stream: tokio::net::TcpStream,
    state: GameServersState,
    config: Config,
    async_rutime: Arc<Runtime>,
) {

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

                            println!("INFO: Client is requesting to connect to a game server");

                            let clients_game_version = GameVersion::from(clients_game_version);

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

                            let mut locked_state = state.lock().await;
                            
                            let finded_server = locked_state.values_mut().find(
                                |server_info| {
                                    println!(
                                        "[{}] server has {} players by matchmaking server, max players per server is {}",
                                        server_info.server_index,
                                        server_info.players_amount_by_matchmaking_server,
                                        config.max_players_per_game_session
                                    );
                                    server_info.players_amount_by_matchmaking_server < config.max_players_per_game_session
                                }
                            );

                            match finded_server {

                                Some(server_info) => {

                                    println!("INFO: Free game server is finded, send to the client server's addres");

                                    server_info.players_amount_by_matchmaking_server += 1;

                                    let message = ClientMatchmakingServerProtocol::MatchmakingServerMessage(
                                        MatchmakingServerMessage::GameServerAddress((
                                            server_info.game_server_ip_address.octets(),
                                            server_info.game_server_port
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

                                    let free_port = get_free_server_port(
                                        &mut locked_state,
                                        config.max_game_sessions,
                                        config.game_severs_min_port,
                                        config.game_severs_max_port,
                                    );

                                    match free_port {
                                        Some(new_port) =>
                                        {
                                            let server_info = spawn_game_server(
                                                new_port,
                                                &config,
                                                async_rutime.clone()
                                            ).await.unwrap();
        
                                            println!("INFO: New game server is successfully created, send to the client server's addres");
        
                                            locked_state.insert(server_info.server_index, server_info.clone());
        
                                            let message = ClientMatchmakingServerProtocol::MatchmakingServerMessage(
                                                MatchmakingServerMessage::GameServerAddress((
                                                    server_info.game_server_ip_address.octets(),
                                                    server_info.game_server_port
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
) -> Result<GameServerInfo, Box<dyn std::error::Error>>
{
    let mut server_proccess = Command::new("./game_server")
        .arg(port.to_string())
        .arg("127.0.0.1") //here will be config.matchmaking_server_ip.to_string() 
        .arg(config.matchmaking_server_port_for_servers.to_string())
        .arg(config.max_players_per_game_session.to_string())
        .arg(config.game_servers_ice_config.urls.clone())
        .arg(config.game_servers_ice_config.username.clone())
        .arg(config.game_servers_ice_config.credential.clone())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let server_stdout = server_proccess.stdout.take().unwrap();
    let mut server_stderr = server_proccess.stderr.take().unwrap();

    let mut server_stdout_reader = BufReader::new(server_stdout).lines();

    while let Some(line) = server_stdout_reader.next_line().await? {
        if line.trim() == "ready" {

            println!("INFO: spawn new game server [{}] on {} port", port, port);

            async_rutime.spawn(
                keep_server_process(
                    server_proccess,
                    server_stdout_reader,
                    server_stderr,
                    port
                )
            );

            return Ok(GameServerInfo {
                game_server_ip_address: config.game_severs_public_ip,
                players_amount_by_matchmaking_server: 1_u32,
                players_amount_by_game_server: 0_u32,
                game_server_port: port,
                server_index: port,
            });
        } else {

            println!("INFO: [{}] server stdout is: {}", port, line);
            continue ;
        }
    }

    let mut errors = String::new();

    server_stderr.read_to_string(&mut errors).await.unwrap();

    panic!("ERROR: [{}] server spawned, but not ready, the server's Stderr is: {}", port, errors)
}


async fn keep_server_process(
    server_proccess: Child,
    mut server_stdout_reader: Lines<BufReader<ChildStdout>>,
    server_stderr: ChildStderr,
    server_index: u16,
) {
    while let Some(line) = server_stdout_reader.next_line().await.unwrap() {
        println!("INFO: [{}] server stdout is: {}", server_index, line);
        continue ;
    }
}


async fn handle_server_message(
    stream: tokio::net::TcpStream,
    state: GameServersState,
    _config: Config
) {
    let ws_stream = accept_async(stream).await.unwrap();

    println!("INFO: websocket connection with game server is opened");

    let (_, mut ws_receiver) = ws_stream.split();

    while let Some(Ok(msg)) = ws_receiver.next().await {
        let message =
            alkahest::deserialize::<GameServerMatchmakingServerProtocol, GameServerMatchmakingServerProtocol>(&msg.into_data());

        if message.is_ok()
        {
            match message.unwrap() {
                GameServerMatchmakingServerProtocol::GameServerMessage(message) => {
                    match message {
                        GameServerMessage::PlayerConnected(game_server_index) => {
                            match state.lock().await.get_mut(&game_server_index) {
                                
                                Some(server_info) => {
                                    
                                    println!("INFO: new player is connected to the {} server", &game_server_index);
                                    
                                    server_info.players_amount_by_game_server += 1;
                                },
                                None => {
                                    println!(
                                        "WARNING: get message from game server that is not exist in matchmaking server's game servers state"
                                    );

                                    return ;
                                }
                            }
                        },
                        GameServerMessage::PlayerDisconnected(game_server_index) => {
                            match state.lock().await.get_mut(&game_server_index) {
                                Some(server_info) => {

                                    println!("INFO: player is disconnected from the {} server", &game_server_index);

                                    if server_info.players_amount_by_game_server > 0 {
                                        server_info.players_amount_by_game_server -= 1;
                                    }
                                },
                                None => {
                                    println!(
                                        "WARNING: get message from game server that is not exist in matchmaking server's game servers state"
                                    );

                                    return ;
                                }
                            }
                        },
                        GameServerMessage::GameServerShutedDown(game_server_index) => {
                            match state.lock().await.remove(&game_server_index) {
                                Some(_) => {

                                    println!("[{}] server is shouted down", &game_server_index);

                                    return ;
                                },
                                None => {
                                    println!(
                                        "WARNING: get message from game server that is not exist in matchmaking server's game servers state"
                                    );

                                    return ;
                                }
                            }
                        },
                        GameServerMessage::ServerStarted(game_server_index) => {
                            println!("[{}] server has started", &game_server_index);
                        }
                    }
                }
                // _ => {
                //     println!("WARNING: Incorrect request to the matchmaking server on server's port");

                //     return ;
                // }
            }
        }
    }

    println!("INFO: websocket connection with game server is closed");
}

fn main() {
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
    let config = load_config();

    let state = Arc::new(Mutex::new(HashMap::new()));
    
    let clients_listener = TcpListener::bind(
        &("0.0.0.0:".to_string() + &config.matchmaking_server_port_for_clients.to_string())
    ).await.unwrap();

    let servers_listener = TcpListener::bind(
        &("0.0.0.0:".to_string() + &config.matchmaking_server_port_for_servers.to_string())
    ).await.unwrap();

    let cloned_state = state.clone();
    let config2 = config.clone();
    
    let runtime_2 = async_runtime.clone();
    async_runtime.spawn(async move {
        loop
        {
            match servers_listener.accept().await
            {
                Ok((stream, _)) => {
                    runtime_2.spawn(
                        handle_server_message(
                            stream,
                            cloned_state.clone(),
                            config2.clone()
                        )
                    );
                }
                Err(e) => {
                    panic!("ERROR: game servers listener error, err: {}", e)
                }
            }  
        }
    });

    let another_cloned_state = state.clone();
    async_runtime.spawn(async move {
        loop
        {
            another_cloned_state
                .lock()
                .await
                .values_mut()
                .for_each(|server_info|{
                    server_info.players_amount_by_matchmaking_server = server_info.players_amount_by_game_server;
                });
            
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    });

    loop
    {
        match clients_listener.accept().await
        {
            Ok((stream, _)) => {
                async_runtime.spawn(
                    handle_client_connection(
                        stream,
                        state.clone(),
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


fn load_config() -> Config 
{
    let mut file = File::open("./matchmaking-server-config.json")
        .expect("ERROR: matchmaking-server-config.json file expected");

    let mut file_content = String::new();
    match file.read_to_string(&mut file_content) {
        Ok(_) => {
            let json_config = serde_json::from_str(&file_content)
                .expect("ERROR: can't parse matchmaking-server-config.json file");

            return parse_json_matchmaking_config(json_config);
        },
        Err(e) => {
            panic!(
                "ERROR: the matchmaking-server-config.json cannot be loaded, err: {}",
                e.to_string()
            );
        }
    }
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

    let game_severs_min_port = {
        object
            .get("game_severs_min_port")
            .expect("ERROR: Have not game_severs_min_port in matchmaking-server-config.json")
            .as_i64()
            .expect("ERROR: game_severs_min_port is not number value in matchmaking-server-config.json")
            as u16
    };

    let game_severs_max_port = {
        object
            .get("game_severs_max_port")
            .expect("ERROR: Have not game_severs_max_port in matchmaking-server-config.json")
            .as_i64()
            .expect("ERROR: game_severs_max_port is not number value in matchmaking-server-config.json")
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
        game_severs_min_port,
        game_severs_max_port,
        game_servers_ice_config,
        max_game_sessions,
        max_players_per_game_session,  
    }
}