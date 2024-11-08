use std::{
    fs::File,
    io::Read,
    net::Ipv4Addr,
    str::FromStr,
    sync::Arc
};
use tokio::{
    sync::Mutex,
    net::TcpListener,
    process::Command
};
use fyrox_core::futures::{SinkExt, StreamExt};
use serde_json::Value;
use tokio_tungstenite::accept_async;
use std::collections::HashMap;
use alkahest::{alkahest, Serialize};

#[repr(C)]
#[alkahest(Formula, Serialize, Deserialize)]
enum MatchmakingServerMessages
{
    GameServerAddress(([u8;4],u16)),
    NoFreeServers,
    WrongGameVersionCorrectIs((u32,u32,u32))
}

#[repr(C)]
#[alkahest(Formula, Serialize, Deserialize)]
enum ClientMessages
{
    RequestToConnectToGameServer((u32,u32,u32))
}

#[repr(C)]
#[alkahest(Formula, Serialize, Deserialize)]
enum ClientMatchmakingServerProtocol
{
    ServerMessages(MatchmakingServerMessages),
    ClientMessages(ClientMessages)
}

impl Into<Vec<u8>> for ClientMatchmakingServerProtocol
{
    fn into(self) -> Vec<u8> {

        let size = <
            ClientMatchmakingServerProtocol as
            Serialize<ClientMatchmakingServerProtocol>
        >::size_hint(&self).unwrap();
        
        let mut packet: Vec<u8> = Vec::with_capacity(size.heap);

        alkahest::serialize_to_vec::<
            ClientMatchmakingServerProtocol,
            ClientMatchmakingServerProtocol
        >(self, &mut packet);

        packet
    }
}

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
    players_amount: u32,
    
    siganling_server_ip_address: Ipv4Addr,
    signaling_server_port: u16,
    
    server_index: u16,
}

type GameServersState = Arc<Mutex<HashMap<u16,GameServerInfo>>>;

#[derive(Clone)]
pub struct GameVersion {
    pub major: u32,
    pub minor: u32,
    pub maintenance: u32,
}

impl Into<(u32,u32,u32)> for GameVersion {
    fn into(self) -> (u32,u32,u32) {
        (
            self.major,
            self.minor,
            self.maintenance
        )
    }
}

impl From<(u32,u32,u32)> for GameVersion
{
    fn from(value: (u32,u32,u32)) -> Self {
        GameVersion {
            major: value.0,
            minor: value.1,
            maintenance: value.2,
        }
    }
}

impl PartialEq for GameVersion {
    fn eq(&self, other: &Self) -> bool {

        if  self.major==other.major &&
            self.minor==other.minor &&
            self.maintenance==other.maintenance
        {
            return true;
        }

        false
    }
}

impl PartialOrd for GameVersion {

    fn gt(&self, other: &Self) -> bool {
        if self.major > other.major {
            return true;
        }
        if self.major < other.major {
            return false;
        }
        if self.minor > other.minor {
            return true;
        }
        if self.minor < other.minor {
            return false;
        }
        if self.maintenance > other.maintenance {
            return true;
        }
        false
    }

    fn ge(&self, other: &Self) -> bool {
        if self > other {
            return true;
        }
        if self == other {
            return true;
        }
        false
    }

    fn lt(&self, other: &Self) -> bool {
        if self > other {
            return false;
        }
        if self == other {
            return false;
        }
        true
    }

    fn le(&self, other: &Self) -> bool {
        if self > other {
            return false;
        }
        true
    }

    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self == other {
            return Some(std::cmp::Ordering::Equal);
        }
        if self > other {
            return Some(std::cmp::Ordering::Greater);
        }
        Some(std::cmp::Ordering::Less)
    }
}

