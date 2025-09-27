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

#![windows_subsystem = "windows"]

mod engine;
mod actor;
mod main_loop;
mod transform;
mod read_args;
mod winsparkle;

use main_loop::MainLoop;
use pollster;

use actor::{flag::Flag, main_player::{player_input_master::{InputMaster, LocalMaster}, MainPlayer, PlayerMessage}, session_controller::{self, SessionController}, ActorWrapper, Message, SpecificActorMessage};
use client_server_protocol::Team;
use engine::input::ActionsFrameState;

use crate::{actor::flag_base::FlagBase, read_args::read_args};


use blink_alloc::GlobalBlinkAlloc;
#[global_allocator]
static GLOBAL_ALLOC: GlobalBlinkAlloc = GlobalBlinkAlloc::new();

#[cfg(target_os = "windows")]
fn init_winsparkle() {
    winsparkle::init();
}

#[cfg(not(target_os = "windows"))]
fn init_winsparkle() {
}

// This is pre-alpha demo version

fn main() {
    env_logger::init();

    init_winsparkle();

    let specific_backend = read_args();

    let main_loop = MainLoop::new();
    
    log::info!("main: main_loop init");

    pollster::block_on(main_loop.run(
        true,
        false,
        // If you made any changes to the game map, you should
        // run raymarch_shader_generator binary to generate a 
        // relevant raymarch shader with a BSP tree before creating the Engine.
        // Unless you see the previous version of the map.
        true,
        specific_backend,
        Box::new(|systems| {
            
            let main_player = MainPlayer::new(
                InputMaster::LocalMaster(
                    LocalMaster::new(ActionsFrameState::empty())
                ),
                systems.world.players_settings.clone(),
                &mut systems.audio,
                systems.world.level.blue_base_position,
                systems.world.level.red_base_position,
            );

            let main_player_id = systems.world.add_main_actor_to_world(
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
    })));
}