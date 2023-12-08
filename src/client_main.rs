mod common_systems;
mod client_systems;
mod assets;

use client_systems::{
    engine::Engine,
    main_loop::MainLoop,
};
use pollster;

fn main() {
    pollster::block_on(client_main());
}


async fn client_main() {
    let main_loop = MainLoop::new();

    let systems = Engine::new(&main_loop).await;
    
    // let runtime = runtimeSystem::RuntimeSystem::new(); 

    // systems.net.connect_to_server();
    main_loop.run(systems).await;
}