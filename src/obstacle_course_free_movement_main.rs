// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2025  Timofei Molokov

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

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

#[cfg(target_os = "windows")]
fn init_winsparkle() {
    winsparkle::init();
}

#[cfg(not(target_os = "windows"))]
fn init_winsparkle() {
}

// This is pre-alpha demo version

fn main() {
    env_logger::init();

    let specific_backend = read_args();

    let main_loop = MainLoop::new();
    
    log::info!("main: main_loop init");

    pollster::block_on(main_loop.run(
        "levels_for_puzzle/level-1".to_string(),
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