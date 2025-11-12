// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2025  Timofei Molokov

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

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
pub mod obstacle_course_player_two_jumps;
pub mod obstacle_course_free_movement_player;

use std::fmt::Display;

use crate::{
    actor::{flag_base::{FlagBase, FlagBaseMessage}, obstacle_course_free_movement_player::ObstacleCourseFreeMovementPlayer, obstacle_course_player_two_jumps::ObstacleCoursePlayerTwoJumps, obstaclesgun_shot::ObstaclesGunShot}, engine::{
        audio::AudioSystem,
        effects::EffectsSystem,
        engine_handle::EngineHandle,
        physics::{
            PhysicsSystem, area::AreaMessage, colliders_container::PhysicalElement, dynamic_collider::DynamicColliderMessage, kinematic_collider::KinematicColliderMessage, static_collider::StaticColliderMessage
        },
        render::{VisualElement, camera::Camera},
        time::TimeSystem,
        ui::UISystem, world::level::Spawn
    }, transform::Transform
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
        message: Message,
        engine_handle: &mut EngineHandle,
        physics_system: &PhysicsSystem,
        audio_system: &mut AudioSystem,
        ui_system: &mut UISystem,
        time_system: &TimeSystem,
        effects_system: &mut EffectsSystem,
    ) {}

    fn get_mut_transform(&mut self) -> &mut Transform;
    
    fn get_transform(&self) -> &Transform;

    fn tick(
        &mut self,
        physic_system: &PhysicsSystem,
        engine_handle: &mut EngineHandle,
        audio_system: &mut AudioSystem,
        ui_system: &mut UISystem,
        time_system: &mut TimeSystem,
        effects_system: &mut EffectsSystem,
        delta: f32
    ) {}

    fn get_physical_element(&mut self) -> Option<PhysicalElement> {None}

    fn get_visual_element(&self) -> Option<VisualElement> {None}

    fn get_id(&self) -> Option<ActorID>;
        
    fn set_id(&mut self, id: ActorID);

    fn change_id(&mut self, id: ActorID, engine_handle: &mut EngineHandle) {
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

        engine_handle.send_boardcast_message(
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

}

pub enum ActorWrapper {
    MainPlayer(MainPlayer),
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
    MoveWBonusSpot(MoveWBonusSpot),
    SessionController(SessionController),
    Wave(Wave),
    MoverW(MoverW),
    ShotgunShotSource(ShotgunShotSource),
    ShotgunLaserShot(ShotgunLaserShot),
    Observer(Observer),
    Diamond,
    Exit,
    ObstacleCoursePlayerTwoJumps(ObstacleCoursePlayerTwoJumps),
    ObstacleCourseFreeMovementPlayer(ObstacleCourseFreeMovementPlayer),
}

impl Display for ActorWrapper
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let actor_type = match self
        {
            ActorWrapper::MainPlayer(_) => "MainPlayer",
            ActorWrapper::PlayerFor2d3dExample(_) => "PlayerFor2d3dExample",
            ActorWrapper::WonderingActor(_) => "WonderingActor",
            ActorWrapper::HoleGunShot(_) => "HoleGunShot",
            ActorWrapper::ObstaclesGunShot(_) => "ObstaclesGunShot",
            ActorWrapper::HoleGunMiss(_) => "HoleGunMiss",
            ActorWrapper::PlayersDoll(_) => "PlayersDoll",
            ActorWrapper::PlayersDeathExplosion(_) => "PlayersDeathExplosion",
            ActorWrapper::MachinegunShot(_) => "MachinegunShot",
            ActorWrapper::ShootingImpact(_) => "ShootingImpact",
            ActorWrapper::Flag(_) => "Flag",
            ActorWrapper::FlagBase(_) => "FlagBase",
            ActorWrapper::Hole(_) => "Hole",
            ActorWrapper::MoveWBonusSpot(_) => "MoveWBonusSpot",
            ActorWrapper::SessionController(_) => "SessionController",
            ActorWrapper::Wave(_) => "Wave",
            ActorWrapper::MoverW(_) => "MoverW",
            ActorWrapper::ShotgunShotSource(_) => "ShotgunShotSource",
            ActorWrapper::ShotgunLaserShot(_) => "ShotgunLaserShot",
            ActorWrapper::Observer(_) => "Observer",
            ActorWrapper::Diamond => "Diamond",
            ActorWrapper::Exit => "Exit",
            ActorWrapper::ObstacleCoursePlayerTwoJumps(_) => "ObstacleCoursePlayerTwoJumps",
            ActorWrapper::ObstacleCourseFreeMovementPlayer(_) => "ObstacleCourseFreeMovementPlayer",
        };

        write!(f, "Actor: {}", actor_type)
    }
}

