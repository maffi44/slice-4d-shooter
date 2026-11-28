// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2026  Timofei Molokov



pub mod audio;
pub mod global_settings;
pub mod input;
pub mod net;
pub mod resources;
pub mod render;
pub mod time;

use self::{
    audio::AudioSystem,
    global_settings::GlobalSettings,
    input::InputSystem,
    net::NetSystem,
    render::{GpuContext, RenderScheduler},
    time::GlobalTimeSystem,
};


/// The systems that exist once per application and are shared by every World.
///
/// Every actor callback receives `&mut EngineSharedSystems` (together with its own `WorldContext`),
/// so any actor of any world reaches input, audio, net, settings and the GPU directly,
/// and an actor that owns a nested World passes them on to its `simulate()`.
pub struct EngineSharedSystems
{
    pub global_settings: GlobalSettings,
    pub time: GlobalTimeSystem,
    pub input: InputSystem,
    pub net: NetSystem,
    pub resources: resources::ResourceLoader<'static>,
    pub audio: AudioSystem,
    /// GPU infrastructure for creating GPU resources. `None` in headless mode.
    pub gpu: Option<GpuContext>,
    /// The render loop (own OS thread). `None` in headless mode.
    pub render: Option<RenderScheduler>,
    /// Async tasks: network sockets, async loading. Declared last, so it's dropped last.
    pub runtime: tokio::runtime::Runtime,
}


impl EngineSharedSystems
{
    /// Called on the loading thread; the loader attaches GPU/rendering before publishing the result.
    pub fn new(headless: bool, runtime: tokio::runtime::Runtime, global_settings: GlobalSettings, resources: resources::ResourceLoader<'static>) -> Self
    {
        log::info!("engine shared systems: global settings init");

        let time = GlobalTimeSystem::new(60_u32);
        log::info!("engine shared systems: time init");

        let input = InputSystem::new();
        log::info!("engine shared systems: input init");

        let mut audio = AudioSystem::new(headless);
        audio.master_volume = global_settings.user.master_volume;
        log::info!("engine shared systems: audio init");

        let net = NetSystem::new(runtime.handle().clone());
        log::info!("engine shared systems: net init");

        EngineSharedSystems {
            resources,
            global_settings,
            time,
            input,
            net,
            audio,
            gpu: None,
            render: None,
            runtime,
        }
    }


    /// Before the root world's simulate.
    pub fn pre_simulate(&mut self)
    {
        self.input.collect_gamepad_button_input();
        self.time.start_of_frame();
        self.input.update_frame_actions();
        self.net.tick();
        self.audio.update();
    }


    /// After the root world's simulate.
    pub fn post_simulate(&mut self)
    {
        self.net.flush();


        self.input.reset_input();
        self.time.end_of_frame();
    }
}
