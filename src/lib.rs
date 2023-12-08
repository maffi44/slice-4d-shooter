mod common_systems;
mod client_systems;
mod assets;

use wasm_bindgen::prelude::*;

use client_systems::{
    engine::Engine,
    main_loop::MainLoop,
};
use pollster;

use wasm_bindgen::prelude::*;

// #[wasm_bindgen]
// extern "C" {
//     fn alert(s: &str);
// }

// #[wasm_bindgen]
// pub fn main() {
//     alert("start wgpu");
    
//     pollster::block_on(client_main());
// }

use log;

#[wasm_bindgen(start)]
async fn client_main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Warn).expect("Could't initialize logger");
    
    let main_loop = MainLoop::new();

    let systems = Engine::new(&main_loop).await;
    
    // let runtime = runtimeSystem::RuntimeSystem::new(); 

    // systems.net.connect_to_server();
    main_loop.run(systems).await;
}