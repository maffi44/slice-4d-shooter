mod actor;
mod main_loop;
mod transform;
mod engine;
mod matchmaking_server_main;

use alkahest::{alkahest, Serialize};
use std::{
    env,
    net::{
        Ipv4Addr, SocketAddr, SocketAddrV4
    },
    process::exit,
    str::FromStr,
    sync::{
        Arc,
        Mutex
    },
    time::{Duration, Instant}
};
use matchmaking_server_main::{
    GameServerMatchmakingServerProtocol,
    GameServerMessage
};
use engine::net::NetMessage;

use fyrox_core::futures::SinkExt;
use matchbox_signaling::SignalingServer;
use matchbox_socket::{
    MultipleChannels, PeerId, PeerState::{Connected, Disconnected}, RtcIceServerConfig, WebRtcChannel, WebRtcSocket
};
use tokio::{
    runtime::{Builder, Runtime}, sync::mpsc::{
        channel, Receiver, Sender
    }, task::JoinHandle
};

use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

type Packet = Box<[u8]>;

#[repr(C)]
#[alkahest(Formula, Serialize, Deserialize)]
enum ClientMessage {
    DirectMessage(u128, NetMessage),
    BoardcastMessage(NetMessage),
}

impl ClientMessage {
    pub fn to_packet(self) -> Packet {
        
        let size = <ClientMessage as Serialize<ClientMessage>>::size_hint(&self).unwrap();
        
        let mut packet: Vec<u8> = Vec::with_capacity(size.heap);

        alkahest::serialize_to_vec::<ClientMessage, ClientMessage>(self, &mut packet);

        packet.into_boxed_slice()
    }

    pub fn from_packet(packet: Packet) -> Option<Self> {
        if let Ok(message) = alkahest::deserialize::<ClientMessage, ClientMessage>(&packet) {
            Some(message)
        } else {
            None
        }
    }
}


#[derive(Clone)]
struct GameServerConfig {
    signaling_port: u16,
    matchmaking_server_ip: Ipv4Addr,
    matchmaking_server_port: u16,
    max_players: u32,
    game_server_index: u16,
}


