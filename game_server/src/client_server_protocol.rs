use alkahest::{alkahest, Serialize};

type Packet = Box<[u8]>;
type ActorID = u128;

type SerializableTransform = ([f32; 4], [f32; 16], [f32; 4]);

#[repr(C)]
#[alkahest(Formula, Serialize, Deserialize)]
pub enum ClientMessage {
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

#[repr(C)]
#[alkahest(Formula, Serialize, Deserialize)]
pub enum ServerMessage {
    PlayerConnected(u128),
    PlayerDisconnected(u128),
    NetMessage(u128, NetMessage),
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
pub enum NetMessage {
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
    PlayerRespawn([f32;4]),
    Enable(bool),
    SetTransform(SerializableTransform),
    SpawnHoleGunShotActor([f32;4], [f32;4], f32, [f32;3], f32),
    SpawHoleGunMissActor([f32;4], [f32;4], f32, [f32;3], f32),
    HoleGunStartCharging,
    SpawnMachineGunShot([f32;4], bool)
}

impl NetMessage {
    pub fn to_packet(self) -> Packet {
        
        let size = <NetMessage as Serialize<NetMessage>>::size_hint(&self).unwrap();
        
        let mut packet: Vec<u8> = Vec::with_capacity(size.heap);

        alkahest::serialize_to_vec::<NetMessage, NetMessage>(self, &mut packet);

        packet.into_boxed_slice()
    }

    pub fn from_packet(packet: Packet) -> Option<Self> {
        if let Ok(message) = alkahest::deserialize::<NetMessage, NetMessage>(&packet) {
            Some(message)
        } else {
            None
        }
    }
}


pub enum NetCommand {
    NetSystemIsConnectedAndGetNewPeerID(u128),
    PeerConnected(u128),
    PeerDisconnected(u128),

    SendDirectNetMessageReliable(NetMessage, u128),
    SendDirectNetMessageUnreliable(NetMessage, u128),
    SendBoardcastNetMessageReliable(NetMessage),
    SendBoardcastNetMessageUnreliable(NetMessage),
}
