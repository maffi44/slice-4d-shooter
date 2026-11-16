// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2026  Timofei Molokov



use web_time::Instant;

use crate::engine::shared_systems::time::{clamp_delta, GlobalTimeSystem};


/// The simulation time of one World.
///
/// A World can be simulated every engine tick, every Nth tick or paused by its owner.
/// Its delta is the time elapsed since *its own* previous `simulate()`, so a world
/// simulated every 2nd tick still runs at real-time speed. For the root world
/// (simulated every tick) this equals the engine's previous frame duration.
pub struct WorldTimeSystem
{
    last_simulate_frame_start: Option<Instant>,
    delta: f32,
    simulate_counter: u64,
    world_time: f64,
}


impl WorldTimeSystem
{
    pub fn new() -> Self
    {
        WorldTimeSystem {
            last_simulate_frame_start: None,
            delta: 0.0,
            simulate_counter: 0,
            world_time: 0.0,
        }
    }

    /// Called first in `World::simulate`.
    pub fn begin_simulate(&mut self, global: &GlobalTimeSystem)
    {
        let frame_start = global.frame_start();

        self.delta = match self.last_simulate_frame_start
        {
            Some(last) => clamp_delta(frame_start.duration_since(last).as_secs_f32()),
            None => global.prev_frame_duration(),
        };

        self.last_simulate_frame_start = Some(frame_start);
        self.simulate_counter += 1;
        self.world_time += self.delta as f64;
    }

    /// Time elapsed since this world's previous simulate (seconds, clamped).
    #[inline]
    pub fn delta(&self) -> f32
    {
        self.delta
    }

    /// How many times this world has been simulated.
    #[inline]
    #[allow(dead_code)] // world time API, not used by the game yet
    pub fn simulate_counter(&self) -> u64
    {
        self.simulate_counter
    }

    /// Simulated seconds since the world was created.
    #[inline]
    #[allow(dead_code)] // world time API, not used by the game yet
    pub fn world_time(&self) -> f64
    {
        self.world_time
    }
}
