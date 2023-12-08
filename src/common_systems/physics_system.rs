use super::transform::{
    Position,
};

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

    pub fn simulate_physics() {

    }


}