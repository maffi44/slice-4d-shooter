pub mod player;

use player::Player;

use super::{
    transform::Transform,
    engine_handle::{
        EngineHandle,
        Command,
    },
    physics::collider::{
        Collider,
        MutCollider,
    },
};


pub type ActorID = u64;

pub enum ActorWrapper {
    Player(Player),
    Diamond,
    Exit,
}

pub trait Actor<'a> {

    fn recieve_message(&mut self, message: Message, engine_handle: &mut EngineHandle);

    fn recieve_boardcast_message(&mut self, message: &Message, engine_handle: &mut EngineHandle);

    fn tick(&mut self, engine_handle: &mut EngineHandle) {}

    fn get_collider(&'a self) -> Option<Collider<'a>> {None}

    fn get_mut_collider(&'a mut self) -> Option<MutCollider<'a>> {None}

    fn get_visual_elem(&self) {}

    fn get_id(&self) -> Option<ActorID>;
    
    fn set_id(&mut self, id: ActorID);
}

impl Actor<'_> for ActorWrapper {
    fn recieve_message(&mut self, message: Message, engine_handle: &mut EngineHandle) {
        match  self {
            ActorWrapper::Player(player) => {
                player.recieve_message(message, engine_handle);
            },
            ActorWrapper::Diamond => {panic!("try to get access to diamond")},
            ActorWrapper::Exit => {panic!("try to get access to exit")},
        }
    }

    fn recieve_boardcast_message(&mut self, message: &Message, engine_handle: &mut EngineHandle) {
        match  self {
            ActorWrapper::Player(player) => {
                player.recieve_boardcast_message(message, engine_handle);
            },
            ActorWrapper::Diamond => {panic!("try to get access to diamond")},
            ActorWrapper::Exit => {panic!("try to get access to exit")},
        }
    }

    fn tick(&mut self, engine_handle: &mut EngineHandle) {
        match  self {
            ActorWrapper::Player(player) => {
                player.tick(engine_handle);
            },
            ActorWrapper::Diamond => {panic!("try to get access to diamond")},
            ActorWrapper::Exit => {panic!("try to get access to exit")},
        }
    }

    fn get_collider(&'_ self) -> Option<Collider<'_>> {
        match  self {
            ActorWrapper::Player(player) => {
                player.get_collider()
            },
            ActorWrapper::Diamond => {panic!("try to get access to diamond")},
            ActorWrapper::Exit => {panic!("try to get access to exit")},
        }
    }

    fn get_mut_collider(&'_ mut self) -> Option<MutCollider<'_>> {
        match  self {
            ActorWrapper::Player(player) => {
                player.get_mut_collider()
            },
            ActorWrapper::Diamond => {panic!("try to get access to diamond")},
            ActorWrapper::Exit => {panic!("try to get access to exit")},
        }
    }

    fn get_visual_elem(&self) {
        match  self {
            ActorWrapper::Player(player) => {
                player.get_visual_elem()
            },
            ActorWrapper::Diamond => {panic!("try to get access to diamond")},
            ActorWrapper::Exit => {panic!("try to get access to exit")},
        }
    }

    fn get_id(&self) -> Option<ActorID> {
        match  self {
            ActorWrapper::Player(player) => {
                player.get_id()
            },
            ActorWrapper::Diamond => {panic!("try to get access to diamond")},
            ActorWrapper::Exit => {panic!("try to get access to exit")},
        }
    }

    fn set_id(&mut self, id: ActorID) {
        match  self {
            ActorWrapper::Player(player) => {
                player.set_id(id);
            },
            ActorWrapper::Diamond => {panic!("try to get access to diamond")},
            ActorWrapper::Exit => {panic!("try to get access to exit")},
        }
    }
}


pub struct Message {
    pub from: ActorID,
    pub message: MessageType,
}
pub enum MessageType {
    DealDamage(u32),
    SetTransform(Transform),
    EnableCollider(bool)
}