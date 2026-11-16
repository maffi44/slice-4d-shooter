// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2026  Timofei Molokov



use std::time::{Duration, SystemTime};
use web_time::Instant;


/// If the previous engine frame took longer than this (a hitch, a debugger pause, a long load),
/// the delta falls back to one nominal frame so that the simulation doesn't make a huge step.
pub const MAX_DELTA: f32 = 0.35;
pub const FALLBACK_DELTA: f32 = 0.0166666;


/// Global time information, part of the Engine Shared Systems.
/// Not tied to any World: engine tick timing, counters, application start, wall clock.
/// Per-world simulation time lives in `WorldTimeSystem`; game-session (server) time lives in `SessionController`.
pub struct GlobalTimeSystem
{
    pub target_frame_duration: Duration,
    app_start: Instant,
    frame_start: Instant,
    prev_frame_duration: f32,
    frame_counter: u64,
    average_frame_duration: f64,
    current_frame_duration: f64,
}


impl GlobalTimeSystem
{
    pub fn new(target_frame_rate: u32) -> Self
    {
        GlobalTimeSystem {
            target_frame_duration: Duration::from_secs_f64(1_f64 / target_frame_rate as f64),
            app_start: Instant::now(),
            frame_start: Instant::now(),
            prev_frame_duration: 0.0,
            frame_counter: 0,
            average_frame_duration: 0.0,
            current_frame_duration: 0.0,
        }
    }

    /// Called once when the main loop starts.
    pub fn init(&mut self)
    {
        self.frame_start = Instant::now();
        self.app_start = Instant::now();
        self.frame_counter = 0;
    }

    #[inline]
    pub fn start_of_frame(&mut self)
    {
        self.average_frame_duration =
            self.app_start.elapsed().as_secs_f64() / self.frame_counter.max(1) as f64;

        self.prev_frame_duration = self.frame_start.elapsed().as_secs_f32();
        self.frame_start = Instant::now();
    }

    #[inline]
    pub fn end_of_frame(&mut self)
    {
        self.frame_counter += 1;
        self.current_frame_duration = self.frame_start.elapsed().as_secs_f64();
    }

    /// Timestamp of the start of the current engine tick.
    /// All worlds simulated in one tick use it, so they share one clock.
    #[inline]
    pub fn frame_start(&self) -> Instant
    {
        self.frame_start
    }

    /// Duration of the previous engine tick, clamped (see `MAX_DELTA`).
    pub fn prev_frame_duration(&self) -> f32
    {
        clamp_delta(self.prev_frame_duration)
    }

    /// Number of finished engine ticks since start.
    #[allow(dead_code)] // time/statistics API, not used by the game yet
    pub fn frame_counter(&self) -> u64
    {
        self.frame_counter
    }

    pub fn time_since_start(&self) -> Duration
    {
        self.app_start.elapsed()
    }

    #[allow(dead_code)] // time/statistics API, not used by the game yet
    pub fn average_frame_duration(&self) -> f64
    {
        self.average_frame_duration
    }

    #[allow(dead_code)] // time/statistics API, not used by the game yet
    pub fn current_frame_duration(&self) -> f64
    {
        self.current_frame_duration
    }

    /// Current date and time (wall clock).
    #[allow(dead_code)] // time/statistics API, not used by the game yet
    pub fn now_utc(&self) -> SystemTime
    {
        SystemTime::now()
    }
}


#[inline]
pub fn clamp_delta(delta: f32) -> f32
{
    if delta > MAX_DELTA
    {
        FALLBACK_DELTA
    }
    else
    {
        delta
    }
}
