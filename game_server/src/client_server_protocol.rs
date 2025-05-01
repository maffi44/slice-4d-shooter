use alkahest::{alkahest, Serialize};

type Packet = Box<[u8]>;
type ActorID = u128;

type SerializableTransform = ([f32; 4], [f32; 16]);

#[repr(C)]
#[alkahest(Formula, Serialize, Deserialize)]
pub enum ClientMessage {
    DirectMessageToPlayer(u128, NetMessageToPlayer),
    BoardcastMessageToPlayers(NetMessageToPlayer),
    MessageToServer(NetMessageToServer)
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
    TryToGetOpponentsFlag(
        // time of attempt
        u128,
    ),
    TryToGetScore(
        // time of attempt
        u128,
    ),
    TryToReturnMyFlag(
        // time of attempt
        u128,
    ),
    TryToGetMoveWBonus(
        // time of attempt
        u128,
        // index of bonus spot
        u32,
    ),
    DropedFlag(
        // which team owns the dropped flag
        Team,
        // position of flag
        [f32;4],
        // droped in space
        bool
    ),
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
    SpawnPlayersDollActor(
        // position
        SerializableTransform,
        // radius
        f32,
        // is_alive
        bool,
        //team
        Team
    ),
    SpawnPlayerDeathExplode([f32;4]),
    RemoveActor(ActorID),
    SpawnHole(
        // position
        [f32;4],
        //radius
        f32,
        // color
        [f32;3],
        // target size is reached
        bool,
        // target radius
        f32,
        // explode current time
        f32,
        // explode final time
        f32,
    ),
}

#[repr(C)]
#[alkahest(Formula, Serialize, Deserialize)]
#[derive(Clone)]
pub enum RemoteMessage {
    DealDamageAndAddForce(
        // damage
        u32,
        // force
        [f32;4],
        // impact position
        [f32;4],
        // damage dealer's team
        Team
    ),
    DieImmediately,
    DieSlowly,
    PlayerRespawn(
        // position
        SerializableTransform,
        // input state for extrapolation
        (bool,bool,bool,bool,bool),
        // force for physic body
        [f32;4],
        // player's team
        Team
    ),
    Enable(bool),
    SetTransform(SerializableTransform),
    SetPlayerDollState(
        // Player's transform
        SerializableTransform,
        // simple input state (for extrapolation)
        (bool,bool,bool,bool,bool),
        // player's velocity
        [f32;4],
        // frame's server time
        u128
    ),
    SpawnHoleGunShotActor(
        // shot's impact position
        [f32;4],
        // shot's source position
        [f32;4],
        // hole radius
        f32,
        // hole color
        [f32;3],
        // shot's flash radius
        f32
    ),
    SpawHoleGunMissActor(
        // shot's impact position
        [f32;4],
        // shot's source position
        [f32;4],
        // hole radius
        f32,
        // hole color
        [f32;3],
        // shot's flash radius
        f32
    ),

    HoleGunStartCharging,
    SpawnMachineGunShot(
        // shot's impact position
        [f32;4],
        // it is miss
        bool
    ),
    SpawnShotgunShot(
        // shot's start position
        [f32;4],
        // shot's main direction
        [f32;4],
        // random seed
        u64,
        //damage dealer's id
        u128,
        //damage dealer's team
        Team,
    ),

    SetFlagStatus(
        // Which team's flag status has changed
        Team,
        // status of the flag
        FlagStatus
    ),
    
    SetMoveWBonusStatus(
        // index of concrete move W bonus spot        
        u32,
        // bonus status
        BonusSpotStatus
    ),

    UpdateTeamsScore(
        // Red team score
        u32,
        // Blue team score
        u32
    ),

    SetNewTeam(
        Team
    ),

    TeamWin(
        Team
    ),
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
    Droped([f32;4]),
}

#[repr(C)]
#[alkahest(Formula, Serialize, Deserialize)]
#[derive(Clone, Copy)]
pub enum BonusSpotStatus
{
    BonusOnTheSpot,
    BonusCollected(
        // player's id who collected the bonus
        u128
    ),
}

#[repr(C)]
#[alkahest(Formula, Serialize, Deserialize)]
#[derive(Clone, Copy)]
pub enum Team
{
    Red,
    Blue,
}

impl PartialEq for Team
{
    fn eq(&self, other: &Self) -> bool {
        match self {
            Team::Red =>
            {
                match other {
                    Team::Red =>
                    {
                        true
                    }
                    Team::Blue =>
                    {
                        false
                    }
                }
            }
            Team::Blue =>
            {
                match other {
                    Team::Red =>
                    {
                        false
                    }
                    Team::Blue =>
                    {
                        true
                    }
                }
            }
        }
    }
}

pub enum NetCommand {
    NetSystemIsConnectedAndGetNewPeerID(u128),
    PeerConnected(u128),
    PeerDisconnected(u128),
    SetServerTime(u128),

    SendMessageToServer(NetMessageToServer),

    SendDirectNetMessageReliable(NetMessageToPlayer, u128),
    SendDirectNetMessageUnreliable(NetMessageToPlayer, u128),
    SendBoardcastNetMessageReliable(NetMessageToPlayer),
    SendBoardcastNetMessageUnreliable(NetMessageToPlayer),
}
