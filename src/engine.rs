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

pub mod render;
pub mod input;
pub mod net;
pub mod time;
pub mod physics;
pub mod effects;
pub mod world;
pub mod engine_handle;
pub mod audio;
pub mod ui;
pub mod settings;

#[cfg(target_arch = "wasm32")]
use std::{
    future::Future,
    pin::Pin,
    rc::Rc,
    task::{
        Context,
        Poll
    }
};

use crate::{
    actor::{
        Actor,
        ActorID,
        ActorWrapper,
        Message,
        MessageType,
        SpecificActorMessage,
        main_player::{
            PlayerMessage,
            player_settings::PlayerSettings
        }
    },
    engine::{
        engine_handle::{
            Command,
            CommandType
        },
        world::level::Level
    }
};

use self::{
    render::RenderSystem,
    input::InputSystem,
    physics::PhysicsSystem,
    time::TimeSystem,
    world::World,
    engine_handle::EngineHandle,
    net::NetSystem,
    audio::AudioSystem,
    ui::UISystem
};

use client_server_protocol::NetCommand;
use effects::EffectsSystem;
// use winit::window::WindowBuilder;

use settings::Settings;
use tokio::{runtime::Runtime, task::JoinHandle};
use wgpu::Backend;
#[cfg(target_arch = "wasm32")]
use winit::platform::web::WindowBuilderExtWebSys;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use winit::window::Window;

pub struct Engine {
    pub render: RenderSystem,
    pub input: InputSystem,
    pub physic: PhysicsSystem,
    pub time: TimeSystem,
    pub world: World,
    pub engine_handle: EngineHandle,
    pub net: NetSystem,
    pub audio: AudioSystem,
    pub ui: UISystem,
    pub effects: EffectsSystem,
    pub settings: Settings,
    pub level_async_load_handler: Option<JoinHandle<Level>>,

    #[cfg(not(target_arch = "wasm32"))]
    pub runtime: tokio::runtime::Runtime,
}

#[cfg(target_arch = "wasm32")]
struct WindowReadyFuture<'a> {
    window: &'a winit::window::Window
}
#[cfg(target_arch = "wasm32")]
impl<'a> Future for WindowReadyFuture<'a> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.window.inner_size().width == 0 || self.window.inner_size().height == 0 {
            let wait_fn = {
                let waker = Rc::new(cx.waker().clone());
                Closure::wrap(Box::new(move || {
                    waker.as_ref().clone().wake();
                }) as Box<dyn Fn()>)
            };
            let _ = web_sys::window()
                .unwrap()
                .set_timeout_with_callback_and_timeout_and_arguments_0(
                    wait_fn.as_ref().unchecked_ref(),
                    50,
                );
            wait_fn.forget();

            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }
}

impl Engine {
    pub async fn new(
        window: Window,
        start_level: String,
        with_ui_renderer: bool,
        it_is_2d_3d_example: bool,
        disable_net_system: bool,
        specific_backend: Option<Backend>,
    ) -> Engine
    {
        #[cfg(not(target_arch = "wasm32"))]
        let mut runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .expect("Can't build tokio async runtime");

        #[cfg(target_arch = "wasm32")]
        {
            let document = web_sys::window().unwrap().document().unwrap();
            let canvas = document.get_element_by_id("game_canvas").unwrap();
            let canvas: web_sys::HtmlCanvasElement = JsValue::from(canvas).into();
    
            let window_builder = WindowBuilder::new();
            window = window_builder
            
                .with_canvas(Some(canvas))
                .with_active(true)
                .with_title("Slice: 4D Shooter")
                // .with_fullscreen(Some(winit::window::Fullscreen::Borderless(None)))
                // .with_inner_size(PhysicalSize::new(1200, 800))
                .build(&client_main_loop.event_loop)
                .unwrap();
        }

        let mut engine_handle = EngineHandle::new();
        log::info!("engine systems:engine_handle init");

        let global_players_settings = PlayerSettings::load_player_settings().await;
        log::info!("engine systems: global_players_settings init");
        
        let settings = Settings::new(global_players_settings.clone());

        let mut time = TimeSystem::new(60_u32);
        log::info!("engine systems: time init");

        let mut pre_initialized_ui = UISystem::new(); 

        let mut render = RenderSystem::new(
            window,
            &time,
            &mut pre_initialized_ui,
            #[cfg(not(target_arch = "wasm32"))]
            &mut runtime,
            with_ui_renderer,
            specific_backend,
        ).await;
        log::info!("engine systems: render init");

        let mut initialized_ui = pre_initialized_ui;

        let mut world = World::new(
            &mut engine_handle,
            global_players_settings,
            // start_level,
        ).await;
        log::info!("engine systems: world init");
        
        let mut physic = PhysicsSystem::new();
        log::info!("engine systems: physic init");

        let input = InputSystem::new();
        log::info!("engine systems: input init");

        let mut audio = AudioSystem::new(false).await;

        let mut effects = EffectsSystem::new();

        let net = NetSystem::new(
            &world.players_settings,
            it_is_2d_3d_example,
            disable_net_system,
            #[cfg(not(target_arch = "wasm32"))]
            &mut runtime
        ).await;
        log::info!("engine systems: net init");

        let level = {
            runtime.block_on(
                world::level::Level::load_level(
                    start_level,
                    world.players_settings.clone(),
                    Some(render.render_pipeline_builder_kit.clone()),

                )
            )
        };

        set_new_level(
            level,
            &mut engine_handle,
            &mut world,
            &mut Some(&mut render),
            &mut physic,
            &mut audio,
            &mut initialized_ui,
            &mut time,
            &mut effects,
        );

        Engine {
            physic,
            input,
            render,
            time,
            world,
            engine_handle,
            net,
            audio,
            ui: initialized_ui,
            effects,
            settings,
            level_async_load_handler: None,
            
            #[cfg(not(target_arch = "wasm32"))]
            runtime,
        }
    }
}