impl Actor for ActorWrapper {

    fn get_transform(&self) -> &Transform {
        match self {
            ActorWrapper::MainPlayer(actor) => {
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
            ActorWrapper::SessionController(actor) => {
                actor.get_transform()
            }
            ActorWrapper::Flag(actor) => {
                actor.get_transform()
            }
            ActorWrapper::Hole(actor) => {
                actor.get_transform()
            }
            ActorWrapper::MoveWBonusSpot(actor) => {
                actor.get_transform()
            }
            ActorWrapper::Wave(actor) => {
                actor.get_transform()
            }
            ActorWrapper::MoverW(actor) => {
                actor.get_transform()
            }
            ActorWrapper::PlayerFor2d3dExample(actor) => {
                actor.get_transform()
            }
            ActorWrapper::ShotgunShotSource(actor) => {
                actor.get_transform()
            }
            ActorWrapper::ShotgunLaserShot(actor) => {
                actor.get_transform()
            }
            ActorWrapper::Observer(actor) => {
                actor.get_transform()
            }
            ActorWrapper::FlagBase(actor) => {
                actor.get_transform()
            }
            ActorWrapper::ObstaclesGunShot(actor) => {
                actor.get_transform()
            }
            ActorWrapper::ObstacleCoursePlayerTwoJumps(actor) => {
                actor.get_transform()
            }
            ActorWrapper::ObstacleCourseFreeMovementPlayer(actor) => {
                actor.get_transform()
            }
            ActorWrapper::Diamond => {unreachable!("try to get access to diamond")},
            ActorWrapper::Exit => {unreachable!("try to get access to exit")},
        }
    }

    fn get_mut_transform(&mut self) -> &mut Transform {
        match  self {
            ActorWrapper::MainPlayer(actor) => {
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
            ActorWrapper::SessionController(actor) => {
                actor.get_mut_transform()
            }
            ActorWrapper::Flag(actor) => {
                actor.get_mut_transform()
            }
            ActorWrapper::Hole(actor) => {
                actor.get_mut_transform()
            }
            ActorWrapper::MoveWBonusSpot(actor) => {
                actor.get_mut_transform()
            }
            ActorWrapper::Wave(actor) => {
                actor.get_mut_transform()
            }
            ActorWrapper::MoverW(actor) => {
                actor.get_mut_transform()
            }
            ActorWrapper::PlayerFor2d3dExample(actor) => {
                actor.get_mut_transform()
            }
            ActorWrapper::ShotgunShotSource(actor) => {
                actor.get_mut_transform()
            }
            ActorWrapper::ShotgunLaserShot(actor) => {
                actor.get_mut_transform()
            }
            ActorWrapper::Observer(actor) => {
                actor.get_mut_transform()
            }
            ActorWrapper::FlagBase(actor) => {
                actor.get_mut_transform()
            }
            ActorWrapper::ObstaclesGunShot(actor) => {
                actor.get_mut_transform()
            }
            ActorWrapper::ObstacleCoursePlayerTwoJumps(actor) => {
                actor.get_mut_transform()
            }
            ActorWrapper::ObstacleCourseFreeMovementPlayer(actor) => {
                actor.get_mut_transform()
            }
            ActorWrapper::Diamond => {unreachable!("try to get access to diamond")},
            ActorWrapper::Exit => {unreachable!("try to get access to exit")},
        }
    }

    fn recieve_message(
        &mut self,
        message: Message,
        engine_handle: &mut EngineHandle,
        physics_system: &PhysicsSystem,
        audio_system: &mut AudioSystem,
        ui_system: &mut UISystem,
        time_system: &TimeSystem,
        effects_system: &mut EffectsSystem,
    ) {
        match  self {
            ActorWrapper::MainPlayer(actor) => {
                actor.recieve_message(message, engine_handle, physics_system, audio_system,  ui_system, time_system, effects_system)
            },
            ActorWrapper::WonderingActor(actor) => {
                actor.recieve_message(message, engine_handle, physics_system, audio_system,  ui_system, time_system, effects_system)
            },
            ActorWrapper::HoleGunShot(actor) => {
                actor.recieve_message(message, engine_handle, physics_system, audio_system,  ui_system, time_system, effects_system)
            },
            ActorWrapper::HoleGunMiss(actor) => {
                actor.recieve_message(message, engine_handle, physics_system, audio_system,  ui_system, time_system, effects_system)
            },
            ActorWrapper::PlayersDoll(actor) => {
                actor.recieve_message(message, engine_handle, physics_system, audio_system,  ui_system, time_system, effects_system)
            },
            ActorWrapper::PlayersDeathExplosion(actor) => {
                actor.recieve_message(message, engine_handle, physics_system, audio_system,  ui_system, time_system, effects_system)
            },
            ActorWrapper::MachinegunShot(actor) => {
                actor.recieve_message(message, engine_handle, physics_system, audio_system,  ui_system, time_system, effects_system)
            },
            ActorWrapper::ShootingImpact(actor) => {
                actor.recieve_message(message, engine_handle, physics_system, audio_system,  ui_system, time_system, effects_system)
            },
            ActorWrapper::SessionController(actor) => {
                actor.recieve_message(message, engine_handle, physics_system, audio_system,  ui_system, time_system, effects_system)
            }
            ActorWrapper::Flag(actor) => {
                actor.recieve_message(message, engine_handle, physics_system, audio_system,  ui_system, time_system, effects_system)
            }
            ActorWrapper::Hole(actor) => {
                actor.recieve_message(message, engine_handle, physics_system, audio_system,  ui_system, time_system, effects_system)
            }
            ActorWrapper::MoveWBonusSpot(actor) => {
                actor.recieve_message(message, engine_handle, physics_system, audio_system,  ui_system, time_system, effects_system)
            }
            ActorWrapper::Wave(actor) => {
                actor.recieve_message(message, engine_handle, physics_system, audio_system,  ui_system, time_system, effects_system)
            }
            ActorWrapper::MoverW(actor) => {
                actor.recieve_message(message, engine_handle, physics_system, audio_system,  ui_system, time_system, effects_system)
            }
            ActorWrapper::PlayerFor2d3dExample(actor) => {
                actor.recieve_message(message, engine_handle, physics_system, audio_system,  ui_system, time_system, effects_system)
            }
            ActorWrapper::ShotgunShotSource(actor) => {
                actor.recieve_message(message, engine_handle, physics_system, audio_system,  ui_system, time_system, effects_system)
            }
            ActorWrapper::ShotgunLaserShot(actor) => {
                actor.recieve_message(message, engine_handle, physics_system, audio_system,  ui_system, time_system, effects_system)
            }
            ActorWrapper::Observer(actor) => {
                actor.recieve_message(message, engine_handle, physics_system, audio_system,  ui_system, time_system, effects_system)
            }
            ActorWrapper::FlagBase(actor) => {
                actor.recieve_message(message, engine_handle, physics_system, audio_system,  ui_system, time_system, effects_system)
            }
            ActorWrapper::ObstaclesGunShot(actor) => {
                actor.recieve_message(message, engine_handle, physics_system, audio_system,  ui_system, time_system, effects_system)
            }
            ActorWrapper::ObstacleCoursePlayerTwoJumps(actor) => {
                actor.recieve_message(message, engine_handle, physics_system, audio_system,  ui_system, time_system, effects_system)
            }
            ActorWrapper::ObstacleCourseFreeMovementPlayer(actor) => {
                actor.recieve_message(message, engine_handle, physics_system, audio_system,  ui_system, time_system, effects_system)
            }
            ActorWrapper::Diamond => {unreachable!("try to get access to diamond")},
            ActorWrapper::Exit => {unreachable!("try to get access to exit")},
        }
    }

    fn tick(
        &mut self,
        physic_system: &PhysicsSystem,
        engine_handle: &mut EngineHandle,
        audio_system: &mut AudioSystem,
        ui_system: &mut UISystem,
        time_system: &mut TimeSystem,
        effects_system: &mut EffectsSystem,
        delta: f32
    ) {
        match  self {
            ActorWrapper::MainPlayer(actor) => {
                actor.tick(physic_system, engine_handle, audio_system, ui_system, time_system, effects_system, delta)
            },
            ActorWrapper::WonderingActor(actor) => {
                actor.tick(physic_system, engine_handle, audio_system, ui_system, time_system, effects_system, delta)
            },
            ActorWrapper::HoleGunShot(actor) => {
                actor.tick(physic_system, engine_handle, audio_system, ui_system, time_system, effects_system, delta)
            },
            ActorWrapper::HoleGunMiss(actor) => {
                actor.tick(physic_system, engine_handle, audio_system, ui_system, time_system, effects_system, delta)
            },
            ActorWrapper::PlayersDoll(actor) => {
                actor.tick(physic_system, engine_handle, audio_system, ui_system, time_system, effects_system, delta)
            },
            ActorWrapper::PlayersDeathExplosion(actor) => {
                actor.tick(physic_system, engine_handle, audio_system, ui_system, time_system, effects_system, delta)
            },
            ActorWrapper::MachinegunShot(actor) => {
                actor.tick(physic_system, engine_handle, audio_system, ui_system, time_system, effects_system, delta)
            },
            ActorWrapper::ShootingImpact(actor) => {
                actor.tick(physic_system, engine_handle, audio_system, ui_system, time_system, effects_system, delta)
            },
            ActorWrapper::SessionController(actor) => {
                actor.tick(physic_system, engine_handle, audio_system, ui_system, time_system, effects_system, delta)
            }
            ActorWrapper::Flag(actor) => {
                actor.tick(physic_system, engine_handle, audio_system, ui_system, time_system, effects_system, delta)
            }
            ActorWrapper::Hole(actor) => {
                actor.tick(physic_system, engine_handle, audio_system, ui_system, time_system, effects_system, delta)
            }
            ActorWrapper::MoveWBonusSpot(actor) => {
                actor.tick(physic_system, engine_handle, audio_system, ui_system, time_system, effects_system, delta)
            }
            ActorWrapper::Wave(actor) => {
                actor.tick(physic_system, engine_handle, audio_system, ui_system, time_system, effects_system, delta)
            }
            ActorWrapper::MoverW(actor) => {
                actor.tick(physic_system, engine_handle, audio_system, ui_system, time_system, effects_system, delta)
            }
            ActorWrapper::PlayerFor2d3dExample(actor) => {
                actor.tick(physic_system, engine_handle, audio_system, ui_system, time_system, effects_system, delta)
            }
            ActorWrapper::ShotgunShotSource(actor) => {
                actor.tick(physic_system, engine_handle, audio_system, ui_system, time_system, effects_system, delta)
            }
            ActorWrapper::ShotgunLaserShot(actor) => {
                actor.tick(physic_system, engine_handle, audio_system, ui_system, time_system, effects_system, delta)
            }
            ActorWrapper::Observer(actor) => {
                actor.tick(physic_system, engine_handle, audio_system, ui_system, time_system, effects_system, delta)
            }
            ActorWrapper::FlagBase(actor) => {
                actor.tick(physic_system, engine_handle, audio_system, ui_system, time_system, effects_system, delta)
            }
            ActorWrapper::ObstaclesGunShot(actor) => {
                actor.tick(physic_system, engine_handle, audio_system, ui_system, time_system, effects_system, delta)
            }
            ActorWrapper::ObstacleCoursePlayerTwoJumps(actor) => {
                actor.tick(physic_system, engine_handle, audio_system, ui_system, time_system, effects_system, delta)
            }
            ActorWrapper::ObstacleCourseFreeMovementPlayer(actor) => {
                actor.tick(physic_system, engine_handle, audio_system, ui_system, time_system, effects_system, delta)
            }
            ActorWrapper::Diamond => {unreachable!("try to get access to diamond")},
            ActorWrapper::Exit => {unreachable!("try to get access to exit")},
        }
    }

    fn get_physical_element(&mut self) -> Option<PhysicalElement> {
        match  self {
            ActorWrapper::MainPlayer(actor) => {
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
            ActorWrapper::SessionController(actor) => {
                actor.get_physical_element()
            }
            ActorWrapper::Flag(actor) => {
                actor.get_physical_element()
            }
            ActorWrapper::Hole(actor) => {
                actor.get_physical_element()
            }
            ActorWrapper::MoveWBonusSpot(actor) => {
                actor.get_physical_element()
            }
            ActorWrapper::Wave(actor) => {
                actor.get_physical_element()
            }
            ActorWrapper::MoverW(actor) => {
                actor.get_physical_element()
            }
            ActorWrapper::PlayerFor2d3dExample(actor) => {
                actor.get_physical_element()
            }
            ActorWrapper::ShotgunShotSource(actor) => {
                actor.get_physical_element()
            }
            ActorWrapper::ShotgunLaserShot(actor) => {
                actor.get_physical_element()
            }
            ActorWrapper::Observer(actor) => {
                actor.get_physical_element()
            }
            ActorWrapper::FlagBase(actor) => {
                actor.get_physical_element()
            }
            ActorWrapper::ObstaclesGunShot(actor) => {
                actor.get_physical_element()
            }
            ActorWrapper::ObstacleCoursePlayerTwoJumps(actor) => {
                actor.get_physical_element()
            }
            ActorWrapper::ObstacleCourseFreeMovementPlayer(actor) => {
                actor.get_physical_element()
            }
            ActorWrapper::Diamond => {unreachable!("try to get access to diamond")},
            ActorWrapper::Exit => {unreachable!("try to get access to exit")},
        }
    }

    fn get_visual_element(&self) -> Option<VisualElement>{
        match self {
            ActorWrapper::MainPlayer(actor) => {
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
            ActorWrapper::SessionController(actor) => {
                actor.get_visual_element()
            }
            ActorWrapper::Flag(actor) => {
                actor.get_visual_element()
            }
            ActorWrapper::Hole(actor) => {
                actor.get_visual_element()
            }
            ActorWrapper::MoveWBonusSpot(actor) => {
                actor.get_visual_element()
            }
            ActorWrapper::Wave(actor) => {
                actor.get_visual_element()
            }
            ActorWrapper::MoverW(actor) => {
                actor.get_visual_element()
            }
            ActorWrapper::PlayerFor2d3dExample(actor) => {
                actor.get_visual_element()
            }
            ActorWrapper::ShotgunShotSource(actor) => {
                actor.get_visual_element()
            }
            ActorWrapper::ShotgunLaserShot(actor) => {
                actor.get_visual_element()
            }
            ActorWrapper::Observer(actor) => {
                actor.get_visual_element()
            }
            ActorWrapper::FlagBase(actor) => {
                actor.get_visual_element()
            }
            ActorWrapper::ObstaclesGunShot(actor) => {
                actor.get_visual_element()
            }
            ActorWrapper::ObstacleCoursePlayerTwoJumps(actor) => {
                actor.get_visual_element()
            }
            ActorWrapper::ObstacleCourseFreeMovementPlayer(actor) => {
                actor.get_visual_element()
            }
            ActorWrapper::Diamond => {unreachable!("try to get access to diamond")},
            ActorWrapper::Exit => {unreachable!("try to get access to exit")},
        }
    }

    fn get_id(&self) -> Option<ActorID> {
        match self {
            ActorWrapper::MainPlayer(actor) => {
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
            ActorWrapper::SessionController(actor) => {
                actor.get_id()
            }
            ActorWrapper::Flag(actor) => {
                actor.get_id()
            }
            ActorWrapper::Hole(actor) => {
                actor.get_id()
            }
            ActorWrapper::MoveWBonusSpot(actor) => {
                actor.get_id()
            }
            ActorWrapper::Wave(actor) => {
                actor.get_id()
            }
            ActorWrapper::MoverW(actor) => {
                actor.get_id()
            }
            ActorWrapper::PlayerFor2d3dExample(actor) => {
                actor.get_id()
            }
            ActorWrapper::ShotgunShotSource(actor) => {
                actor.get_id()
            }
            ActorWrapper::ShotgunLaserShot(actor) => {
                actor.get_id()
            }
            ActorWrapper::Observer(actor) => {
                actor.get_id()
            }
            ActorWrapper::FlagBase(actor) => {
                actor.get_id()
            }
            ActorWrapper::ObstaclesGunShot(actor) => {
                actor.get_id()
            }
            ActorWrapper::ObstacleCoursePlayerTwoJumps(actor) => {
                actor.get_id()
            }
            ActorWrapper::ObstacleCourseFreeMovementPlayer(actor) => {
                actor.get_id()
            }
            ActorWrapper::Diamond => {unreachable!("try to get access to diamond")},
            ActorWrapper::Exit => {unreachable!("try to get access to exit")},
        }
    }

    fn change_id(&mut self, id: ActorID, engine_handle: &mut EngineHandle) {
        match self {
            ActorWrapper::MainPlayer(actor) => {
                actor.change_id(id, engine_handle)
            },
            ActorWrapper::WonderingActor(actor) => {
                actor.change_id(id, engine_handle)
            },
            ActorWrapper::HoleGunShot(actor) => {
                actor.change_id(id, engine_handle)
            },
            ActorWrapper::HoleGunMiss(actor) => {
                actor.change_id(id, engine_handle)
            },
            ActorWrapper::PlayersDoll(actor) => {
                actor.change_id(id, engine_handle)
            },
            ActorWrapper::PlayersDeathExplosion(actor) => {
                actor.change_id(id, engine_handle)
            },
            ActorWrapper::MachinegunShot(actor) => {
                actor.change_id(id, engine_handle)
            },
            ActorWrapper::ShootingImpact(actor) => {
                actor.change_id(id, engine_handle)
            },
            ActorWrapper::SessionController(actor) => {
                actor.change_id(id, engine_handle)
            }
            ActorWrapper::Flag(actor) => {
                actor.change_id(id, engine_handle)
            }
            ActorWrapper::Hole(actor) => {
                actor.change_id(id, engine_handle)
            }
            ActorWrapper::MoveWBonusSpot(actor) => {
                actor.change_id(id, engine_handle)
            }
            ActorWrapper::Wave(actor) => {
                actor.change_id(id, engine_handle)
            }
            ActorWrapper::MoverW(actor) => {
                actor.change_id(id, engine_handle)
            }
            ActorWrapper::PlayerFor2d3dExample(actor) => {
                actor.change_id(id, engine_handle)
            }
            ActorWrapper::ShotgunShotSource(actor) => {
                actor.change_id(id, engine_handle)
            }
            ActorWrapper::ShotgunLaserShot(actor) => {
                actor.change_id(id, engine_handle)
            }
            ActorWrapper::Observer(actor) => {
                actor.change_id(id, engine_handle)
            }
            ActorWrapper::FlagBase(actor) => {
                actor.change_id(id, engine_handle)
            }
            ActorWrapper::ObstaclesGunShot(actor) => {
                actor.change_id(id, engine_handle)
            }
            ActorWrapper::ObstacleCoursePlayerTwoJumps(actor) => {
                actor.change_id(id, engine_handle)
            }
            ActorWrapper::ObstacleCourseFreeMovementPlayer(actor) => {
                actor.change_id(id, engine_handle)
            }
            ActorWrapper::Diamond => {unreachable!("try to get access to diamond")},
            ActorWrapper::Exit => {unreachable!("try to get access to exit")},
        }
    }

    fn set_id(&mut self, id: ActorID) {
        match  self {
            ActorWrapper::MainPlayer(actor) => {
                actor.set_id(id)
            },
            ActorWrapper::WonderingActor(actor) => {
                actor.set_id(id)
            },
            ActorWrapper::HoleGunShot(actor) => {
                actor.set_id(id)
            },
            ActorWrapper::HoleGunMiss(actor) => {
                actor.set_id(id)
            },
            ActorWrapper::PlayersDoll(actor) => {
                actor.set_id(id)
            },
            ActorWrapper::PlayersDeathExplosion(actor) => {
                actor.set_id(id)
            },
            ActorWrapper::MachinegunShot(actor) => {
                actor.set_id(id)
            },
            ActorWrapper::ShootingImpact(actor) => {
                actor.set_id(id)
            },
            ActorWrapper::SessionController(actor) => {
                actor.set_id(id)
            }
            ActorWrapper::Flag(actor) => {
                actor.set_id(id)
            }
            ActorWrapper::Hole(actor) => {
                actor.set_id(id)
            }
            ActorWrapper::MoveWBonusSpot(actor) => {
                actor.set_id(id)
            }
            ActorWrapper::Wave(actor) => {
                actor.set_id(id)
            }
            ActorWrapper::MoverW(actor) => {
                actor.set_id(id)
            }
            ActorWrapper::PlayerFor2d3dExample(actor) => {
                actor.set_id(id)
            }
            ActorWrapper::ShotgunShotSource(actor) => {
                actor.set_id(id)
            }
            ActorWrapper::ShotgunLaserShot(actor) => {
                actor.set_id(id)
            }
            ActorWrapper::Observer(actor) => {
                actor.set_id(id)
            }
            ActorWrapper::FlagBase(actor) => {
                actor.set_id(id)
            }
            ActorWrapper::ObstaclesGunShot(actor) => {
                actor.set_id(id)
            }
            ActorWrapper::ObstacleCoursePlayerTwoJumps(actor) => {
                actor.set_id(id)
            }
            ActorWrapper::ObstacleCourseFreeMovementPlayer(actor) => {
                actor.set_id(id)
            }
            ActorWrapper::Diamond => {unreachable!("try to get access to diamond")},
            ActorWrapper::Exit => {unreachable!("try to get access to exit")},
        }
    }

    fn get_actor_as_controlled(&self) -> Option<&dyn ControlledActor>
    {
        match  self {
            ActorWrapper::MainPlayer(actor) => {
                actor.get_actor_as_controlled()
            },
            ActorWrapper::WonderingActor(actor) => {
                actor.get_actor_as_controlled()
            },
            ActorWrapper::HoleGunShot(actor) => {
                actor.get_actor_as_controlled()
            },
            ActorWrapper::HoleGunMiss(actor) => {
                actor.get_actor_as_controlled()
            },
            ActorWrapper::PlayersDoll(actor) => {
                actor.get_actor_as_controlled()
            },
            ActorWrapper::PlayersDeathExplosion(actor) => {
                actor.get_actor_as_controlled()
            },
            ActorWrapper::MachinegunShot(actor) => {
                actor.get_actor_as_controlled()
            },
            ActorWrapper::ShootingImpact(actor) => {
                actor.get_actor_as_controlled()
            },
            ActorWrapper::SessionController(actor) => {
                actor.get_actor_as_controlled()
            }
            ActorWrapper::Flag(actor) => {
                actor.get_actor_as_controlled()
            }
            ActorWrapper::Hole(actor) => {
                actor.get_actor_as_controlled()
            }
            ActorWrapper::MoveWBonusSpot(actor) => {
                actor.get_actor_as_controlled()
            }
            ActorWrapper::Wave(actor) => {
                actor.get_actor_as_controlled()
            }
            ActorWrapper::MoverW(actor) => {
                actor.get_actor_as_controlled()
            }
            ActorWrapper::PlayerFor2d3dExample(actor) => {
                actor.get_actor_as_controlled()
            }
            ActorWrapper::ShotgunShotSource(actor) => {
                actor.get_actor_as_controlled()
            }
            ActorWrapper::ShotgunLaserShot(actor) => {
                actor.get_actor_as_controlled()
            }
            ActorWrapper::Observer(actor) => {
                actor.get_actor_as_controlled()
            }
            ActorWrapper::FlagBase(actor) => {
                actor.get_actor_as_controlled()
            }
            ActorWrapper::ObstaclesGunShot(actor) => {
                actor.get_actor_as_controlled()
            }
            ActorWrapper::ObstacleCoursePlayerTwoJumps(actor) => {
                actor.get_actor_as_controlled()
            }
            ActorWrapper::ObstacleCourseFreeMovementPlayer(actor) => {
                actor.get_actor_as_controlled()
            }
            ActorWrapper::Diamond => {unreachable!("try to get access to diamond")},
            ActorWrapper::Exit => {unreachable!("try to get access to exit")},
        }
    }

    fn get_actor_as_controlled_mut(&mut self) -> Option<&mut dyn ControlledActor>
    {
        match  self {
            ActorWrapper::MainPlayer(actor) => {
                actor.get_actor_as_controlled_mut()
            },
            ActorWrapper::WonderingActor(actor) => {
                actor.get_actor_as_controlled_mut()
            },
            ActorWrapper::HoleGunShot(actor) => {
                actor.get_actor_as_controlled_mut()
            },
            ActorWrapper::HoleGunMiss(actor) => {
                actor.get_actor_as_controlled_mut()
            },
            ActorWrapper::PlayersDoll(actor) => {
                actor.get_actor_as_controlled_mut()
            },
            ActorWrapper::PlayersDeathExplosion(actor) => {
                actor.get_actor_as_controlled_mut()
            },
            ActorWrapper::MachinegunShot(actor) => {
                actor.get_actor_as_controlled_mut()
            },
            ActorWrapper::ShootingImpact(actor) => {
                actor.get_actor_as_controlled_mut()
            },
            ActorWrapper::SessionController(actor) => {
                actor.get_actor_as_controlled_mut()
            }
            ActorWrapper::Flag(actor) => {
                actor.get_actor_as_controlled_mut()
            }
            ActorWrapper::Hole(actor) => {
                actor.get_actor_as_controlled_mut()
            }
            ActorWrapper::MoveWBonusSpot(actor) => {
                actor.get_actor_as_controlled_mut()
            }
            ActorWrapper::Wave(actor) => {
                actor.get_actor_as_controlled_mut()
            }
            ActorWrapper::MoverW(actor) => {
                actor.get_actor_as_controlled_mut()
            }
            ActorWrapper::PlayerFor2d3dExample(actor) => {
                actor.get_actor_as_controlled_mut()
            }
            ActorWrapper::ShotgunShotSource(actor) => {
                actor.get_actor_as_controlled_mut()
            }
            ActorWrapper::ShotgunLaserShot(actor) => {
                actor.get_actor_as_controlled_mut()
            }
            ActorWrapper::Observer(actor) => {
                actor.get_actor_as_controlled_mut()
            }
            ActorWrapper::FlagBase(actor) => {
                actor.get_actor_as_controlled_mut()
            }
            ActorWrapper::ObstaclesGunShot(actor) => {
                actor.get_actor_as_controlled_mut()
            }
            ActorWrapper::ObstacleCoursePlayerTwoJumps(actor) => {
                actor.get_actor_as_controlled_mut()
            }
            ActorWrapper::ObstacleCourseFreeMovementPlayer(actor) => {
                actor.get_actor_as_controlled_mut()
            }
            ActorWrapper::Diamond => {unreachable!("try to get access to diamond")},
            ActorWrapper::Exit => {unreachable!("try to get access to exit")},
        }
    }
}

#[derive(Clone)]
pub struct Message {
    pub from: ActorID,
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
use main_player::{player_input_master::InputMaster, PlayerScreenEffects};
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
    IncrementPosition(Vec4),
    IWasChangedMyId(ActorID),
    ClientDisconnectedFromGameServer,
}

#[derive(Clone)]
pub enum SpecificActorMessage {
    SessionControllerMessage(SessionControllerMessage),
    MoveWBonusSpotMessage(MoveWBonusSpotMessage),
    PlayersDollMessage(PlayersDollMessage),
    FlagBaseMessage(FlagBaseMessage),
    PlayerMessage(PlayerMessage),
    FlagMessage(FlagMessage),
    MoverW(MoverWMessage)
}

#[derive(Clone)]
pub enum PhysicsMessages {
    KinematicColliderMessage(KinematicColliderMessage),
    StaticColliderMessage(StaticColliderMessage),
    DynamicColliderMessage(DynamicColliderMessage),
    AreaMessage(AreaMessage),
}

pub trait ControlledActor {
    fn get_camera(&self) -> Camera;

    fn get_screen_effects(&self) -> &PlayerScreenEffects;

    fn get_team(&self) -> Team;

    fn get_input_master(&mut self) -> &mut InputMaster;

    fn spawn(
        &mut self,
        spawns: &mut Vec<Spawn>,
        physics_system: &PhysicsSystem,
        ui_system: &mut UISystem,
        audio_system: &mut AudioSystem,
        engine_handle: &mut EngineHandle,
    );
}

