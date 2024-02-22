mod systems;

use wasm_bindgen::prelude::*;

use systems::{
    engine::Engine,
    main_loop::MainLoop,
};  

use log;

#[wasm_bindgen(start)]
async fn client_main() {

    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    

    console_log::init_with_level(log::Level::Info).expect("Could't initialize logger");

    let main_loop = MainLoop::new();
    
    log::info!("main: main_loop init");

    let systems = Engine::new(&main_loop).await;
    
    log::info!("main: Engine systems init");

    main_loop.run(systems).await;
}