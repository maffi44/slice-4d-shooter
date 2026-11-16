// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2026  Timofei Molokov



// The game client with a console window on Windows (no `windows_subsystem = "windows"`),
// e.g. to see the log output. The WinSparkle auto-updater is initialized.

mod engine;
mod actor;
mod read_args;
mod winsparkle;

use engine::Engine;

use crate::read_args::read_args;

#[cfg(not(debug_assertions))]
use blink_alloc::GlobalBlinkAlloc;

#[cfg(not(debug_assertions))]
#[global_allocator]
static GLOBAL_ALLOC: GlobalBlinkAlloc = GlobalBlinkAlloc::new();

fn main() {
    env_logger::init();

    #[cfg(target_os = "windows")]
    winsparkle::init();

    let args = read_args();

    let start_level = args.specific_level.unwrap_or("capture_the_flag_map".to_string());

    let engine = Engine::new(args.headless);

    engine.run(start_level, args.graphic_backend);
}
