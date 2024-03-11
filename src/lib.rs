mod engine;
mod actor;
mod main_loop;
mod transform;

use engine::Engine;
use main_loop::MainLoop;

use wasm_bindgen::prelude::*;
use log;

#[wasm_bindgen(start)]
async fn client_main() {

    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    
    #[cfg(debug_assertions)]
    console_log::init_with_level(log::Level::Info).expect("Could't initialize logger");

    #[cfg(not(debug_assertions))]
    console_log::init_with_level(log::Level::Warn).expect("Could't initialize logger");

    let main_loop = MainLoop::new();
    
    log::info!("main: main_loop init");

    let systems = Engine::new(&main_loop).await;
    
    log::info!("main: Engine systems init");

    main_loop.run(systems).await;
}