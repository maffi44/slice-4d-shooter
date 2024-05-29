pub mod player;
pub mod diamond;
pub mod wandering_actor;
pub mod device;
pub mod holegun_shot;
pub mod holegun_miss;
pub mod players_doll;
pub mod players_death_explosion;
pub mod machinegun_shot;
pub mod shooting_impact;

use crate::{
    engine::{
        audio::AudioSystem, engine_handle::EngineHandle, physics::{
            area::AreaMessages, colliders_container::PhysicalElement, dynamic_collider::DynamicColliderMessages, kinematic_collider::KinematicColliderMessages, static_collider::StaticColliderMessages, PhysicsSystem
        }, render::VisualElement
    },
    transform::Transform,
};

use self::{
    holegun_miss::HoleGunMiss, holegun_shot::HoleGunShot, machinegun_shot::MachinegunShot, player::{
        Player, PlayerMessages
    }, players_death_explosion::PlayersDeathExplosion, players_doll::{PlayersDoll, PlayersDollMessages}, shooting_impact::ShootingImpact, wandering_actor::WanderingActor
};


pub type ActorID = u128;

pub trait Actor {

    fn recieve_message(
        &mut self,
        message: &Message,
        engine_handle: &mut EngineHandle,
        physics_system: &PhysicsSystem,
        audio_system: &mut AudioSystem
    ) {}

    fn get_mut_transform(&mut self) -> &mut Transform;
    
    fn get_transform(&self) -> &Transform;

    fn tick(
        &mut self,
        physic_system: &PhysicsSystem,
        engine_handle: &mut EngineHandle,
        audio_system: &mut AudioSystem,
        delta: f32
    ) {}

    fn get_physical_element(&mut self) -> Option<PhysicalElement> {None}

    fn get_visual_element(&self) -> Option<VisualElement> {None}

    fn get_id(&self) -> Option<ActorID>;

    fn set_id(&mut self, id: ActorID, engine_handle: &mut EngineHandle);
    
    fn init(&mut self, id: ActorID);
}

pub enum ActorWrapper {
    Player(Player),
    WonderingActor(WanderingActor),
    HoleGunShot(HoleGunShot),
    HoleGunMiss(HoleGunMiss),
    PlayersDoll(PlayersDoll),
    PlayersDeathExplosion(PlayersDeathExplosion),
    MachinegunShot(MachinegunShot),
    ShootingImpact(ShootingImpact),
    Diamond,
    Exit,
}

impl Actor for ActorWrapper {

