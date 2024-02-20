pub mod player;
pub mod diamond;

use player::Player;

use super::{
    engine_handle::EngineHandle, physics::{
        area::Area, colliders_container::CollidersContainer, dynamic_collider::DynamicCollider, kinematic_collider::KinematicCollider, static_collider::StaticCollider
    }, transform::Transform
};


pub type ActorID = u64;

pub trait Actor {

    fn recieve_message(&mut self, message: &Message, engine_handle: &mut EngineHandle);

    fn tick(&mut self, engine_handle: &mut EngineHandle) {}

    fn get_colliders_container(&mut self) -> Option<CollidersContainer> {None}

    fn get_visual_elem(&self) {}

    fn get_id(&self) -> Option<ActorID>;
    
    fn init(&mut self, id: ActorID);
}

pub enum ActorWrapper {
    Player(Player),
    Diamond,
    Exit,
}

impl Actor for ActorWrapper {

    fn recieve_message(&mut self, message: &Message, engine_handle: &mut EngineHandle) {
        match  self {
            ActorWrapper::Player(player) => {
                player.recieve_message(message, engine_handle);
            },
            ActorWrapper::Diamond => {unreachable!("try to get access to diamond")},
            ActorWrapper::Exit => {unreachable!("try to get access to exit")},
        }
    }

    fn tick(&mut self, engine_handle: &mut EngineHandle) {
        match  self {
            ActorWrapper::Player(player) => {
                player.tick(engine_handle);
            },
            ActorWrapper::Diamond => {unreachable!("try to get access to diamond")},
            ActorWrapper::Exit => {unreachable!("try to get access to exit")},
        }
    }

    fn get_colliders_container(&mut self) -> Option<CollidersContainer> {
        match  self {
            ActorWrapper::Player(player) => {
                player.get_colliders_container()
            },
            ActorWrapper::Diamond => {unreachable!("try to get access to diamond")},
            ActorWrapper::Exit => {unreachable!("try to get access to exit")},
        }
    }

    fn get_visual_elem(&self) {
        match  self {
            ActorWrapper::Player(player) => {
                player.get_visual_elem()
            },
            ActorWrapper::Diamond => {unreachable!("try to get access to diamond")},
            ActorWrapper::Exit => {unreachable!("try to get access to exit")},
        }
    }

    fn get_id(&self) -> Option<ActorID> {
        match  self {
            ActorWrapper::Player(player) => {
                player.get_id()
            },
            ActorWrapper::Diamond => {unreachable!("try to get access to diamond")},
            ActorWrapper::Exit => {unreachable!("try to get access to exit")},
        }
    }

    fn init(&mut self, id: ActorID) {
        match  self {
            ActorWrapper::Player(player) => {
                player.init(id);
            },
            ActorWrapper::Diamond => {unreachable!("try to get access to diamond")},
            ActorWrapper::Exit => {unreachable!("try to get access to exit")},
        }
    }
}


pub trait Component {
    fn init(&mut self, id: ActorID);

    fn get_id(&self) -> Option<ActorID>;
}

pub struct Message {
    pub from: ActorID,
    pub message: MessageType,
}
pub enum MessageType {
    CommonActorsMessages(CommonActorsMessages),
    SpecificActorMessage(SpecificActorMessage),
    PhysicsMessages(PhysicsMessages),
}

use glam::Vec4;
pub enum CommonActorsMessages {
    SetTransform(Transform),
    EnableCollider(bool),
    IncrementPosition(Vec4),
}

use self::player::PLayerMessages;

pub enum SpecificActorMessage {
    PLayerMessages(PLayerMessages),
}

use super::physics::{
    kinematic_collider::KinematicColliderMessages,
    dynamic_collider::DynamicColliderMessages,
    static_collider::StaticColliderMessages,
    area::AreaMessages,
};

pub enum PhysicsMessages {
    KinematicColliderMessages(KinematicColliderMessages),
    StaticColliderMessages(StaticColliderMessages),
    DynamicColliderMessages(DynamicColliderMessages),
    AreaMessages(AreaMessages),
}

