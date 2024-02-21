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
    
    console_log::init_with_level(log::Level::Warn).expect("Could't initialize logger");

    let main_loop = MainLoop::new();
    
    log::warn!("Pre systems init");

    let systems = Engine::new(&main_loop).await;
    
    // log::warn!("Engine ready to start main loop");


    main_loop.run(systems).await;
}