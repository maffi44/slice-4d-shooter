mod map;

use std::collections::HashMap;


use super::{
    actor::{
        Message,
        ActorID,
        player::{
            player_input_master::InputMaster,
            player_settings::PlayerSettings,
            Player,
        },
    },
    engine_handle::EngineHandle,
    projectiles::ProjectileType,
    engine_handle::CommandType::{
        SendMessage,
        SpawnEffect,
        SpawnProjectile,
    },
    static_obj::StaticObject,
};

use getrandom;
use glam::Vec4;


pub enum PlayerAccessError {
    HaveNotPlayer
}

pub struct World {
    pub pool_of_players: HashMap<ActorID, Player>,
    pub spawned_players: Vec<ActorID>,
    pub main_camera_from: ActorID,
    pub static_objects: Vec<StaticObject>,
    pub spawn_position: Vec4,
    // fx_pool
    // devices_pool
    // projectiles_pool
}


// use wasm_bindgen::prelude::*;
// use web_sys::{Request, RequestInit, RequestMode, Response};

impl World {

    pub async fn new() -> Self {

        let (static_objects, spawn_position) = map::load_map().await;

        World {
            pool_of_players: HashMap::with_capacity(2),
            // pool_of_rockets: Vec::
            spawned_players: Vec::with_capacity(2),
            main_camera_from: 0,
            static_objects,
            spawn_position,
        }
    }

    pub fn process_commands(&mut self, engine_handle: &mut EngineHandle) {
        if let Some(command) = engine_handle.command_buffer.pop() {
            let from = command.sender;
            
            match command.command_type {
                SendMessage(to, message) => {
                    self.send_message_to_player(
                        from,
                        to,
                        message,
                        engine_handle
                    );
                }
                SpawnEffect(_) => {}
                SpawnProjectile(_) => {}
            }
        }
    }

    pub fn send_message_to_player(
        &mut self,
        from: ActorID,
        to: ActorID,
        message: Message,
        engine_handle: &mut EngineHandle
    ) {
        if let Some(player) = self.pool_of_players.get_mut(&to) {
            player.recieve_message(from, message, engine_handle)
        }
    }

    pub fn spawn_projectile(&mut self, projectile_type: ProjectileType, sender: ActorID) {

    }

    pub fn add_new_player(&mut self, master: InputMaster, player_settings: PlayerSettings) -> ActorID {

        let mut id: ActorID = make_random_id();

        while self.pool_of_players.contains_key(&id) {
            id = make_random_id();
        }

        let new_player = Player::new(id, master, player_settings);

        self.pool_of_players.insert(id, new_player);

        id
    }

    pub fn spawn_player_from_pool(&mut self, id: ActorID) -> Result<(), PlayerAccessError>{
        if self.pool_of_players.contains_key(&id) {
            self.spawned_players.push(id);

            Ok(())
        } else {
            Err(PlayerAccessError::HaveNotPlayer)
        }
    }

    pub fn add_and_spawn_new_player(&mut self, master: InputMaster, player_settings: PlayerSettings) -> ActorID {
        let id = self.add_new_player(master, player_settings);
        match self.spawn_player_from_pool(id) {
            Ok(()) => return id,
            Err(e) => panic!("in fn add_and_spawn_new_player after fn add_new_player have not player in pool")
        };
    }

    pub fn process_input(&mut self, engine_handle: &mut EngineHandle) {
        for player_id in self.spawned_players.iter() {
            let player = self.pool_of_players.get_mut(player_id);

            if let Some(player) = player {
                player.process_input(engine_handle);
            }
        }
    }
}


fn make_random_id() -> u64 {
    let mut bytes: [u8; 8] = [0_u8; 8];

    if let Err(err) = getrandom::getrandom(&mut bytes) {
        panic!("getrandom error, error code is {}", err);
    }
    u64::from_be_bytes(bytes)
}

// fn is_id_unique() -> bool {

// }
