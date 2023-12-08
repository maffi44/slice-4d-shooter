use std::{collections::HashMap};

use indexmap::IndexMap;
// use windows_sys::Win32::Foundation::E_MBN_NOT_REGISTERED;

use crate::common_systems::actions::Actions;
// use crate::systems::engineHandle::EngineHandle;

use super::{
    player::{Player, PlayerID, Message},
    engine_handle::EngineHandle,
    projectiles::ProjectileType,
};

use rand;


enum PlayerAccessError {
    HaveNotPlayer
}


pub struct World {
    pool_of_players: HashMap<PlayerID, Player>,
    spawned_players: Vec<PlayerID>,
    // fx_pool
    // devices_pool
    // projectiles_pool
}

impl World {

    pub fn new() -> Self {
        World {
            pool_of_players: HashMap::with_capacity(20),
            // pool_of_rockets: Vec::
            spawned_players: Vec::with_capacity(100),
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

    pub fn add_new_player(&mut self) -> PlayerID {

        let mut id: PlayerID = rand::random();

        while self.pool_of_players.contains_key(&id) {
            id = rand::random();
        }

        let new_player = Player::new(id);

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

    pub fn add_and_spawn_new_player(&mut self) -> PlayerID {
        let id = self.add_new_player();
        match self.spawn_player_from_pool(id) {
            Ok(()) => return id,
            Err(e) => panic!("in fn add_and_spawn_new_player after fn add_new_player have not player in pool")
        };
    }

    pub fn process_input(&mut self, mut input: Actions, engine_handle: &mut EngineHandle) {
        for player_id in self.spawned_players.iter() {
            let player = self.pool_of_players.get_mut(player_id);

            if let Some(player) = player {
                player.process_input(&mut input, engine_handle);
            }
        }
    }
}

