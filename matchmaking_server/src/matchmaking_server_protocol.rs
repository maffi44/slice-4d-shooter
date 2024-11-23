use alkahest::{alkahest, Serialize};

#[derive(Clone, Debug)]
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

impl From<&str> for GameVersion
{
    fn from(value: &str) -> Self {
        let parts: Vec<&str> = value.split('.').collect();

        GameVersion {
            major: parts[0]
                .parse::<u32>()
                .expect("ERROR: Can't parse cargo env VERSION into GameVersion"),
            minor: parts[1]
                .parse::<u32>()
                .expect("ERROR: Can't parse cargo env VERSION into GameVersion"),
            maintenance: parts[2]
                .parse::<u32>()
                .expect("ERROR: Can't parse cargo env VERSION into GameVersion"),
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
    GameServerShutedDown(u16),
    ServerStarted(u16),
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