pub mod level;
pub mod static_object;

use static_object::StaticObject;
use core::panic;
use std::collections::HashMap;


use self::level::Level;

use super::{
    actor::{
        self, Actor, ActorID, ActorWrapper, Message
    },
    engine_handle::{CommandType::SpawnEffect, EngineHandle},
};

use glam::{Vec3, Vec4};


pub enum PlayerAccessError {
    HaveNotPlayer
}

pub struct World {
    pub level: Level,
    pub actors: HashMap<ActorID, ActorWrapper>,
    all_ids: Vec<ActorID>,
    pub main_camera_from: ActorID,
}

impl World {

    pub async fn new() -> Self {

        
        let mut all_ids = Vec::with_capacity(20);

        // 0 it is id of engine
        // in case when engine send message to the some actor
        // sender property will be 0      
        all_ids.push(0);
        
        let (level, actors) = Level::download_level_from_server().await;

        log::info!("world system: level downloaded and init");

        let mut world = World {
            actors: HashMap::with_capacity(actors.len()),
            all_ids,
            level,
            main_camera_from: 0,
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
                        SpawnEffect(_) => {}
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

    // pub fn spawn_projectile(&mut self, projectile_type: ProjectileType, sender: ActorID) {

    // }

    pub fn add_actor_to_world(&mut self, mut actor: ActorWrapper) -> ActorID {

        let id = match actor.get_id() {
            Some(id) => id,
            None => {
                let id = self.make_new_unique_id_and_store_it();

                actor.init(id);

                id
            },
        };

        self.actors.insert(id, actor);

        id
    }

    pub fn remove_actor_from_world(&mut self, id: ActorID) -> Option<ActorWrapper> {

        self.actors.remove(&id)
    }

    // pub fn spawn_player_from_pool(&mut self, id: ActorID) -> Result<(), PlayerAccessError>{
    //     if self.actors.contains_key(&id) {
    //         self.all_ids.push(id);

    //         Ok(())
    //     } else {
    //         Err(PlayerAccessError::HaveNotPlayer)
    //     }
    // }

    // pub fn add_and_spawn_new_player(&mut self, master: InputMaster, player_settings: PlayerSettings) -> ActorID {
    //     let id = self.add_new_actor(master, player_settings);
    //     match self.spawn_player_from_pool(id) {
    //         Ok(()) => return id,
    //         Err(e) => panic!("in fn add_and_spawn_new_player after fn add_new_player have not player in pool")
    //     };
    // }

    pub fn tick(&mut self, engine_handle: &mut EngineHandle) {
        for (_, actor) in self.actors.iter_mut() {
            actor.tick(engine_handle)
        }
    }

    fn make_new_unique_id_and_store_it(&mut self) -> ActorID {
        if let Some(last_id) = self.all_ids.last() {
            if *last_id < u64::MAX {
                let new_id = last_id + 1;

                self.all_ids.push(new_id);

                new_id
            } else {
                panic!("in world system in all_ids last value is maximum of u64 type")
            }
        } else {
            panic!("in world system in all_ids buffer have no any value")
        }
    }
}


// fn make_random_id() -> u64 {
//     let mut bytes: [u8; 8] = [0_u8; 8];

//     if let Err(err) = getrandom::getrandom(&mut bytes) {
//         panic!("getrandom error, error code is {}", err);
//     }
//     u64::from_be_bytes(bytes)
// }

// fn is_id_unique() -> bool {

// }
