pub mod player;
pub mod diamond;
pub mod wandering_actor;

use player::Player;

use super::{
    engine_handle::EngineHandle, physics::{
        area::Area, colliders_container::PhysicalElement, dynamic_collider::DynamicCollider, kinematic_collider::KinematicCollider, static_collider::StaticCollider
    }, render::VisualElement, transform::Transform
};


pub type ActorID = u64;

pub trait Actor {

    fn recieve_message(&mut self, message: &Message, engine_handle: &mut EngineHandle) {}

    fn get_mut_transform(&mut self) -> &mut Transform;
    
    fn get_transform(&self) -> &Transform;

    fn tick(&mut self, engine_handle: &mut EngineHandle, delta: f32) {}

    fn get_physical_element(&mut self) -> Option<PhysicalElement> {None}

    fn get_visual_element(&self) -> Option<VisualElement> {None}

    fn get_id(&self) -> Option<ActorID>;
    
    fn init(&mut self, id: ActorID);
}

pub enum ActorWrapper {
    Player(Player),
    WonderingActor(WonderingActor),
    Diamond,
    Exit,
}

impl Actor for ActorWrapper {

    fn get_transform(&self) -> &Transform {
        match  self {
            ActorWrapper::Player(actor) => {
                actor.get_transform()
            },
            ActorWrapper::WonderingActor(actor) => {
                actor.get_transform()
            }
            ActorWrapper::Diamond => {unreachable!("try to get access to diamond")},
            ActorWrapper::Exit => {unreachable!("try to get access to exit")},
        }
    }

    fn get_mut_transform(&mut self) -> &mut Transform {
        match  self {
            ActorWrapper::Player(actor) => {
                actor.get_mut_transform()
            },
            ActorWrapper::WonderingActor(actor) => {
                actor.get_mut_transform()
            },
            ActorWrapper::Diamond => {unreachable!("try to get access to diamond")},
            ActorWrapper::Exit => {unreachable!("try to get access to exit")},
        }
    }

    fn recieve_message(&mut self, message: &Message, engine_handle: &mut EngineHandle) {
        match  self {
            ActorWrapper::Player(actor) => {
                actor.recieve_message(message, engine_handle);
            },
            ActorWrapper::WonderingActor(actor) => {
                actor.recieve_message(message, engine_handle);
            },
            ActorWrapper::Diamond => {unreachable!("try to get access to diamond")},
            ActorWrapper::Exit => {unreachable!("try to get access to exit")},
        }
    }

    fn tick(&mut self, engine_handle: &mut EngineHandle, delta: f32) {
        match  self {
            ActorWrapper::Player(actor) => {
                actor.tick(engine_handle, delta);
            },
            ActorWrapper::WonderingActor(actor) => {
                actor.tick(engine_handle, delta);
            },
            ActorWrapper::Diamond => {unreachable!("try to get access to diamond")},
            ActorWrapper::Exit => {unreachable!("try to get access to exit")},
        }
    }

    fn get_physical_element(&mut self) -> Option<PhysicalElement> {
        match  self {
            ActorWrapper::Player(actor) => {
                actor.get_physical_element()
            },
            ActorWrapper::WonderingActor(actor) => {
                actor.get_physical_element()
            },
            ActorWrapper::Diamond => {unreachable!("try to get access to diamond")},
            ActorWrapper::Exit => {unreachable!("try to get access to exit")},
        }
    }

    fn get_visual_element(&self) -> Option<VisualElement>{
        match self {
            ActorWrapper::Player(actor) => {
                actor.get_visual_element()
            },
            ActorWrapper::WonderingActor(actor) => {
                actor.get_visual_element()
            },
            ActorWrapper::Diamond => {unreachable!("try to get access to diamond")},
            ActorWrapper::Exit => {unreachable!("try to get access to exit")},
        }
    }

    fn get_id(&self) -> Option<ActorID> {
        match self {
            ActorWrapper::Player(actor) => {
                actor.get_id()
            },
            ActorWrapper::WonderingActor(actor) => {
                actor.get_id()
            },
            ActorWrapper::Diamond => {unreachable!("try to get access to diamond")},
            ActorWrapper::Exit => {unreachable!("try to get access to exit")},
        }
    }

    fn init(&mut self, id: ActorID) {
        match  self {
            ActorWrapper::Player(actor) => {
                actor.init(id);
            },
            ActorWrapper::WonderingActor(actor) => {
                actor.init(id);
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

use self::{player::PLayerMessages, wandering_actor::WonderingActor};

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

