use alkahest::{alkahest, Serialize};
use super::NetMessage;

type Packet = Box<[u8]>;

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
