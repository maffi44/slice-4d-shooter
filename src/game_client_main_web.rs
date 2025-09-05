// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2025  Timofei Molokov

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

mod engine;
mod actor;
mod main_loop;
mod transform;

use actor::{flag::Flag, main_player::{player_input_master::{InputMaster, LocalMaster}, MainPlayer, PlayerMessage}, session_controller::{self, SessionController}, ActorWrapper, Message, SpecificActorMessage};
use client_server_protocol::Team;
use engine::input::ActionsFrameState;
use main_loop::MainLoop;

use wasm_bindgen::prelude::*;
use log;

use crate::actor::flag_base::FlagBase;

// The web version is currently not working.
// Some times ago I stopped supporting the web version because it took too much time.
// Later, when I choose the final sdf render approach (and graphics backend along with it)
// I will restore web support.
#[wasm_bindgen(start)]
async fn client_main() {

    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    
    #[cfg(debug_assertions)]
    console_log::init_with_level(log::Level::Info).expect("Could't initialize logger");

    #[cfg(not(debug_assertions))]
    console_log::init_with_level(log::Level::Warn).expect("Could't initialize logger");

    let main_loop = MainLoop::new();
    
    log::info!("main: main_loop init");

    main_loop.run(true, false, false, None, Box::new(|systems| {
            let main_player = MainPlayer::new(
                InputMaster::LocalMaster(
                    LocalMaster::new(ActionsFrameState::empty())
                ),
                systems.world.players_settings.clone(),
                &mut systems.audio,
                systems.world.level.blue_base_position,
                systems.world.level.red_base_position,
            );
    
            let main_player_id = systems.world.add_actor_to_world(
                ActorWrapper::MainPlayer(main_player),
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
    
            systems.world.main_actor_id = main_player_id;
    })).await;
}