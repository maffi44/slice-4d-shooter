mod common_systems;
mod server_systems;
mod assets;

use common_systems::actions::Actions;
use server_systems::{
    engine::Engine,
    engine_handle::{
        EngineHandle,
        Command,
        CommandType,
    },
};


fn main() {
    let mut systems = Engine::new();
    systems.time.before_starting_main_loop();

    let mut engine_handle = EngineHandle::new();

    loop {
        systems.time.start_of_frame();

        // get input from client by net stage
        // todo - make normal get input from net
        // for each player take individual input or predict it
        let input = Actions::new();// <- TEMP

        // process input stage
        systems.world.process_input(input, &mut engine_handle);

        // excute command stage
        excute_commands(&mut systems, &mut engine_handle);

        //physics stage
        // TODO physics stage

        //post_physics_stage

        //send world state to players stage
        
        systems.time.end_of_frame();
    }
}

#[inline]
fn excute_commands(systems: &mut Engine, engine_handle: &mut EngineHandle) {
    while let Some(command) = engine_handle.command_buffer.pop() {
        let sender = command.sender;
        match command.command_type {
            CommandType::SpawnProjectile(t) => {
                
                systems.world.spawn_projectile(t, sender);
            },
            CommandType::SendMessage(to, message) => {
                systems.world.send_message_to_player(sender, to, message, engine_handle)
            },
            CommandType::SpawnEffect(effect) => {},
        }
    }
}

// fn main() {
//     pollster::block_on(server_main());
// }

// async fn server_main() {
//     let mut systems = Engine::new();
//     systems.time.before_starting_main_loop();

//     loop {
//         systems.time.start_of_frame();
        
//         systems.time.end_of_frame();
//     }
// }
    