    fn get_transform(&self) -> &Transform {
        match self {
            ActorWrapper::Player(actor) => {
                actor.get_transform()
            },
            ActorWrapper::WonderingActor(actor) => {
                actor.get_transform()
            }
            ActorWrapper::HoleGunShot(actor) => {
                actor.get_transform()
            }
            ActorWrapper::HoleGunMiss(actor) => {
                actor.get_transform()
            }
            ActorWrapper::PlayersDoll(actor) => {
                actor.get_transform()
            }
            ActorWrapper::PlayersDeathExplosion(actor) => {
                actor.get_transform()
            }
            ActorWrapper::MachinegunShot(actor) => {
                actor.get_transform()
            }
            ActorWrapper::ShootingImpact(actor) => {
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
            ActorWrapper::HoleGunShot(actor) => {
                actor.get_mut_transform()
            },
            ActorWrapper::HoleGunMiss(actor) => {
                actor.get_mut_transform()
            },
            ActorWrapper::PlayersDoll(actor) => {
                actor.get_mut_transform()
            },
            ActorWrapper::PlayersDeathExplosion(actor) => {
                actor.get_mut_transform()
            },
            ActorWrapper::MachinegunShot(actor) => {
                actor.get_mut_transform()
            },
            ActorWrapper::ShootingImpact(actor) => {
                actor.get_mut_transform()
            },
            ActorWrapper::Diamond => {unreachable!("try to get access to diamond")},
            ActorWrapper::Exit => {unreachable!("try to get access to exit")},
        }
    }

    fn recieve_message(
        &mut self,
        message: &Message,
        engine_handle: &mut EngineHandle,
        physics_system: &PhysicsSystem,
        audio_system: &mut AudioSystem
    ) {
        match  self {
            ActorWrapper::Player(actor) => {
                actor.recieve_message(message, engine_handle, physics_system, audio_system);
            },
            ActorWrapper::WonderingActor(actor) => {
                actor.recieve_message(message, engine_handle, physics_system, audio_system);
            },
            ActorWrapper::HoleGunShot(actor) => {
                actor.recieve_message(message, engine_handle, physics_system, audio_system);
            },
            ActorWrapper::HoleGunMiss(actor) => {
                actor.recieve_message(message, engine_handle, physics_system, audio_system);
            },
            ActorWrapper::PlayersDoll(actor) => {
                actor.recieve_message(message, engine_handle, physics_system, audio_system);
            },
            ActorWrapper::PlayersDeathExplosion(actor) => {
                actor.recieve_message(message, engine_handle, physics_system, audio_system);
            },
            ActorWrapper::MachinegunShot(actor) => {
                actor.recieve_message(message, engine_handle, physics_system, audio_system);
            },
            ActorWrapper::ShootingImpact(actor) => {
                actor.recieve_message(message, engine_handle, physics_system, audio_system);
            },
            ActorWrapper::Diamond => {unreachable!("try to get access to diamond")},
            ActorWrapper::Exit => {unreachable!("try to get access to exit")},
        }
    }

    fn tick(
        &mut self,
        physic_system: &PhysicsSystem,
        engine_handle: &mut EngineHandle,
        audio_system: &mut AudioSystem,
        delta: f32
    ) {
        match  self {
            ActorWrapper::Player(actor) => {
                actor.tick(physic_system, engine_handle, audio_system, delta);
            },
            ActorWrapper::WonderingActor(actor) => {
                actor.tick(physic_system, engine_handle, audio_system, delta);
            },
            ActorWrapper::HoleGunShot(actor) => {
                actor.tick(physic_system, engine_handle, audio_system, delta);
            },
            ActorWrapper::HoleGunMiss(actor) => {
                actor.tick(physic_system, engine_handle, audio_system, delta);
            },
            ActorWrapper::PlayersDoll(actor) => {
                actor.tick(physic_system, engine_handle, audio_system, delta);
            },
            ActorWrapper::PlayersDeathExplosion(actor) => {
                actor.tick(physic_system, engine_handle, audio_system, delta);
            },
            ActorWrapper::MachinegunShot(actor) => {
                actor.tick(physic_system, engine_handle, audio_system, delta);
            },
            ActorWrapper::ShootingImpact(actor) => {
                actor.tick(physic_system, engine_handle, audio_system, delta);
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
            ActorWrapper::HoleGunShot(actor) => {
                actor.get_physical_element()
            },
            ActorWrapper::HoleGunMiss(actor) => {
                actor.get_physical_element()
            },
            ActorWrapper::PlayersDoll(actor) => {
                actor.get_physical_element()
            },
            ActorWrapper::PlayersDeathExplosion(actor) => {
                actor.get_physical_element()
            },
            ActorWrapper::MachinegunShot(actor) => {
                actor.get_physical_element()
            },
            ActorWrapper::ShootingImpact(actor) => {
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
            ActorWrapper::HoleGunShot(actor) => {
                actor.get_visual_element()
            },
            ActorWrapper::HoleGunMiss(actor) => {
                actor.get_visual_element()
            },
            ActorWrapper::PlayersDoll(actor) => {
                actor.get_visual_element()
            },
            ActorWrapper::PlayersDeathExplosion(actor) => {
                actor.get_visual_element()
            },
            ActorWrapper::MachinegunShot(actor) => {
                actor.get_visual_element()
            },
            ActorWrapper::ShootingImpact(actor) => {
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
            ActorWrapper::HoleGunShot(actor) => {
                actor.get_id()
            },
            ActorWrapper::HoleGunMiss(actor) => {
                actor.get_id()
            },
            ActorWrapper::PlayersDoll(actor) => {
                actor.get_id()
            },
            ActorWrapper::PlayersDeathExplosion(actor) => {
                actor.get_id()
            },
            ActorWrapper::MachinegunShot(actor) => {
                actor.get_id()
            },
            ActorWrapper::ShootingImpact(actor) => {
                actor.get_id()
            },
            ActorWrapper::Diamond => {unreachable!("try to get access to diamond")},
            ActorWrapper::Exit => {unreachable!("try to get access to exit")},
        }
    }

    fn set_id(&mut self, id: ActorID, engine_handle: &mut EngineHandle) {
        match self {
            ActorWrapper::Player(actor) => {
                actor.set_id(id, engine_handle);
            },
            ActorWrapper::WonderingActor(actor) => {
                actor.set_id(id, engine_handle);
            },
            ActorWrapper::HoleGunShot(actor) => {
                actor.set_id(id, engine_handle);
            },
            ActorWrapper::HoleGunMiss(actor) => {
                actor.set_id(id, engine_handle);
            },
            ActorWrapper::PlayersDoll(actor) => {
                actor.set_id(id, engine_handle);
            },
            ActorWrapper::PlayersDeathExplosion(actor) => {
                actor.set_id(id, engine_handle);
            },
            ActorWrapper::MachinegunShot(actor) => {
                actor.set_id(id, engine_handle);
            },
            ActorWrapper::ShootingImpact(actor) => {
                actor.set_id(id, engine_handle);
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
            ActorWrapper::HoleGunShot(actor) => {
                actor.init(id);
            },
            ActorWrapper::HoleGunMiss(actor) => {
                actor.init(id);
            },
            ActorWrapper::PlayersDoll(actor) => {
                actor.init(id);
            },
            ActorWrapper::PlayersDeathExplosion(actor) => {
                actor.init(id);
            },
            ActorWrapper::MachinegunShot(actor) => {
                actor.init(id);
            },
            ActorWrapper::ShootingImpact(actor) => {
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
    Enable(bool),
    IncrementPosition(Vec4),
    IWasChangedMyId(ActorID),
}

pub enum SpecificActorMessage {
    PLayerMessages(PlayerMessages),
    PlayersDollMessages(PlayersDollMessages),
}

pub enum PhysicsMessages {
    KinematicColliderMessages(KinematicColliderMessages),
    StaticColliderMessages(StaticColliderMessages),
    DynamicColliderMessages(DynamicColliderMessages),
    AreaMessages(AreaMessages),
}

