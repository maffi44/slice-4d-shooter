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

pub mod static_object;
pub mod level;

use self::level::Level;

use crate::{
    actor::{
        Actor, ActorID, ActorWrapper, Message, MessageType, SpecificActorMessage, main_player::{
            PlayerMessage, player_settings::PlayerSettings
        }
    },
    engine::{
        engine_handle::{
            CommandType,
            EngineHandle,
        },
        physics::PhysicsSystem, render::RenderSystem
    },
};

use core::panic;
use std::collections::HashMap;

use super::{
    audio::AudioSystem, effects::EffectsSystem, engine_handle::Command, net::NetSystem, time::TimeSystem, ui::UISystem
};

use client_server_protocol::{NetCommand, Team};
use tokio::task::JoinHandle;

pub struct World {
    pub level: Option<Level>,
    pub actors: HashMap<ActorID, ActorWrapper>,
    pub main_actor_id: ActorID,
    pub players_settings: PlayerSettings,
    pub preloaded_levels: HashMap<String, JoinHandle<Level>>
}

impl World {

    pub async fn new(
        engine_handle: &mut EngineHandle,
        players_settings: PlayerSettings,
        // level_name: String
    ) -> Self
    {
        
        // 0 it is id of engine
        // in case when engine send message to the some actor
        // sender property will be 0      

        log::info!("world system: level downloaded and init");

        let world = World {
            actors: HashMap::new(),
            players_settings,
            level: None,
            main_actor_id: 0,
            preloaded_levels: HashMap::with_capacity(2)
        };

        // engine_handle.send_command(
        //     Command {
        //         sender: 0u128,
        //         command_type: CommandType::LoadNewLevelSync(level_name),
        //     }
        // );

        world
    }

    pub fn change_actor_id(&mut self, old_id: ActorID, new_id: ActorID, engine_handle: &mut EngineHandle) {
        if let Some(mut actor) = self.remove_actor_from_world(old_id) {
            actor.change_id(new_id, engine_handle);

            if let Some(mut swaped_actor) = self.actors.insert(new_id, actor) {
                
                let new_id_for_swaped_actor = self.get_new_random_uniq_id();

                swaped_actor.change_id(new_id_for_swaped_actor, engine_handle);

                self.actors.insert(new_id_for_swaped_actor, swaped_actor);
            }
        }
    }

    pub fn set_new_level(
        &mut self,
        mut level: Level,
        engine_handle: &mut EngineHandle,
        physic_system: &PhysicsSystem,
        audio_system: &mut AudioSystem,
        ui_system: &mut UISystem,
        time_system: &mut TimeSystem,
        effects_system: &mut EffectsSystem
    )
    {
        self.actors.clear();

        self.add_main_actor_to_world(
            level.main_actor.take().expect("level have not main actor"),
            engine_handle,
            physic_system,
            audio_system ,
            ui_system ,
            time_system,
            effects_system
        );

        for actor in level.actors.take().unwrap() {
            self.add_actor_to_world(
                actor,
                engine_handle,
                physic_system,
                audio_system ,
                ui_system ,
                time_system,
                effects_system
            );
        }

        self.level = Some(level);
    }

    pub fn add_actor_to_world(
        &mut self,
        mut actor: ActorWrapper,
        engine_handle: &mut EngineHandle,
        physic_system: &PhysicsSystem,
        audio_system: &mut AudioSystem,
        ui_system: &mut UISystem,
        time_system: &mut TimeSystem,
        effects_system: &mut EffectsSystem
    ) -> ActorID {

        let id = match actor.get_id() {
            Some(id) => id,
            None => {
                let new_id = self.get_new_random_uniq_id();

                actor.set_id(new_id);

                new_id
            },
        };

        if let Some(mut swaped_actor) = self.actors.insert(id, actor) {
                
            let new_id_for_swaped_actor = self.get_new_random_uniq_id();

            swaped_actor.change_id(new_id_for_swaped_actor, engine_handle);

            self.actors.insert(new_id_for_swaped_actor, swaped_actor);
        }

        self.actors.get_mut(&id).expect("world have not actor after added it")
            .on_added_to_world(
                physic_system,
                engine_handle,
                audio_system,
                ui_system,
                time_system,
                effects_system
            );

        id
    }

    pub fn add_main_actor_to_world(
        &mut self,
        actor: ActorWrapper,
        engine_handle: &mut EngineHandle,
        physic_system: &PhysicsSystem,
        audio_system: &mut AudioSystem,
        ui_system: &mut UISystem,
        time_system: &mut TimeSystem,
        effects_system: &mut EffectsSystem
    ) -> ActorID {

        let id = self.add_actor_to_world(
            actor,
            engine_handle,
            physic_system,
            audio_system ,
            ui_system ,
            time_system,
            effects_system
        );

        self.main_actor_id = id;

        id
    }

    pub fn remove_actor_from_world(&mut self, id: ActorID) -> Option<ActorWrapper> {

        self.actors.remove(&id)
    }

    pub fn tick(
        &mut self,
        physic_system: &PhysicsSystem,
        engine_handle: &mut EngineHandle,
        audio_system: &mut AudioSystem,
        ui_system: &mut UISystem,
        time_system: &mut TimeSystem,
        effects_system: &mut EffectsSystem,
    ) {
        let delta = time_system.get_prev_frame_duration();

        for (_, actor) in self.actors.iter_mut()
        {
            actor.tick(
                physic_system,
                engine_handle,
                audio_system,
                ui_system,
                time_system,
                effects_system,
                delta
            );
        }

    }

    fn get_new_random_uniq_id(&self) -> ActorID {
        let mut new_id = get_random_non_zero_id();

        while self.actors.contains_key(&new_id) {
            new_id = get_random_non_zero_id();
        }

        new_id
    }

}

fn get_random_non_zero_id() -> ActorID {
    let mut bytes : [u8;16] = [0;16];
    let res = getrandom::getrandom(&mut bytes);
    
    if let Err(err) = res {
        panic!("Can't make random u128 in get_random_id function");
    }

    let mut id: u128 = u128::from_be_bytes(bytes);

    // 0 is reserved ID for the Engine itself
    while id == 0u128 {
        id = get_random_non_zero_id();
    }

    id
}