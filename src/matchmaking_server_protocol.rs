use alkahest::{alkahest, Serialize};

#[repr(C)]
#[alkahest(Formula, Serialize, Deserialize)]
pub enum MatchmakingServerMessage
{
    GameServerAddress(([u8;4],u16)),
    NoFreeServers,
    WrongGameVersionCorrectIs((u32,u32,u32))
}

#[repr(C)]
#[alkahest(Formula, Serialize, Deserialize)]
pub enum ClientMessage
{
    RequestToConnectToGameServer((u32,u32,u32))
}

#[repr(C)]
#[alkahest(Formula, Serialize, Deserialize)]
pub enum ClientMatchmakingServerProtocol
{
    MatchmakingServerMessage(MatchmakingServerMessage),
    ClientMessage(ClientMessage)
}

#[repr(C)]
#[alkahest(Formula, Serialize, Deserialize)]
pub enum GameServerMatchmakingServerProtocol
{
    GameServerMessage(GameServerMessage),
}

impl GameServerMatchmakingServerProtocol
{
    pub fn to_packet(self) -> Vec<u8> {

        let size = <
            GameServerMatchmakingServerProtocol as
            Serialize<GameServerMatchmakingServerProtocol>
        >::size_hint(&self).unwrap();
        
        let mut packet: Vec<u8> = Vec::with_capacity(size.heap);

        alkahest::serialize_to_vec::<
            GameServerMatchmakingServerProtocol,
            GameServerMatchmakingServerProtocol
        >(self, &mut packet);

        packet
    }
}

#[repr(C)]
#[alkahest(Formula, Serialize, Deserialize)]
pub enum GameServerMessage
{
    GameServerHasShutDown(u16),
    ServerHasStarted(u16),
    PlayerConnected(u16),
    PlayerDisconnected(u16),
}


impl ClientMatchmakingServerProtocol
{
    pub fn to_packet(self) -> Vec<u8> {

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