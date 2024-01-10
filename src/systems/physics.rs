use super::world::World;

use glam::Vec4;

pub mod collisions;

pub struct PhysicsSystem {

}
pub struct PhysicsState {

}

impl PhysicsState {
    pub fn new() -> Self {
        PhysicsState {  }
    }

    pub fn ray_cast(&mut self, from: Vec4, direction: Vec4, len: f64) -> Option<Hit> {
        None
    }
}

pub struct Hit {
    pub hit_point: Vec4,
    pub hited_players_id: Option<u32>, 
    pub hit_normal: Vec4,
}


impl PhysicsSystem {
    pub fn new() -> Self {
        PhysicsSystem {
            
        }
    }

    pub fn process_physics(&mut self, world: &mut World, dt: f32) {
        for player_id in world.spawned_players.iter() {
            if let Some(player) = world.pool_of_players.get_mut(player_id) {
                player.get_mut_collider().physics_tick(dt, &world.map);
            }
        }
    }


}