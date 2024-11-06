use std::{
    fs::File, io::Read, net::Ipv4Addr, str::FromStr, sync::{Arc, Mutex}
};

use serde_json::Value;

use tokio::net::TcpListener;
use tokio::process::Command;
use tokio_tungstenite::accept_async;
use futures_util::{StreamExt, SinkExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
struct ConnectionRequest {
    client_version: String,
}

#[derive(Serialize, Deserialize)]
struct ServerInfo {
    ip: String,
    port: u16,
}


struct Config
{
    pub matchmaking_server_port_for_clients: u32,
    pub matchmaking_server_port_for_servers: u32,

    pub game_severs_public_ip: String,
    pub game_severs_min_port: u32,
    pub game_severs_max_port: u32,

    pub max_game_sessions: u32,

    pub max_players_per_game_session: u32,
}

type ServerState = Arc<Mutex<HashMap<u16, ServerInfo>>>;

async fn handle_client_connection(raw_stream: tokio::net::TcpStream, state: ServerState) {
    let ws_stream = accept_async(raw_stream).await.unwrap();
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    // Получаем запрос от клиента
    if let Some(Ok(msg)) = ws_receiver.next().await {
        let request: ConnectionRequest = serde_json::from_str(msg.to_text().unwrap()).unwrap();

        // Проверяем, есть ли доступный игровой сервер
        let mut state_guard = state.lock().unwrap();
        let server_info = state_guard.values().find(|info| /* логика проверки свободных мест */ true);

        let response = if let Some(info) = server_info {
            // Если сервер найден, возвращаем информацию о сервере
            serde_json::to_string(info).unwrap()
        } else {
            // Если сервер не найден, спавним новый
            let new_port = 9000 + state_guard.len() as u16;
            let server_info = spawn_game_server(new_port).await.unwrap();
            state_guard.insert(new_port, server_info.clone());
            serde_json::to_string(&server_info).unwrap()
        };

        ws_sender.send(tokio_tungstenite::tungstenite::Message::Text(response)).await.unwrap();
    }
}

// Функция для спавна игрового сервера
async fn spawn_game_server(port: u16) -> Result<ServerInfo, Box<dyn std::error::Error>> {
    // Запуск игрового сервера как отдельного процесса с передачей порта
    Command::new("path/to/game_server_binary")
        .arg(port.to_string())
        .spawn()?
        .wait()
        .await?;

    // Параметры, которые сервер вернет клиенту
    Ok(ServerInfo {
        ip: "127.0.0.1".to_string(), // Заменить на IP сервера
        port,
    })
}

async fn handle_server_message(raw_stream: tokio::net::TcpStream, state: ServerState)
{

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

    tokio::spawn(async {
        loop
        {
            match servers_listener.accept().await
            {
                Ok((stream, _)) => {
                    let state = Arc::clone(&state);
                    tokio::spawn(handle_server_message(stream, state));
                }
                Err(e) => {
                    panic!("ERROR: Server listener error, err: {}", e)
                }
            }  
        }
    });

    loop
    {
        match clients_listener.accept().await
        {
            Ok((stream, _)) => {
                let state = Arc::clone(&state);
                tokio::spawn(handle_client_connection(stream, state));
            }
            Err(e) => {
                panic!("ERROR: Clients listener error, err: {}", e)
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
            as u32
    };

    let matchmaking_server_port_for_servers = {
        object
            .get("ERROR: matchmaking_server_port_for_servers")
            .expect("ERROR: Have not matchmaking_server_port_for_servers in matchmaking-server-config.json")
            .as_i64()
            .expect("ERROR: matchmaking_server_port_for_servers is not number value in matchmaking-server-config.json")
            as u32
    };

    let game_severs_public_ip = {
        object
            .get("ERROR: game_severs_public_ip")
            .expect("ERROR: Have not game_severs_public_ip in matchmaking-server-config.json")
            .as_str()
            .expect("ERROR: game_severs_public_ip is not string value in matchmaking-server-config.json")
            .to_string()
    };

    let game_severs_min_port = {
        object
            .get("ERROR: game_severs_min_port")
            .expect("ERROR: Have not game_severs_min_port in matchmaking-server-config.json")
            .as_i64()
            .expect("ERROR: game_severs_min_port is not number value in matchmaking-server-config.json")
            as u32
    };

    let game_severs_max_port = {
        object
            .get("ERROR: game_severs_max_port")
            .expect("ERROR: Have not game_severs_max_port in matchmaking-server-config.json")
            .as_i64()
            .expect("ERROR: game_severs_max_port is not number value in matchmaking-server-config.json")
            as u32
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

    Ipv4Addr::from_str(&game_severs_public_ip).map_err(|_|
        panic!(
            "ERROR: wrong game_severs_public_ip ip address format"
        )
    );

    Config {
        matchmaking_server_port_for_clients,
        matchmaking_server_port_for_servers,
        game_severs_public_ip,
        game_severs_min_port,
        game_severs_max_port,
        max_game_sessions,
        max_players_per_game_session,  
    }
}