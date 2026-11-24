// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2026  Timofei Molokov



#![windows_subsystem = "windows"]

mod engine;
mod actor;
mod read_args;
mod winsparkle;

use engine::main_loop::MainLoop;

use crate::read_args::read_args;

#[cfg(not(debug_assertions))]
use blink_alloc::GlobalBlinkAlloc;

#[cfg(not(debug_assertions))]
#[global_allocator]
static GLOBAL_ALLOC: GlobalBlinkAlloc = GlobalBlinkAlloc::new();

fn main() -> Result<(), String> {
    env_logger::init();

    #[cfg(target_os = "windows")]
    winsparkle::init();

    let args = read_args();

    let start_level = args.specific_level.unwrap_or("capture_the_flag_map".to_string());

    let loading = engine::load(start_level, args.graphic_backend, args.headless);

    MainLoop::run(loading)
}
