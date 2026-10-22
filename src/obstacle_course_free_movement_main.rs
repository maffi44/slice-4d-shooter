// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2026  Timofei Molokov



#![windows_subsystem = "windows"]

mod engine;
mod actor;
mod main_loop;
mod transform;
mod read_args;
mod winsparkle;

use main_loop::MainLoop;
use pollster;

use actor::{flag::Flag, main_player::{player_input_master::{InputMaster, LocalMaster}, MainPlayer, PlayerMessage}, session_controller::{self, SessionController}, ActorWrapper, Message, SpecificActorMessage};
use client_server_protocol::Team;
use engine::input::ActionsFrameState;

use crate::{actor::{flag_base::FlagBase, obstacle_course_free_movement_player::ObstacleCourseFreeMovementPlayer, obstacle_course_player_two_jumps::ObstacleCoursePlayerTwoJumps}, read_args::read_args};

#[cfg(not(debug_assertions))]
use blink_alloc::GlobalBlinkAlloc;

#[cfg(not(debug_assertions))]
#[global_allocator]
static GLOBAL_ALLOC: GlobalBlinkAlloc = GlobalBlinkAlloc::new();

fn init_winsparkle() {
    #[cfg(target_os = "windows")]
    winsparkle::init();
}

fn main() {
    env_logger::init();

    let (specific_backend, specific_level) = read_args();

    let start_level = specific_level.unwrap_or("obstacle_course_levels/level-1".to_string());
 
    let main_loop = MainLoop::new();
    
    log::info!("main: main_loop init");

    pollster::block_on(main_loop.run(
        start_level,
        true,
        false,
        // If you made any changes to the game map, you should
        // run raymarch_shader_generator binary to generate a 
        // relevant raymarch shader with a BSP tree before creating the Engine.
        // Unless you see the previous version of the map.
        specific_backend,
        true,
    ));
}