mod engine;
mod actor;
mod main_loop;
mod transform;
mod read_args;

use main_loop::MainLoop;
use pollster;
use blink_alloc::UnsafeGlobalBlinkAlloc;

use actor::{flag::Flag, main_player::{player_input_master::{InputMaster, LocalMaster}, PlayerMessage}, player_for_2d_3d_example::PlayerFor2d3dExample, session_controller::{self, SessionController}, ActorWrapper, Message, SpecificActorMessage};
use client_server_protocol::Team;
use engine::input::ActionsFrameState;

use crate::{actor::flag_base::FlagBase, read_args::read_args};


#[global_allocator]
static GLOBAL_ALLOC: UnsafeGlobalBlinkAlloc = unsafe {
    UnsafeGlobalBlinkAlloc::new()
};

fn main() {
    env_logger::init();

    let specific_backend = read_args();

    let main_loop = MainLoop::new();
    
    log::info!("main: main_loop init");

    pollster::block_on(main_loop.run(
        false,
        true,
        false,
        specific_backend,
        Box::new(|systems| {

            let main_player = PlayerFor2d3dExample::new(
                InputMaster::LocalMaster(
                    LocalMaster::new(ActionsFrameState::empty())
                ),
                systems.world.players_settings.clone(),
                &mut systems.audio,
                systems.world.level.blue_base_position,
                systems.world.level.red_base_position,
            );

            let main_player_id = systems.world.add_main_actor_to_world(
                ActorWrapper::PlayerFor2d3dExample(main_player),
                &mut systems.engine_handle,
            );

            systems.engine_handle.send_boardcast_message(
                Message {
                    from: 0u128,
                    remote_sender: false,
                    message: crate::actor::MessageType::SpecificActorMessage(
                        SpecificActorMessage::PlayerMessage(
                            PlayerMessage::SetNewTeam(
                                session_controller::DEFAULT_TEAM
                            )
                        )
                    )
                }
            );

            let red_flag_base = FlagBase::new(
                Team::Red,
                systems.world.level.red_flag_base
            );

            let blue_flag_base = FlagBase::new(
                Team::Blue,
                systems.world.level.blue_flag_base
            );

            systems.world.add_actor_to_world(
                ActorWrapper::FlagBase(red_flag_base),
                &mut systems.engine_handle,
            );

            systems.world.add_actor_to_world(
                ActorWrapper::FlagBase(blue_flag_base),
                &mut systems.engine_handle,
            );

            let red_flag = Flag::new(
                Team::Red,
                systems.world.level.red_flag_base
            );

            systems.world.add_actor_to_world(
                ActorWrapper::Flag(red_flag),
                &mut systems.engine_handle,
            );

            let blue_flag = Flag::new(
                Team::Blue,
                systems.world.level.blue_flag_base
            );

            systems.world.add_actor_to_world(
                ActorWrapper::Flag(blue_flag),
                &mut systems.engine_handle,
            );

            let session_controller = SessionController::new(
                &mut systems.ui,
                systems.world.level.red_flag_base.get_position(),
                systems.world.level.blue_flag_base.get_position(),
                false,
            );
            
            systems.world.add_actor_to_world(
                ActorWrapper::SessionController(session_controller),
                &mut systems.engine_handle,
            );
    })));
}