pub struct HeadlessEngine
{
    pub input: InputSystem,
    pub physic: PhysicsSystem,
    pub time: TimeSystem,
    pub world: World,
    pub engine_handle: EngineHandle,
    pub net: NetSystem,
    pub audio: AudioSystem,
    pub ui: UISystem,
    pub effects: EffectsSystem,
    pub settings: Settings,

    #[cfg(not(target_arch = "wasm32"))]
    pub runtime: tokio::runtime::Runtime,
}

impl HeadlessEngine
{
    pub async fn new(
        disable_net_system: bool,
    ) -> HeadlessEngine {

        #[cfg(not(target_arch = "wasm32"))]
        let mut runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .expect("Can't build tokio async runtime");

        let mut engine_handle = EngineHandle::new();
        log::info!("engine systems:engine_handle init");

        let global_players_settings = PlayerSettings::load_player_settings().await;
        log::info!("engine systems: global_players_settings init");
        
        let settings = Settings::new(global_players_settings.clone());

        let world = World::new(
            &mut engine_handle,
            global_players_settings,
            // "map".to_string()
        ).await;
        log::info!("engine systems: world init");
        
        let physic = PhysicsSystem::new();
        log::info!("engine systems: physic init");

        let input = InputSystem::new();
        log::info!("engine systems: input init");

        let time = TimeSystem::new(60_u32);
        log::info!("engine systems: time init");

        let ui = UISystem::new(); 

        let audio = AudioSystem::new(true).await;

        let effects = EffectsSystem::new();

        let net = NetSystem::new(
            &world.players_settings,
            false,
            disable_net_system,
            #[cfg(not(target_arch = "wasm32"))]
            &mut runtime
        ).await;
        log::info!("engine systems: net init");


        HeadlessEngine {
            physic,
            input,
            time,
            world,
            engine_handle,
            net,
            audio,
            ui,
            effects,
            settings,
            
            #[cfg(not(target_arch = "wasm32"))]
            runtime,
        }
    }
}

