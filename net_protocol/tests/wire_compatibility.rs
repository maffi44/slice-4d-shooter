// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2026  Timofei Molokov

use net_protocol::{
    gameplay::{ClientMessage, NetMessageToServer},
    matchmaking::{
        ClientMatchmakingServerProtocol, GameServerMatchmakingServerProtocol, GameServerMessage,
        MatchmakingServerMessage,
    },
};

// Captured from the server-owned Alkahest definitions in baa9800.
#[test]
fn gameplay_flag_request() {
    let packet = [
        0xea, 0x16, 0xb0, 0x4c, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0,
    ];
    let request =
        ClientMessage::MessageToServer(NetMessageToServer::TryToGetOpponentsFlag(9_876_543_210));
    assert_eq!(request.to_packet(), packet);
    assert!(matches!(
        ClientMessage::from_packet(&packet),
        Some(ClientMessage::MessageToServer(
            NetMessageToServer::TryToGetOpponentsFlag(9_876_543_210)
        ))
    ));
}

#[test]
fn matchmaking_address_reply() {
    let packet = [0x45, 0xb0, 1, 0, 0, 127, 1, 0, 0, 0, 0, 0, 0, 0];
    let address = MatchmakingServerMessage::GameServerAddress(([127, 0, 0, 1], 45125));
    let reply = ClientMatchmakingServerProtocol::MatchmakingServerMessage(address);
    assert_eq!(reply.to_packet(), packet);
    assert!(matches!(
        ClientMatchmakingServerProtocol::from_packet(&packet),
        Ok(ClientMatchmakingServerProtocol::MatchmakingServerMessage(
            MatchmakingServerMessage::GameServerAddress(([127, 0, 0, 1], 45125))
        ))
    ));
}

#[test]
fn game_server_player_count_reply() {
    let packet = [3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let reply = GameServerMatchmakingServerProtocol::GameServerMessage(
        GameServerMessage::PlayersAmoutIs(3),
    );
    assert_eq!(reply.to_packet(), packet);
    assert!(matches!(
        GameServerMatchmakingServerProtocol::from_packet(&packet),
        Ok(GameServerMatchmakingServerProtocol::GameServerMessage(
            GameServerMessage::PlayersAmoutIs(3)
        ))
    ));
}
