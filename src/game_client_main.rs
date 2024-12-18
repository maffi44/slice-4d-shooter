mod engine;
mod actor;
mod main_loop;
mod transform;

use engine::Engine;
use main_loop::MainLoop;
use pollster;
use blink_alloc::UnsafeGlobalBlinkAlloc;

#[global_allocator]
static GLOBAL_ALLOC: UnsafeGlobalBlinkAlloc = unsafe {
    UnsafeGlobalBlinkAlloc::new()
};

fn main() {
    env_logger::init();

    let main_loop = MainLoop::new();
    
    log::info!("main: main_loop init");

    let systems = pollster::block_on(Engine::new(&main_loop));
    
    log::info!("main: Engine systems init");

    pollster::block_on(main_loop.run(systems));
}