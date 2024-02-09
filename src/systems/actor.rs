pub mod player;
pub mod diamond;

use player::Player;

use super::{
    transform::Transform,
    engine_handle::EngineHandle,
    physics::collider::{
        DynamicCollider,
        StaticCollider,
        Area
    },
};


pub type ActorID = u64;

pub trait Actor<'a> {

    fn recieve_message(&mut self, message: Message, engine_handle: &mut EngineHandle);

    fn recieve_boardcast_message(&mut self, message: &Message, engine_handle: &mut EngineHandle);

    fn tick(&mut self, engine_handle: &mut EngineHandle) {}

    fn get_dynamic_collider(&'a mut self) -> Option<&'a mut DynamicCollider> {None}

    fn get_static_colliders(&'a mut self) -> Option<&'a mut Vec<StaticCollider>> {None}
    
    fn get_areas(&'a mut self) -> Option<&'a mut Vec<Area>> {None}

    fn get_visual_elem(&'a self) {}

    fn get_id(&self) -> Option<ActorID>;
    
    fn set_id(&mut self, id: ActorID);
}

pub enum ActorWrapper {
    Player(Player),
    Diamond,
    Exit,
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

    fn get_dynamic_collider(&mut self) -> Option<&mut DynamicCollider> {
        match  self {
            ActorWrapper::Player(player) => {
                player.get_dynamic_collider()
            },
            ActorWrapper::Diamond => {panic!("try to get access to diamond")},
            ActorWrapper::Exit => {panic!("try to get access to exit")},
        }
    }

    fn get_static_colliders(&mut self) -> Option<&mut Vec<StaticCollider>> {
        match  self {
            ActorWrapper::Player(player) => {
                player.get_static_colliders()
            },
            ActorWrapper::Diamond => {panic!("try to get access to diamond")},
            ActorWrapper::Exit => {panic!("try to get access to exit")},
        }
    }

    fn get_areas(&mut self) -> Option<&mut Vec<Area>> {
        match  self {
            ActorWrapper::Player(player) => {
                player.get_areas()
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