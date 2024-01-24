mod map;

use std::collections::HashMap;


use super::{
    player::{
        player_input_master::InputMaster, player_settings::PlayerSettings, Message, Player, PlayerID
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
    pub pool_of_players: HashMap<PlayerID, Player>,
    pub spawned_players: Vec<PlayerID>,
    pub main_camera_from: PlayerID,
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
        from: PlayerID,
        to: PlayerID,
        message: Message,
        engine_handle: &mut EngineHandle
    ) {
        if let Some(player) = self.pool_of_players.get_mut(&to) {
            player.recieve_message(from, message, engine_handle)
        }
    }

    pub fn spawn_projectile(&mut self, projectile_type: ProjectileType, sender: PlayerID) {

    }

    pub fn add_new_player(&mut self, master: InputMaster, player_settings: PlayerSettings) -> PlayerID {

        let mut id: PlayerID = make_random_id();

        while self.pool_of_players.contains_key(&id) {
            id = make_random_id();
        }

        let new_player = Player::new(id, master, player_settings);

        self.pool_of_players.insert(id, new_player);

        id
    }

    pub fn spawn_player_from_pool(&mut self, id: PlayerID) -> Result<(), PlayerAccessError>{
        if self.pool_of_players.contains_key(&id) {
            self.spawned_players.push(id);

            Ok(())
        } else {
            Err(PlayerAccessError::HaveNotPlayer)
        }
    }

    pub fn add_and_spawn_new_player(&mut self, master: InputMaster, player_settings: PlayerSettings) -> PlayerID {
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


fn make_random_id() -> u32 {
    let mut bytes: [u8; 4] = [0_u8; 4];

    if let Err(err) = getrandom::getrandom(&mut bytes) {
        panic!("getrandom error, error code is {}", err);
    }
    u32::from_be_bytes(bytes)
}
