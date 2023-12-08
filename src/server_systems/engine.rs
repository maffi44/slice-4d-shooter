use crate::common_systems::{
    physics_system::PhysicsSystem,
    timeSystem::TimeSystem,
};

use crate::server_systems::{
    world_system::World,
};

pub struct Engine {
    // net: NetSystem,
    pub physics: PhysicsSystem,
    pub world: World,
    pub time: TimeSystem,
}

impl Engine {
    pub fn new() -> Self {
        // let net = NetSystem::new();

        let physics = PhysicsSystem::new();

        let world = World::new();

        let time = TimeSystem::new(60_u32);

        Engine {
            // net,
            physics,
            world,
            time,
        }
    }
}

