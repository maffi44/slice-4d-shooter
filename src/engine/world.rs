pub mod static_object;
pub mod level;

use self::level::Level;

use crate::{
    actor::{
        Actor, ActorID, ActorWrapper, Message
    },
    engine::{
        physics::PhysicsSystem,
        engine_handle::{
            CommandType,
            EngineHandle,
        },
    },
};

use core::panic;
use std::collections::HashMap;

use super::net::NetCommand;

pub struct World {
    pub level: Level,
    pub actors: HashMap<ActorID, ActorWrapper>,
    pub main_player_id: ActorID,
}

impl World {

    pub async fn new() -> Self {
        
        // 0 it is id of engine
        // in case when engine send message to the some actor
        // sender property will be 0      
        let (level, actors) = Level::download_level_from_server().await;

        log::info!("world system: level downloaded and init");

        let mut world = World {
            actors: HashMap::with_capacity(actors.len()),
            level,
            main_player_id: 0,
        };

        for actor in actors {
            world.add_actor_to_world(actor);
        }

        world
    }

    pub fn send_messages_and_process_commands(&mut self, engine_handle: &mut EngineHandle) {
        
        loop {
                while let Some(message) = engine_handle.boardcast_message_buffer.pop() {
                    self.send_boardcast_messages(message, engine_handle)                
                }

                while let Some((to, message)) = engine_handle.direct_message_buffer.pop() {
                    self.send_direct_messages(to, message, engine_handle)                
                }

                while let Some(command) = engine_handle.command_buffer.pop() {
                    let from = command.sender;
                    
                    match command.command_type {
                        CommandType::SpawnEffect(_) => {}
                        CommandType::SpawnActor(actor) => {
                            self.add_actor_to_world(actor);
                        }
                        CommandType::RemoveActor(id) => {
                            self.actors.remove(&id);
                        }
                        CommandType::NetCommand(command) => {
                            match command {
                                NetCommand::NetSystemIsConnectedAndGetNewPeerID(new_id) => {
                                    self.change_actor_id(from, new_id, engine_handle);
                                },
                                NetCommand::PeerConnected(id) => {

                                },
                                NetCommand::PeerDisconnected(id) => {

                                }
                            }
                        }
                    }
                }

                if engine_handle.direct_message_buffer.is_empty() &&
                    engine_handle.boardcast_message_buffer.is_empty() &&
                    engine_handle.command_buffer.is_empty()
                {
                            
                    return;
                }
            }
    }

    fn send_direct_messages(
        &mut self,
        to: ActorID,
        message: Message,
        engine_handle: &mut EngineHandle
    ) {
        if let Some(actor) = self.actors.get_mut(&to) {
            actor.recieve_message(&message, engine_handle);
        }
    }

    fn send_boardcast_messages(
        &mut self,
        message: Message,
        engine_handle: &mut EngineHandle
    ) {
        for (_, actor) in self.actors.iter_mut() {
            if actor.get_id().expect("actor does not have id") != message.from {
                actor.recieve_message(&message, engine_handle);
            } 
        }
    }

    fn change_actor_id(&mut self, old_id: ActorID, new_id: ActorID, engine_handle: &mut EngineHandle) {
        if let Some(mut actor) = self.actors.remove(&old_id) {
            actor.set_id(new_id, engine_handle);

            if let Some(mut swaped_actor) = self.actors.insert(new_id, actor) {
                
                let new_id_for_swaped_actor = self.get_new_random_uniq_id();

                swaped_actor.set_id(new_id_for_swaped_actor, engine_handle);

                self.actors.insert(new_id_for_swaped_actor, swaped_actor);
            }
        }
    }

    pub fn add_actor_to_world(&mut self, mut actor: ActorWrapper) -> ActorID {

        let id = match actor.get_id() {
            Some(id) => id,
            None => {
                let new_id = self.get_new_random_uniq_id();

                actor.init(new_id);

                new_id
            },
        };

        self.actors.insert(id, actor);

        id
    }

    pub fn remove_actor_from_world(&mut self, id: ActorID) -> Option<ActorWrapper> {

        self.actors.remove(&id)
    }

    pub fn tick(
        &mut self,
        physic_system: &PhysicsSystem,
        engine_handle: &mut EngineHandle,
        delta: f32
    ) {
        for (_, actor) in self.actors.iter_mut() {
            actor.tick(physic_system, engine_handle, delta)
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

    // 0 it is reserved ID for the Engine itself
    if id == 0u128 {
        id = get_random_non_zero_id();
    }

    id
}