#[inline]
pub fn execute_command(
    command: Command,
    world: &mut World,
    net_system: &mut NetSystem,
    physics_system: &mut PhysicsSystem,
    engine_handle: &mut EngineHandle,
    audio_system: &mut AudioSystem,
    ui_system: &mut UISystem,
    time_system: &mut TimeSystem,
    render_system: &mut Option<&mut RenderSystem>,
    effects_system: &mut EffectsSystem,
    runtime: &mut Runtime,
    player_settings: PlayerSettings,
    level_async_load_handler: &mut Option<JoinHandle<Level>>
) {
    let from = command.sender;

    match command.command_type {
        CommandType::ShowConnectionStatusUI =>
        {
            net_system.set_is_visible_for_connection_status(true);
        }
        CommandType::RemoveAllHolesAndEffects =>
        {
            let mut keys_for_remove = Vec::new();

            for (key, actor) in world.actors.iter()
            {
                match actor {
                    ActorWrapper::Hole(_) =>
                    {
                        keys_for_remove.push(*key);
                    }

                    ActorWrapper::HoleGunMiss(_) =>
                    {
                        keys_for_remove.push(*key);
                    }

                    ActorWrapper::MachinegunShot(_) =>
                    {
                        keys_for_remove.push(*key);
                    }

                    ActorWrapper::PlayersDeathExplosion(_) =>
                    {
                        keys_for_remove.push(*key);
                    }

                    ActorWrapper::HoleGunShot(_) =>
                    {
                        keys_for_remove.push(*key);
                    }

                    ActorWrapper::ShootingImpact(_) =>
                    {
                        keys_for_remove.push(*key);
                    }

                    _ => {}
                }
            }

            for key in keys_for_remove
            {
                world.actors.remove(&key);
            }
        }
        CommandType::SpawnEffect(_) => {}
        CommandType::SpawnActor(actor) =>
        {
            world.add_actor_to_world(
                actor,
                engine_handle,
                physics_system,
                audio_system,
                ui_system,
                time_system,
                effects_system,
            );
        }
        CommandType::RemoveActor(id) =>
        {
            world.remove_actor_from_world(id);
        }
        CommandType::NetCommand(command) =>
        {
            match command {
                NetCommand::SetServerTime(server_time) => {
                    time_system.set_server_time(server_time);
                },
                NetCommand::NetSystemIsConnectedAndGetNewPeerID(new_id) => {
                    world.change_actor_id(world.main_actor_id, new_id, engine_handle);

                    world.main_actor_id = new_id;                       
                },
                NetCommand::SendBoardcastNetMessageReliable(message) => {
                    net_system.send_boardcast_message_reliable(message);
                },

                NetCommand::SendBoardcastNetMessageUnreliable(message) => {
                    net_system.send_boardcast_message_unreliable(message);
                },

                NetCommand::SendDirectNetMessageReliable(message, peer) => {
                    net_system.send_direct_message_reliable(message, peer);
                },

                NetCommand::SendDirectNetMessageUnreliable(message, peer) => {
                    net_system.send_direct_message_unreliable(message, peer);
                },

                NetCommand::PeerConnected(peer_id) => {
                    engine_handle.send_boardcast_message(
                        Message {
                            from: 0u128,
                            remote_sender: false,
                            message: MessageType::SpecificActorMessage(
                                SpecificActorMessage::PlayerMessage(
                                    PlayerMessage::NewPeerConnected(peer_id)
                                )
                            )
                        }
                    )
                },

                NetCommand::SendMessageToServer(message) =>
                {
                    net_system.send_message_to_game_server(message);
                }

                NetCommand::PeerDisconnected(id) => {
                    world.remove_actor_from_world(id);
                }
            }
        }

        CommandType::PreloadNewLevelAsync(level_name) => 
        {
            let render_pipeline_builder_kit =
            {
                if render_system.is_some()
                {
                    Some(render_system.as_ref().unwrap().render_pipeline_builder_kit.clone())
                }
                else
                {
                    None
                }
            };
            
            let preloaded_level_handle = runtime.spawn(
        Level::load_level(
                    level_name.clone(),
                    player_settings,
                    render_pipeline_builder_kit
                )
            );

            world.preloaded_levels.insert(level_name, preloaded_level_handle);
        },

        CommandType::LoadNewLevelSync(level_name) =>
        {
            let level= if world.preloaded_levels.contains_key(&level_name)
            {
                runtime.block_on(
                    world.preloaded_levels.remove(&level_name).unwrap()
                ).unwrap()
            }
            else
            {
                let render_pipeline_builder_kit =
                {
                    if render_system.is_some()
                    {
                        Some(render_system.as_ref().unwrap().render_pipeline_builder_kit.clone())
                    }
                    else
                    {
                        None
                    }
                };

                let level = {
                    runtime.block_on(
                        Level::load_level(
                            level_name,
                            player_settings,
                            render_pipeline_builder_kit
                        )
                    )
                };

                level
            };

            set_new_level(
                level,
                engine_handle,
                world,
                render_system,
                physics_system,
                audio_system,
                ui_system,
                time_system,
                effects_system,
            );
        },

        CommandType::LoadNewLevelAsync(level_name) => 
        {
            let level_task = if world.preloaded_levels.contains_key(&level_name)
            {
                world.preloaded_levels.remove(&level_name)
            }
            else
            {
                let render_pipeline_builder_kit =
                {
                    if render_system.is_some()
                    {
                        Some(render_system.as_ref().unwrap().render_pipeline_builder_kit.clone())
                    }
                    else
                    {
                        None
                    }
                };

                Some(runtime.spawn(
            Level::load_level(
                        level_name,
                        player_settings,
                        render_pipeline_builder_kit
                    )
                ))
            };

            *level_async_load_handler = level_task;
        },
    }
}

