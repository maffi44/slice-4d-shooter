mod matchmaking_server_protocol;

use matchmaking_server_protocol::{
    GameServerMatchmakingServerProtocol,
    ClientMatchmakingServerProtocol,
    MatchmakingServerMessage,
    GameServerMessage,
    ClientMessage,
};

use core::panic;
use std::{
    fs::File,
    io::Read,
    net::Ipv4Addr,
    str::FromStr,
    sync::Arc, time::Duration
};
use tokio::{
    io::{
        AsyncBufReadExt,
        BufReader
    },
    net::TcpListener,
    process::Command,
    sync::Mutex
};

use fyrox_core::futures::{SinkExt, StreamExt};
use serde_json::Value;
use tokio_tungstenite::accept_async;
use std::collections::HashMap;
use alkahest::{alkahest, Serialize};

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

    pub max_game_sessions: u32,

    pub max_players_per_game_session: u32,
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

async fn handle_client_connection(stream: tokio::net::TcpStream, state: GameServersState, config: Config) {

    let mut locked_state = state.lock().await;

    let ws_stream = accept_async(stream).await.unwrap();
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    while let Some(Ok(msg)) = ws_receiver.next().await {
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

                                ws_sender
                                    .send(tokio_tungstenite::tungstenite::Message::binary(message))
                                    .await
                                    .unwrap();

                                return ;
                            }
                            println!("INFO: Client's game version is correct");
                            
                            let finded_server = locked_state.values_mut().find(
                                |server_info| {
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

                                    ws_sender
                                        .send(tokio_tungstenite::tungstenite::Message::binary( message))
                                        .await
                                        .unwrap();

                                    continue ;
                                }

                                None => {

                                    println!("INFO: Free game server is not finded, creating new one");

                                    let new_port = config.game_severs_min_port + locked_state.len() as u16;

                                    if new_port > config.game_severs_max_port || locked_state.len() as u32 >= config.max_game_sessions {

                                        println!("WARNING: Can not create new game server because is out of the limit");

                                        let message = ClientMatchmakingServerProtocol::MatchmakingServerMessage(
                                            MatchmakingServerMessage::NoFreeServers
                                        );
    
                                        let message: Vec<u8> = message.to_packet();
    
                                        ws_sender
                                            .send(tokio_tungstenite::tungstenite::Message::binary( message))
                                            .await
                                            .unwrap();
    
                                        return ;
                                    }

                                    let server_info = spawn_game_server(
                                        new_port,
                                        &config
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

                                    ws_sender
                                        .send(tokio_tungstenite::tungstenite::Message::binary( message))
                                        .await
                                        .unwrap();

                                    continue ;

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

async fn spawn_game_server(
    port: u16,
    config: &Config
) -> Result<GameServerInfo, Box<dyn std::error::Error>>
{
    let mut server_proccess = Command::new("./game-server")
        .arg(port.to_string())
        .arg("127.0.0.1") //here will be config.matchmaking_server_ip.to_string() 
        .arg(config.matchmaking_server_port_for_servers.to_string())
        .arg(config.max_players_per_game_session.to_string())
        .spawn()?;

    let server_stdout = server_proccess.stdout.take().unwrap();

    let mut reader = BufReader::new(server_stdout).lines();

    if let Some(line) = reader.next_line().await? {
        if line.trim() == "ready" {

            println!("spawn new game sever on {} port", port);

            return Ok(GameServerInfo {
                game_server_ip_address: config.game_severs_public_ip,
                players_amount_by_matchmaking_server: 1_u32,
                players_amount_by_game_server: 0_u32,
                game_server_port: port,
                server_index: port,
            });
        } else {

            panic!("server spawned, but not ready")

        }
    }
    unreachable!()
}

async fn handle_server_message(
    stream: tokio::net::TcpStream,
    state: GameServersState,
    _config: Config
) {

    let mut locked_state = state.lock().await;

    let ws_stream = accept_async(stream).await.unwrap();
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
                            match locked_state.get_mut(&game_server_index) {
                                
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
                            match locked_state.get_mut(&game_server_index) {
                                Some(server_info) => {

                                    println!("INFO: player is disconnected from the {} server", &game_server_index);

                                    server_info.players_amount_by_game_server -= 1;
                                },
                                None => {
                                    println!(
                                        "WARNING: get message from game server that is not exist in matchmaking server's game servers state"
                                    );

                                    return ;
                                }
                            }
                        },
                        GameServerMessage::GameServerHasShutDown(game_server_index) => {
                            match locked_state.remove(&game_server_index) {
                                Some(_) => {

                                    println!("{} server is shouted down", &game_server_index);

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
                        GameServerMessage::ServerHasStarted(game_server_index) => {
                            println!("{} server has started", &game_server_index);
                        }
                    }
                }
                _ => {
                    println!("WARNING: Incorrect request to the matchmaking server on server's port");

                    return ;
                }
            }
        }
    }
}

fn main() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .worker_threads(1)
        .build()
        .unwrap();

    runtime.block_on(async_main());
}

async fn async_main() {
    let config = load_config();

    let state = Arc::new(Mutex::new(HashMap::new()));
    
    let clients_listener = TcpListener::bind(
        &("127.0.0.1:".to_string() + &config.matchmaking_server_port_for_clients.to_string())
    ).await.unwrap();

    let servers_listener = TcpListener::bind(
        &("127.0.0.1:".to_string() + &config.matchmaking_server_port_for_servers.to_string())
    ).await.unwrap();

    let cloned_state = state.clone();
    let config2 = config.clone();
    tokio::spawn(async move {
        loop
        {
            match servers_listener.accept().await
            {
                Ok((stream, _)) => {
                    tokio::spawn(handle_server_message(stream, cloned_state.clone(), config2.clone()));
                }
                Err(e) => {
                    panic!("ERROR: game servers listener error, err: {}", e)
                }
            }  
        }
    });

    let another_cloned_state = state.clone();
    tokio::spawn(async move {
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
                tokio::spawn(handle_client_connection(stream, state.clone(), config.clone()));
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
            .get("ERROR: matchmaking_server_port_for_clients")
            .expect("ERROR: Have not matchmaking_server_port_for_clients in matchmaking-server-config.json")
            .as_i64()
            .expect("ERROR: matchmaking_server_port_for_clients is not number value in matchmaking-server-config.json")
            as u16
    };

    let matchmaking_server_port_for_servers = {
        object
            .get("ERROR: matchmaking_server_port_for_servers")
            .expect("ERROR: Have not matchmaking_server_port_for_servers in matchmaking-server-config.json")
            .as_i64()
            .expect("ERROR: matchmaking_server_port_for_servers is not number value in matchmaking-server-config.json")
            as u16
    };

    let game_severs_public_ip = {
        object
            .get("ERROR: game_severs_public_ip")
            .expect("ERROR: Have not game_severs_public_ip in matchmaking-server-config.json")
            .as_str()
            .expect("ERROR: game_severs_public_ip is not string value in matchmaking-server-config.json")
            .to_string()
    };

    let game_severs_public_ip = Ipv4Addr::from_str(&game_severs_public_ip)
        .expect("ERROR: wrong game_severs_public_ip ip address format");

    let matchmaking_server_ip = {
        object
            .get("ERROR: matchmaking_server_ip")
            .expect("ERROR: Have not matchmaking_server_ip in matchmaking-server-config.json")
            .as_str()
            .expect("ERROR: matchmaking_server_ip is not string value in matchmaking-server-config.json")
            .to_string()
    };

    let matchmaking_server_ip = Ipv4Addr::from_str(&matchmaking_server_ip)
        .expect("ERROR: wrong matchmaking_server_ip ip address format");

    let game_severs_min_port = {
        object
            .get("ERROR: game_severs_min_port")
            .expect("ERROR: Have not game_severs_min_port in matchmaking-server-config.json")
            .as_i64()
            .expect("ERROR: game_severs_min_port is not number value in matchmaking-server-config.json")
            as u16
    };

    let game_severs_max_port = {
        object
            .get("ERROR: game_severs_max_port")
            .expect("ERROR: Have not game_severs_max_port in matchmaking-server-config.json")
            .as_i64()
            .expect("ERROR: game_severs_max_port is not number value in matchmaking-server-config.json")
            as u16
    };

    let max_game_sessions = {
        object
            .get("ERROR: max_game_sessions")
            .expect("ERROR: Have not max_game_sessions in matchmaking-server-config.json")
            .as_i64()
            .expect("ERROR: max_game_sessions is not number value in matchmaking-server-config.json")
            as u32
    };

    let max_players_per_game_session = {
        object
            .get("ERROR: max_players_per_game_session")
            .expect("ERROR: Have not max_players_per_game_session in matchmaking-server-config.json")
            .as_i64()
            .expect("ERROR: max_players_per_game_session is not number value in matchmaking-server-config.json")
            as u32
    };

    let current_game_major_version = {
        object
            .get("ERROR: current_game_major_version")
            .expect("ERROR: Have not current_game_major_version in matchmaking-server-config.json")
            .as_i64()
            .expect("ERROR: current_game_major_version is not number value in matchmaking-server-config.json")
            as u32
    };

    let current_game_minor_version = {
        object
            .get("ERROR: current_game_minor_version")
            .expect("ERROR: Have not current_game_minor_version in matchmaking-server-config.json")
            .as_i64()
            .expect("ERROR: current_game_minor_version is not number value in matchmaking-server-config.json")
            as u32
    };

    let current_game_maintenance_version = {
        object
            .get("ERROR: current_game_maintenance_version")
            .expect("ERROR: Have not current_game_maintenance_version in matchmaking-server-config.json")
            .as_i64()
            .expect("ERROR: current_game_maintenance_version is not number value in matchmaking-server-config.json")
            as u32
    };

    

    let current_game_version = GameVersion::from((
        current_game_major_version,
        current_game_minor_version,
        current_game_maintenance_version
    ));

    

    Config {
        matchmaking_server_ip,
        current_game_version,
        matchmaking_server_port_for_clients,
        matchmaking_server_port_for_servers,
        game_severs_public_ip,
        game_severs_min_port,
        game_severs_max_port,
        max_game_sessions,
        max_players_per_game_session,  
    }
}