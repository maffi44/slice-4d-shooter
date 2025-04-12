mod engine;
mod actor;
mod main_loop;
mod transform;

use engine::Engine;
use main_loop::MainLoop;
use pollster;
use blink_alloc::UnsafeGlobalBlinkAlloc;

use actor::{flag::Flag, move_w_bonus::MoveWBonusSpot, main_player::{player_input_master::{InputMaster, LocalMaster}, MainPlayer, PlayerMessage}, player_for_2d_3d_example::PlayerFor2d3dExample, session_controller::{self, SessionController}, ActorWrapper, Message, SpecificActorMessage};
use client_server_protocol::Team;
use engine::input::ActionsFrameState;


#[global_allocator]
static GLOBAL_ALLOC: UnsafeGlobalBlinkAlloc = unsafe {
    UnsafeGlobalBlinkAlloc::new()
};

fn main() {
    env_logger::init();

    let main_loop = MainLoop::new();
    
    log::info!("main: main_loop init");

    let systems = pollster::block_on(
        Engine::new(
            &main_loop,
             true,
            )
        );
    
    log::info!("main: Engine systems init");

    pollster::block_on(main_loop.run(systems, Box::new(|systems| {

        let mut main_player = PlayerFor2d3dExample::new(
            InputMaster::LocalMaster(
                LocalMaster::new(ActionsFrameState::empty())
            ),
            systems.world.players_settings.clone(),
            &mut systems.audio,
            systems.world.level.w_levels.clone()
        );

        // let spawn = systems
        //     .world
        //     .level
        //     .get_random_spawn_position(main_player.get_team());

        // main_player.get_mut_transform().set_position(
        //     spawn.spawn_position
        // );

        // main_player.set_current_w_level(spawn.w_level);

        let main_player_id = systems.world.add_actor_to_world(
            ActorWrapper::PlayerFor2d3dExample(main_player),
            &mut systems.engine_handle,
        );

        systems.engine_handle.send_boardcast_message(
            Message {
                from: 0u128,
                message: crate::actor::MessageType::SpecificActorMessage(
                    SpecificActorMessage::PLayerMessage(
                        PlayerMessage::SetNewTeam(
                            session_controller::DEFAULT_TEAM
                        )
                    )
                )
            }
        );

        while let Some(mover_w) = systems.world.level.mover_w_list.pop()
        {
            systems.world.add_actor_to_world(
                ActorWrapper::MoverW(mover_w),
                &mut systems.engine_handle
            );
        } 

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
            systems.world.level.blue_flag_base.get_position()
        );
        
        systems.world.add_actor_to_world(
            ActorWrapper::SessionController(session_controller),
            &mut systems.engine_handle,
        );

        let move_w_bonus = MoveWBonusSpot::new(
            systems.world.level.move_w_bonus_spot,
            0
        );

        systems.world.add_actor_to_world(
            ActorWrapper::MoveWBonusSpot(move_w_bonus),
            &mut systems.engine_handle,
        );

        systems.world.main_player_id = main_player_id;
    })));
}