#[inline]
pub fn send_direct_messages(
    to: ActorID,
    message: Message,
    engine_handle: &mut EngineHandle,
    world: &mut World,
    physics_system: &PhysicsSystem,
    audio_system: &mut AudioSystem,
    ui_system: &mut UISystem,
    time_system: &TimeSystem,
    effects_system: &mut EffectsSystem,

) {
    if let Some(actor) = world.actors.get_mut(&to)
    {
        actor.recieve_message(
            message,
            engine_handle,
            physics_system,
            audio_system,
            ui_system,
            time_system,
            effects_system,
        );
    }
}

#[inline]
pub fn send_boardcast_messages(
    message: Message,
    engine_handle: &mut EngineHandle,
    world: &mut World,
    physics_system: &PhysicsSystem,
    audio_system: &mut AudioSystem,
    ui_system: &mut UISystem,
    time_system: &TimeSystem,
    effects_system: &mut EffectsSystem,
) {
    for (_, actor) in world.actors.iter_mut() {
        if actor.get_id().expect("actor does not have id") != message.from
        {
            actor.recieve_message
            (
                message.clone(),
                engine_handle,
                physics_system,
                audio_system,
                ui_system,
                time_system,
                effects_system
            );
        } 
    }
}

#[inline]
pub fn send_messages_and_process_commands(
    world: &mut World,
    net_system: &mut NetSystem,
    physics_system: &mut PhysicsSystem,
    audio_system: &mut AudioSystem,
    ui_system: &mut UISystem,
    engine_handle: &mut EngineHandle,
    time_system: &mut TimeSystem,
    effects_system: &mut EffectsSystem,
    mut render_system: Option<&mut RenderSystem>,
    runtime: &mut Runtime,
    level_async_load_handler: &mut Option<JoinHandle<Level>>
) {

    let player_settings = world.players_settings.clone();
    
    loop {
        while let Some(message) = engine_handle.boardcast_message_buffer.pop()
        {
            send_boardcast_messages(
                message,
                engine_handle,
                world,
                physics_system,
                audio_system,
                ui_system,
                time_system,
                effects_system,
            )                
        }

        while let Some((to, message)) = engine_handle.direct_message_buffer.pop()
        {
            send_direct_messages(
                to,
                message,
                engine_handle,
                world,
                physics_system,
                audio_system,
                ui_system,
                time_system,
                effects_system,
            )                
        }

        while let Some(command) = engine_handle.command_buffer.pop()
        {
            execute_command(
                command,
                world,
                net_system,
                physics_system,
                engine_handle,
                audio_system,
                ui_system,
                time_system,
                &mut render_system,
                effects_system,
                runtime,
                player_settings.clone(),
                level_async_load_handler,
            );
        }

        if engine_handle.direct_message_buffer.is_empty() &&
            engine_handle.boardcast_message_buffer.is_empty() &&
            engine_handle.command_buffer.is_empty()
        {   
            return;
        }
    }
}

pub fn set_new_level (
    level: Level,
    engine_handle: &mut EngineHandle,
    world: &mut World,
    render_system: &mut Option<&mut RenderSystem>,
    physics_system: &mut PhysicsSystem,
    audio_system: &mut AudioSystem,
    ui_system: &mut UISystem,
    time_system: &mut TimeSystem,
    effects_system: &mut EffectsSystem,
)
{
    world.set_new_level(
        level,
        engine_handle,
        physics_system,
        audio_system,
        ui_system,
        time_system,
        effects_system,
    );
 
    physics_system.set_new_level(world);

    if render_system.is_some()
    {
        (*render_system).as_mut().unwrap().set_new_level(world);
    }
}