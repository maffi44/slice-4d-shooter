use alkahest::{alkahest, Serialize};

type Packet = Box<[u8]>;
type ActorID = u128;

type SerializableTransform = ([f32; 4], [f32; 16], [f32; 4]);

#[repr(C)]
#[alkahest(Formula, Serialize, Deserialize)]
pub enum ClientMessage {
    DirectMessage(u128, NetMessageToPlayer),
    BoardcastMessage(NetMessageToPlayer),
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

#[repr(C)]
#[alkahest(Formula, Serialize, Deserialize)]
pub enum ServerMessage {
    JoinTheMatch(
        // time_in_millis_from_game_session_init
        u128,
        // which team has this player added to
        Team,
        // status of the Red Flag
        FlagStatus,
        // status of the Blue Flag
        FlagStatus,
        // status of the Move W Bonus
        BonusSpotStatus,
        // score of Red team
        u32,
        // score of Blue team
        u32
    ),

    // u128 - time_in_millis_from_game_session_init
    // Team - Which team has this player added to
    NewSessionStarted(u128, Team),
    
    // u128 - id of connected player
    PlayerConnected(u128),
    
    // u128 - id of disconnected player
    PlayerDisconnected(u128),
    
    // u128 - id of message sender
    // NetMessageToPlayer - message
    NetMessageToPlayer(u128, NetMessageToPlayer),
    
    // NetMessageToPlayer - message
    NetMessageToServer(NetMessageToPlayer),
}

impl ServerMessage {
    pub fn to_packet(self) -> Packet {
        
        let size = <ServerMessage as Serialize<ServerMessage>>::size_hint(&self).unwrap();
        
        let mut packet: Vec<u8> = Vec::with_capacity(size.heap);

        alkahest::serialize_to_vec::<ServerMessage, ServerMessage>(self, &mut packet);

        packet.into_boxed_slice()
    }

    pub fn from_packet(packet: Packet) -> Option<Self> {
        if let Ok(message) = alkahest::deserialize::<ServerMessage, ServerMessage>(&packet) {
            Some(message)
        } else {
            None
        }
    }
}

#[repr(C)]
#[alkahest(Formula, Serialize, Deserialize)]
#[derive(Clone)]
pub enum NetMessageToServer {
    None
}

#[repr(C)]
#[alkahest(Formula, Serialize, Deserialize)]
#[derive(Clone)]
pub enum NetMessageToPlayer {
    RemoteCommand(RemoteCommand),
    RemoteDirectMessage(ActorID, RemoteMessage),
    RemoteBoardCastMessage(RemoteMessage),
}

#[repr(C)]
#[alkahest(Formula, Serialize, Deserialize)]
#[derive(Clone)]
pub enum RemoteCommand {
    // transform, radius, is_alive status
    SpawnPlayersDollActor(SerializableTransform, f32, bool),
    SpawnPlayerDeathExplode([f32;4]),
    RemoveActor(ActorID),
}

#[repr(C)]
#[alkahest(Formula, Serialize, Deserialize)]
#[derive(Clone)]
pub enum RemoteMessage {
    DealDamageAndAddForce(u32, [f32;4], [f32;4]),
    DieImmediately,
    DieSlowly,
    PlayerRespawn(SerializableTransform, (bool,bool,bool,bool,bool,u32), [f32;4]),
    Enable(bool),
    SetTransform(SerializableTransform),
    SetPlayerDollState(SerializableTransform, (bool,bool,bool,bool,bool,u32), [f32;4], u128),
    SpawnHoleGunShotActor([f32;4], [f32;4], f32, [f32;3], f32),
    SpawHoleGunMissActor([f32;4], [f32;4], f32, [f32;3], f32),
    HoleGunStartCharging,
    SpawnMachineGunShot([f32;4], bool),

    // Team - Which team's flag status has changed
    // FlagStatus - status of the flag
    SetFlagStatus(Team, FlagStatus),
    
    // u32 - index of concrete move W bonus spot
    // BonusSpotStatus - bonus status
    SetMoveWBonusStatus(u32, BonusSpotStatus),

    // u32 - score of Red team
    // u32 - score of Blue team
    UpdateTeamsScore(u32, u32)
}

impl NetMessageToPlayer {
    pub fn to_packet(self) -> Packet {
        
        let size = <NetMessageToPlayer as Serialize<NetMessageToPlayer>>::size_hint(&self).unwrap();
        
        let mut packet: Vec<u8> = Vec::with_capacity(size.heap);

        alkahest::serialize_to_vec::<NetMessageToPlayer, NetMessageToPlayer>(self, &mut packet);

        packet.into_boxed_slice()
    }

    pub fn from_packet(packet: Packet) -> Option<Self> {
        if let Ok(message) = alkahest::deserialize::<NetMessageToPlayer, NetMessageToPlayer>(&packet) {
            Some(message)
        } else {
            None
        }
    }
}

#[repr(C)]
#[alkahest(Formula, Serialize, Deserialize)]
#[derive(Clone, Copy)]
pub enum FlagStatus
{
    OnTheBase,
    Captured(u128),
    Missed([f32;4]),
}

#[repr(C)]
#[alkahest(Formula, Serialize, Deserialize)]
#[derive(Clone, Copy)]
pub enum BonusSpotStatus
{
    BonusOnTheSpot,
    BonusCollected,
}

#[repr(C)]
#[alkahest(Formula, Serialize, Deserialize)]
#[derive(Clone, Copy)]
pub enum Team
{
    Red,
    Blue,
}

pub enum NetCommand {
    NetSystemIsConnectedAndGetNewPeerID(u128),
    PeerConnected(u128),
    PeerDisconnected(u128),
    ConnectedToGameServer(u128),

    SendDirectNetMessageReliable(NetMessageToPlayer, u128),
    SendDirectNetMessageUnreliable(NetMessageToPlayer, u128),
    SendBoardcastNetMessageReliable(NetMessageToPlayer),
    SendBoardcastNetMessageUnreliable(NetMessageToPlayer),
}
