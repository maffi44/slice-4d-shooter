// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2026  Timofei Molokov



pub mod main_player;
pub mod diamond;
pub mod wandering_actor;
pub mod device;
pub mod holegun_shot;
pub mod holegun_miss;
pub mod players_doll;
pub mod players_death_explosion;
pub mod machinegun_shot;
pub mod shooting_impact;
pub mod flag;
pub mod flag_base;
pub mod session_controller;
pub mod move_w_bonus;
pub mod hole;
pub mod wave;
pub mod mover_w;
pub mod player_for_2d_3d_example;
pub mod shotgun_shot_source;
pub mod shotgun_laser_shot;
pub mod observer;
pub mod obstaclesgun_shot;
pub mod obstacle_course_free_movement_player;
pub mod triggering_wandering_actor;
pub mod trigger;
pub mod trgger_orb;
pub mod final_trigger;
pub mod triggering_special_lift;
pub mod droped_rotator_tool;
pub mod new_spawn_area;

use std::fmt::Display;

use crate::{
    actor::{droped_rotator_tool::{DropedRotatorTool, DropedRotatorToolMessage}, final_trigger::{FinalTrgger, FinalTrggerMessage}, flag_base::{FlagBase, FlagBaseMessage}, new_spawn_area::{NewSpawnArea, NewSpawnAreaMessage}, obstacle_course_free_movement_player::ObstacleCourseFreeMovementPlayer, obstaclesgun_shot::ObstaclesGunShot, trgger_orb::{TriggerOrb, TriggerOrbMessage}, trigger::{Trigger, TriggerMessage}, triggering_special_lift::TriggeringSpecialLiftActor, triggering_wandering_actor::TriggeringWanderingActor}, engine::{
        world::physics::{
            area::AreaMessage, colliders_container::PhysicalElement, dynamic_collider::DynamicColliderMessage, kinematic_collider::KinematicColliderMessage, static_collider::StaticColliderMessage
        },
        shared_systems::render::{VisualElement, camera::Camera},
        shared_systems::EngineSharedSystems,
        world::{messenger::Messenger, WorldContext},
    }, engine::transform::Transform
};

use self::{
    holegun_miss::HoleGunMiss,
    holegun_shot::HoleGunShot,
    machinegun_shot::MachinegunShot,
    main_player::{
        MainPlayer,
        PlayerMessage
    },
    players_death_explosion::PlayersDeathExplosion,
    players_doll::{
        PlayerDoll,
        PlayersDollMessage
    },
    flag::FlagMessage,
    session_controller::SessionControllerMessage,
    shooting_impact::ShootingImpact,
    wandering_actor::WanderingActor
};


pub type ActorID = u128;

/// An actor of a World. Every callback gets the actor's own world (`WorldContext`: messenger,
/// physics, world time) and the Engine Shared Systems (input, audio, UI, net, settings, GPU, ...).
pub trait Actor {

    fn get_actor_as_controlled(&self) -> Option<&dyn ControlledActor>
    {
        None
    }

    fn get_actor_as_controlled_mut(&mut self) -> Option<&mut dyn ControlledActor>
    {
        None
    }

    fn recieve_message(
        &mut self,
        _message: Message,
        _world: &mut WorldContext,
        _shared: &mut EngineSharedSystems,
    ) {}

    #[allow(dead_code)] // game content that is not wired into the current maps/modes; kept for later
    fn get_mut_transform(&mut self) -> &mut Transform;
    
    fn get_transform(&self) -> &Transform;

    fn tick(
        &mut self,
        _world: &mut WorldContext,
        _shared: &mut EngineSharedSystems,
    ) {}

    fn get_physical_element(&mut self) -> Option<PhysicalElement<'_>> {None}

    fn get_visual_element(&self) -> Option<VisualElement<'_>> {None}

    fn get_id(&self) -> Option<ActorID>;
        
    fn set_id(&mut self, id: ActorID);

    fn change_id(&mut self, id: ActorID, messenger: &mut Messenger) {
        let prev_id = match self.get_id() {
            Some(id) =>
            {
                id
            }
            None =>
            {
                self.set_id(id);
                return;
            }
        };

        messenger.send_broadcast_message(
            Message {
                from: prev_id,
                remote_sender: false,
                message: MessageType::CommonActorsMessages(
                    CommonActorsMessage::IWasChangedMyId(id)
                )
            }
        );

        self.set_id(id);
    }

    fn on_added_to_world(
        &mut self,
        _world: &mut WorldContext,
        _shared: &mut EngineSharedSystems,
    ) {}

    /// Called once on every actor during world activation (formerly `on_level_loaded`).
    fn on_world_started(
        &mut self,
        _world: &mut WorldContext,
        _shared: &mut EngineSharedSystems,
    ) {}
}


