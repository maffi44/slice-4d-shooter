// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2026  Timofei Molokov



// The game client without the WinSparkle auto-updater (and with a console window on Windows).

mod engine;
mod actor;
mod read_args;

use engine::main_loop::MainLoop;

use crate::read_args::read_args;

#[cfg(not(debug_assertions))]
use blink_alloc::GlobalBlinkAlloc;

#[cfg(not(debug_assertions))]
#[global_allocator]
static GLOBAL_ALLOC: GlobalBlinkAlloc = GlobalBlinkAlloc::new();

fn main() -> Result<(), String> {
    env_logger::init();

    let args = read_args();

    let start_level = args.specific_level.unwrap_or("capture_the_flag_map".to_string());

    let loading = engine::load(start_level, args.graphic_backend, args.headless);

    MainLoop::run(loading)
}