impl GameServerConfig {
    fn new(args: Vec<String>) -> Result<Self, &'static str> {
        if args.len() != 4 {
            return Err(
                "Usage: game_server <signaling_port> <matchmaking_server_ip> <matchmaking_server_port>"
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

        Ok(GameServerConfig {
            signaling_port,
            matchmaking_server_ip,
            matchmaking_server_port,
            max_players,
            game_server_index: signaling_port
        })
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

    let runtime = Arc::new(
        Builder::new_current_thread()
            .enable_all()
            .worker_threads(1)
            .build()
            .unwrap()
    );

    runtime.spawn(async_main(runtime.clone(), config));
}


async fn async_main(
    runtime: Arc<Runtime>,
    config: GameServerConfig
) {
    
    let (sender_to_matchmaking_server, reciever) =
        channel::<GameServerMatchmakingServerProtocol>(10);

    let handle_to_matchmaking_server_connect =
        runtime.spawn(connect_to_matchmaking_server(
            config.matchmaking_server_ip.to_string(),
            config.matchmaking_server_port,
            reciever
        ));

    runtime.spawn(run_singnaling_server(
        config.signaling_port,
        config.max_players,
        config.clone(),
        sender_to_matchmaking_server.clone()
    ));

    let (mut webrtc_socket, socket_future) =
        matchbox_socket::WebRtcSocketBuilder::new(
            format!("ws://localhost:{}/", config.signaling_port)
        )
        .ice_server(RtcIceServerConfig::default())
        .add_reliable_channel()
        .add_unreliable_channel()
        .build();

    runtime.spawn(socket_future);

    let instant = std::time::Instant::now();
    while webrtc_socket.id().is_none() {
        std::thread::sleep(Duration::from_millis(10));

        if instant.elapsed().as_millis() > 3000 {

            println!("fail to connect to signaling server");
        }
    }

    println!("ready");

    sender_to_matchmaking_server.blocking_send(
        GameServerMatchmakingServerProtocol::GameServerMessage(
            GameServerMessage::ServerHasStarted(config.game_server_index)
        )
    ).unwrap();

    game_server_main_loop(
        webrtc_socket,
        sender_to_matchmaking_server,
        config,
        handle_to_matchmaking_server_connect,
    ).await;
}

async fn game_server_main_loop(
    mut webrtc_socket: WebRtcSocket<MultipleChannels>,
    sender_to_matchmaking_server: Sender<GameServerMatchmakingServerProtocol>,
    config: GameServerConfig,
    handle_to_matchmaking_server_connect: JoinHandle<()>,
) {
    let mut idle_timer: Option<Instant> = None;

    let mut players_state: Vec<PeerId> = Vec::with_capacity(config.max_players as usize);

    let mut relaible_channel = webrtc_socket
        .take_channel(0)
        .unwrap();

    let mut unrelaible_channel = webrtc_socket
        .take_channel(1)
        .unwrap();

    loop {

        // shutdown the game server if no players on the server for more than 3 minutes
        if webrtc_socket.connected_peers().count() == 0 {
            if idle_timer.is_some() {
                if idle_timer.unwrap().elapsed().as_secs() > 180 {
                    
                    shutdown_game_server(
                        sender_to_matchmaking_server,
                        config,
                        handle_to_matchmaking_server_connect,
                    ).await;
                }
            } else {
                idle_timer = Some(Instant::now());
            }
        } else {
            idle_timer = None;
        }

        let updated_players = webrtc_socket.update_peers();

        for (player_id, player_state) in updated_players {
            match player_state {
                Connected => {
                    handle_player_connection(
                        &mut players_state,
                        player_id
                    )
                }
                Disconnected => {
                    handle_player_disconnection(
                        &mut players_state,
                        player_id
                    )
                }
            }
        }

        let recieved_messages = unrelaible_channel.receive();

        for (from_player, packet) in recieved_messages {
            
            proccess_unrelaible_message(
                &mut unrelaible_channel,
                &players_state,
                from_player,
                packet
            );
        }

        let recieved_messages = relaible_channel.receive();
        
        for (from_player, packet) in recieved_messages {
            
            proccess_relaible_message(
                &mut relaible_channel,
                &players_state,
                from_player,
                packet
            );
        }

        tokio::time::sleep(Duration::from_millis(1)).await;
    }
}


fn handle_player_connection(
    players_state: &mut Vec<PeerId>,
    player_id: PeerId,
) {

}


fn handle_player_disconnection(
    players_state: &mut Vec<PeerId>,
    player_id: PeerId
) {
    
}


fn proccess_relaible_message(
    channel: &mut WebRtcChannel,
    players_state: &Vec<PeerId>,
    from_player: PeerId,
    packet: Box<[u8]>,
) {
    if let Some(message) = ClientMessage::from_packet(packet) {
        match message {
            ClientMessage::DirectMessage(player_id, message) => {

            }
            ClientMessage::BoardcastMessage(message) => {
                
            }
        }
    }
}


fn proccess_unrelaible_message(
    channel: &mut WebRtcChannel,
    players_state: &Vec<PeerId>,
    from_player: PeerId,
    packet: Box<[u8]>,
) {
    
}


async fn shutdown_game_server(
    sender_to_matchmaking_server: Sender<GameServerMatchmakingServerProtocol>,
    config: GameServerConfig,
    handle_to_matchmaking_server_connect: JoinHandle<()>,
) -> ! 
{
    sender_to_matchmaking_server.blocking_send(
        GameServerMatchmakingServerProtocol::GameServerMessage(
            GameServerMessage::GameServerHasShutDown(
                config.game_server_index
            )
        )
    ).unwrap();

    let timer = Instant::now();

    while !handle_to_matchmaking_server_connect.is_finished() {
        if timer.elapsed().as_secs() > 3 {
            exit(1)
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    exit(0)
}


async fn run_singnaling_server(
    port: u16,
    max_players: u32,
    config: GameServerConfig,
    sender_to_matchmaking_server: Sender<GameServerMatchmakingServerProtocol>
) {
    let players_amount = Arc::new(Mutex::new(0u32));

    let players_amount_1 = players_amount.clone();
    let players_amount_2 = players_amount.clone();

    let sender_to_matchmaking_server_1 = sender_to_matchmaking_server.clone();
    let sender_to_matchmaking_server_2 = sender_to_matchmaking_server.clone();

    let server = 
        SignalingServer::client_server_builder(
            SocketAddr::V4(
                SocketAddrV4::new(Ipv4Addr::LOCALHOST, port)
            )
        )

        .on_connection_request(move |_connection| {

            if *players_amount.lock().unwrap() >= max_players {
                Ok(false)
            } else {
                Ok(true)
            }
        })

        .on_client_connected(move |_id| {
            *players_amount_1.lock().unwrap() += 1;

            sender_to_matchmaking_server_1.blocking_send(
                GameServerMatchmakingServerProtocol::GameServerMessage(
                    GameServerMessage::PlayerConnected(config.game_server_index)
                )
            ).unwrap();
        })

        .on_client_disconnected(move |_id| {
            *players_amount_2.lock().unwrap() -= 1;

            sender_to_matchmaking_server_2.blocking_send(
                GameServerMatchmakingServerProtocol::GameServerMessage(
                    GameServerMessage::PlayerDisconnected(config.game_server_index)
                )
            ).unwrap();
        })

        // .on_id_assignment(|(_socket, _id)| {})
        // .on_host_connected(|_id| {})
        // .on_host_disconnected(|_id| {})
        // .cors()
        // .trace()

        .build();

    server.serve().await.unwrap()
}


async fn connect_to_matchmaking_server(
    matchmaking_server_ip: String,
    matchmaking_server_port: u16,
    mut reciever: Receiver<GameServerMatchmakingServerProtocol>
) {
    
    let server_url = format!("ws://{}:{}/", matchmaking_server_ip, matchmaking_server_port);
    
    let (mut ws_stream, _) =
        connect_async(server_url)
        .await
        .expect("Failed to connect to matchmaking server");

    while let Some(message) = reciever.recv().await {

        let mut shutdown = false;

        match &message {
            GameServerMatchmakingServerProtocol::GameServerMessage(
                message
            ) => {
                match message {
                    GameServerMessage::GameServerHasShutDown(index) => {
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
    }
}