async fn handle_client_connection(stream: tokio::net::TcpStream, state: GameServersState, config: Config) {

    let mut locked_state = state.lock().await;

    let ws_stream = accept_async(stream).await.unwrap();
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    if let Some(Ok(msg)) = ws_receiver.next().await {
        let message =
            alkahest::deserialize::<ClientMatchmakingServerProtocol, ClientMatchmakingServerProtocol>(&msg.into_data());

        if message.is_ok()
        {
            let message = message.unwrap();

            match message
            {
                ClientMatchmakingServerProtocol::ClientMessages(client_message) =>
                {
                    match client_message {
                        ClientMessages::RequestToConnectToGameServer(clients_game_version) => {

                            let clients_game_version = GameVersion::from(clients_game_version);

                            if clients_game_version != config.current_game_version
                            {
                                let message = ClientMatchmakingServerProtocol::ServerMessages(
                                    MatchmakingServerMessages::WrongGameVersionCorrectIs(config.current_game_version.clone().into())
                                );

                                let message: Vec<u8> = message.into();

                                ws_sender
                                    .send(tokio_tungstenite::tungstenite::Message::binary(message))
                                    .await
                                    .unwrap();

                                return ;
                            }
                            
                            let finded_server = locked_state.values().find(
                                |server_info| {
                                    server_info.players_amount < config.max_players_per_game_session
                                }
                            );

                            match finded_server {
                                Some(server_info) => {

                                    let message = ClientMatchmakingServerProtocol::ServerMessages(
                                        MatchmakingServerMessages::GameServerAddress((
                                            server_info.siganling_server_ip_address.octets(),
                                            server_info.signaling_server_port
                                        ))
                                    );

                                    let message: Vec<u8> = message.into();

                                    ws_sender
                                        .send(tokio_tungstenite::tungstenite::Message::binary( message))
                                        .await
                                        .unwrap();

                                    return ;
                                }

                                None => {

                                    let new_port = config.game_severs_min_port + locked_state.len() as u16;

                                    let server_info = spawn_game_server(
                                        config.matchmaking_server_ip.clone(),
                                        new_port,
                                        &config
                                    ).await.unwrap();

                                    locked_state.insert(server_info.server_index, server_info.clone());

                                    let message = ClientMatchmakingServerProtocol::ServerMessages(
                                        MatchmakingServerMessages::GameServerAddress((
                                            server_info.siganling_server_ip_address.octets(),
                                            server_info.signaling_server_port
                                        ))
                                    );

                                    let message: Vec<u8> = message.into();

                                    ws_sender
                                        .send(tokio_tungstenite::tungstenite::Message::binary( message))
                                        .await
                                        .unwrap();

                                    return ;

                                }
                            }

                        }
                    }
                },
                _ => {return ;}
            }
        }
    }
}

async fn spawn_game_server(
    matchmaking_server_ip: Ipv4Addr,
    port: u16,
    config: &Config
) -> Result<GameServerInfo, Box<dyn std::error::Error>>
{
    // Command::new("path/to/game_server_binary")
    //     .arg(port.to_string())
    //     .spawn()?
    //     .wait()
    //     .await?;

    Ok(GameServerInfo {
        siganling_server_ip_address: config.matchmaking_server_ip,
        players_amount: 0_u32,
        signaling_server_port: port,
        server_index: port,
    })
}

async fn handle_server_message(
    raw_stream: tokio::net::TcpStream,
    state: GameServersState,
    config: Config
) {

}


#[tokio::main]
async fn main() {
    let config = load_config();

    let state = Arc::new(Mutex::new(HashMap::new()));
    
    let clients_listener = TcpListener::bind(
        &("127.0.0.1:".to_string() + &config.matchmaking_server_port_for_clients.to_string())
    ).await.unwrap();

    let servers_listener = TcpListener::bind(
        &("127.0.0.1:".to_string() + &config.matchmaking_server_port_for_servers.to_string())
    ).await.unwrap();

    let state2 = state.clone();
    let config2 = config.clone();
    tokio::spawn(async move {
        loop
        {
            match servers_listener.accept().await
            {
                Ok((stream, _)) => {
                    tokio::spawn(handle_server_message(stream, state2.clone(), config2.clone()));
                }
                Err(e) => {
                    panic!("ERROR: game servers listener error, err: {}", e)
                }
            }  
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