/// Generates `enum ActorWrapper`, its `Display` and the static-dispatch `impl Actor for ActorWrapper`.
/// To add an actor type, add one line to the `actor_wrapper!` invocation below.
/// `markers` are variants without an actor value (not dispatchable).
/// Attributes before a line (e.g. `#[allow(dead_code)]`) go to that `ActorWrapper` variant.
macro_rules! actor_wrapper {
    (
        actors { $( $(#[$variant_meta:meta])* $variant:ident ( $actor_type:ty ) ),* $(,)? }
        markers { $( $(#[$marker_meta:meta])* $marker:ident ),* $(,)? }
    ) => {
        pub enum ActorWrapper {
            $( $(#[$variant_meta])* $variant($actor_type), )*
            $( $(#[$marker_meta])* $marker, )*
        }

        impl Display for ActorWrapper
        {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let actor_type = match self
                {
                    $( ActorWrapper::$variant(_) => stringify!($variant), )*
                    $( ActorWrapper::$marker => stringify!($marker), )*
                };

                write!(f, "Actor: {}", actor_type)
            }
        }

        impl Actor for ActorWrapper {

            fn get_actor_as_controlled(&self) -> Option<&dyn ControlledActor>
            {
                match self {
                    $( ActorWrapper::$variant(actor) => actor.get_actor_as_controlled(), )*
                    $( ActorWrapper::$marker => unreachable!(concat!("try to get access to ", stringify!($marker))), )*
                }
            }

            fn get_actor_as_controlled_mut(&mut self) -> Option<&mut dyn ControlledActor>
            {
                match self {
                    $( ActorWrapper::$variant(actor) => actor.get_actor_as_controlled_mut(), )*
                    $( ActorWrapper::$marker => unreachable!(concat!("try to get access to ", stringify!($marker))), )*
                }
            }

            fn recieve_message(&mut self, message: Message, world: &mut WorldContext, shared: &mut EngineSharedSystems)
            {
                match self {
                    $( ActorWrapper::$variant(actor) => actor.recieve_message(message, world, shared), )*
                    $( ActorWrapper::$marker => unreachable!(concat!("try to get access to ", stringify!($marker))), )*
                }
            }

            fn get_mut_transform(&mut self) -> &mut Transform
            {
                match self {
                    $( ActorWrapper::$variant(actor) => actor.get_mut_transform(), )*
                    $( ActorWrapper::$marker => unreachable!(concat!("try to get access to ", stringify!($marker))), )*
                }
            }

            fn get_transform(&self) -> &Transform
            {
                match self {
                    $( ActorWrapper::$variant(actor) => actor.get_transform(), )*
                    $( ActorWrapper::$marker => unreachable!(concat!("try to get access to ", stringify!($marker))), )*
                }
            }

            fn tick(&mut self, world: &mut WorldContext, shared: &mut EngineSharedSystems)
            {
                match self {
                    $( ActorWrapper::$variant(actor) => actor.tick(world, shared), )*
                    $( ActorWrapper::$marker => unreachable!(concat!("try to get access to ", stringify!($marker))), )*
                }
            }

            fn get_physical_element(&mut self) -> Option<PhysicalElement<'_>>
            {
                match self {
                    $( ActorWrapper::$variant(actor) => actor.get_physical_element(), )*
                    $( ActorWrapper::$marker => unreachable!(concat!("try to get access to ", stringify!($marker))), )*
                }
            }

            fn get_visual_element(&self) -> Option<VisualElement<'_>>
            {
                match self {
                    $( ActorWrapper::$variant(actor) => actor.get_visual_element(), )*
                    $( ActorWrapper::$marker => unreachable!(concat!("try to get access to ", stringify!($marker))), )*
                }
            }

            fn get_id(&self) -> Option<ActorID>
            {
                match self {
                    $( ActorWrapper::$variant(actor) => actor.get_id(), )*
                    $( ActorWrapper::$marker => unreachable!(concat!("try to get access to ", stringify!($marker))), )*
                }
            }

            fn set_id(&mut self, id: ActorID)
            {
                match self {
                    $( ActorWrapper::$variant(actor) => actor.set_id(id), )*
                    $( ActorWrapper::$marker => unreachable!(concat!("try to get access to ", stringify!($marker))), )*
                }
            }

            fn change_id(&mut self, id: ActorID, messenger: &mut Messenger)
            {
                match self {
                    $( ActorWrapper::$variant(actor) => actor.change_id(id, messenger), )*
                    $( ActorWrapper::$marker => unreachable!(concat!("try to get access to ", stringify!($marker))), )*
                }
            }

            fn on_added_to_world(&mut self, world: &mut WorldContext, shared: &mut EngineSharedSystems)
            {
                match self {
                    $( ActorWrapper::$variant(actor) => actor.on_added_to_world(world, shared), )*
                    $( ActorWrapper::$marker => unreachable!(concat!("try to get access to ", stringify!($marker))), )*
                }
            }

            fn on_world_started(&mut self, world: &mut WorldContext, shared: &mut EngineSharedSystems)
            {
                match self {
                    $( ActorWrapper::$variant(actor) => actor.on_world_started(world, shared), )*
                    $( ActorWrapper::$marker => unreachable!(concat!("try to get access to ", stringify!($marker))), )*
                }
            }
        }
    };
}


actor_wrapper! {
    actors {
        MainPlayer(MainPlayer),
        #[allow(dead_code)] // game content that is not wired into the current maps/modes; kept for later
        PlayerFor2d3dExample(PlayerFor2d3dExample),
        WonderingActor(WanderingActor),
        HoleGunShot(HoleGunShot),
        ObstaclesGunShot(ObstaclesGunShot),
        HoleGunMiss(HoleGunMiss),
        PlayersDoll(PlayerDoll),
        PlayersDeathExplosion(PlayersDeathExplosion),
        MachinegunShot(MachinegunShot),
        ShootingImpact(ShootingImpact),
        Flag(Flag),
        FlagBase(FlagBase),
        Hole(Hole),
        #[allow(dead_code)] // game content that is not wired into the current maps/modes; kept for later
        MoveWBonusSpot(MoveWBonusSpot),
        SessionController(SessionController),
        Wave(Wave),
        #[allow(dead_code)] // game content that is not wired into the current maps/modes; kept for later
        MoverW(MoverW),
        ShotgunShotSource(ShotgunShotSource),
        ShotgunLaserShot(ShotgunLaserShot),
        #[allow(dead_code)] // game content that is not wired into the current maps/modes; kept for later
        Observer(Observer),
        ObstacleCourseFreeMovementPlayer(ObstacleCourseFreeMovementPlayer),
        Trigger(Trigger),
        FinalTrigger(FinalTrgger),
        TriggeringWanderingActor(TriggeringWanderingActor),
        TriggeringSpecialLiftActor(TriggeringSpecialLiftActor),
        TriggerOrb(TriggerOrb),
        DropedRotatorTool(DropedRotatorTool),
        NewSpawnArea(NewSpawnArea),
    }
    markers {
        #[allow(dead_code)] // game content that is not wired into the current maps/modes; kept for later
        Diamond,
        #[allow(dead_code)] // game content that is not wired into the current maps/modes; kept for later
        Exit,
    }
}

#[derive(Clone)]
pub struct Message {
    pub from: ActorID,
    #[allow(dead_code)] // game content that is not wired into the current maps/modes; kept for later
    pub remote_sender: bool,
    pub message: MessageType,
}

#[derive(Clone)]
pub enum MessageType {
    CommonActorsMessages(CommonActorsMessage),
    SpecificActorMessage(SpecificActorMessage),
    PhysicsMessages(PhysicsMessages),
}

use client_server_protocol::Team;
use flag::Flag;
use glam::Vec4;
use hole::Hole;
use move_w_bonus::{MoveWBonusSpot, MoveWBonusSpotMessage};
use mover_w::{MoverW, MoverWMessage};
use main_player::PlayerScreenEffects;
use observer::Observer;
use player_for_2d_3d_example::PlayerFor2d3dExample;
use session_controller::SessionController;
use shotgun_laser_shot::ShotgunLaserShot;
use shotgun_shot_source::ShotgunShotSource;
use wave::Wave;

#[derive(Clone)]
pub enum CommonActorsMessage {
    SetTransform(Transform),
    Enable(bool),
    #[allow(dead_code)] // game content that is not wired into the current maps/modes; kept for later
    IncrementPosition(Vec4),
    IWasChangedMyId(ActorID),
    ClientDisconnectedFromGameServer,
}

#[derive(Clone)]
pub enum SpecificActorMessage {
    SessionControllerMessage(SessionControllerMessage),
    DropedRotatorToolMessage(DropedRotatorToolMessage),
    MoveWBonusSpotMessage(MoveWBonusSpotMessage),
    FinalTrggerMessage(FinalTrggerMessage),
    PlayersDollMessage(PlayersDollMessage),
    TriggerOrbMessage(TriggerOrbMessage),
    NewSpawnArea(NewSpawnAreaMessage),
    FlagBaseMessage(FlagBaseMessage),
    TriggerMessage(TriggerMessage),
    PlayerMessage(PlayerMessage),
    FlagMessage(FlagMessage),
    MoverW(MoverWMessage),
}

#[derive(Clone)]
pub enum PhysicsMessages {
    KinematicColliderMessage(KinematicColliderMessage),
    #[allow(dead_code)] // game content that is not wired into the current maps/modes; kept for later
    DynamicColliderMessage(DynamicColliderMessage),
    #[allow(dead_code)] // game content that is not wired into the current maps/modes; kept for later
    StaticColliderMessage(StaticColliderMessage),
    AreaMessage(AreaMessage),
}

pub trait ControlledActor {
    fn get_camera(&self) -> Camera;

    fn get_screen_effects(&self) -> &PlayerScreenEffects;

    fn get_team(&self) -> Team;

    fn spawn(
        &mut self,
        world: &mut WorldContext,
        shared: &mut EngineSharedSystems,
    );
}

