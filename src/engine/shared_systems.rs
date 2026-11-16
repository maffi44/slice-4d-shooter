// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2026  Timofei Molokov



pub mod audio;
pub mod global_settings;
pub mod input;
pub mod net;
pub mod render;
pub mod time;
pub mod ui;

use std::sync::Arc;

use wgpu::Backend;
use winit::window::Window;

use self::{
    audio::AudioSystem,
    global_settings::GlobalSettings,
    input::InputSystem,
    net::NetSystem,
    render::{GpuContext, RenderScheduler},
    time::GlobalTimeSystem,
    ui::UISystem,
};


/// The systems that exist once per application and are shared by every World.
///
/// Every actor callback receives `&mut EngineSharedSystems` (together with its own `WorldContext`),
/// so any actor of any world reaches input, audio, UI, net, settings and the GPU directly,
/// and an actor that owns a nested World passes them on to its `simulate()`.
pub struct EngineSharedSystems
{
    pub global_settings: GlobalSettings,
    pub time: GlobalTimeSystem,
    pub input: InputSystem,
    pub net: NetSystem,
    pub audio: AudioSystem,
    pub ui: UISystem,
    /// GPU infrastructure for creating GPU resources. Created when the window is created at startup
    /// (`create_gpu_and_render`); `None` before that and in headless mode.
    pub gpu: Option<GpuContext>,
    /// The render loop (own OS thread). `None` before the window is created and in headless mode.
    pub render: Option<RenderScheduler>,
    /// Async tasks: network sockets, async loading. Declared last, so it's dropped last.
    pub runtime: tokio::runtime::Runtime,
}


impl EngineSharedSystems
{
    /// Everything except the GPU and the render loop: they need the window, which winit lets us
    /// create only inside the running event loop (see `create_gpu_and_render`).
    /// `headless`: no sound device (and the engine never creates a window, GPU or render loop).
    pub fn new(headless: bool) -> Self
    {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .expect("ERROR: can't build the tokio async runtime");

        let global_settings = GlobalSettings::load();
        log::info!("engine shared systems: global settings init");

        let time = GlobalTimeSystem::new(60_u32);
        log::info!("engine shared systems: time init");

        let ui = UISystem::new();

        let input = InputSystem::new();
        log::info!("engine shared systems: input init");

        let mut audio = AudioSystem::new(headless);
        audio.master_volume = global_settings.user.master_volume;
        log::info!("engine shared systems: audio init");

        let net = NetSystem::new(runtime.handle().clone());
        log::info!("engine shared systems: net init");

        EngineSharedSystems {
            global_settings,
            time,
            input,
            net,
            audio,
            ui,
            gpu: None,
            render: None,
            runtime,
        }
    }


    /// Creates the GPU context and the render loop for the window. Called once, at startup.
    pub fn create_gpu_and_render(&mut self, window: Arc<Window>, graphic_backend: Option<Backend>)
    {
        let (gpu, surface, surface_config) = pollster::block_on(GpuContext::new(&window, graphic_backend));
        log::info!("engine shared systems: gpu context init");

        let render = RenderScheduler::new(
            &gpu,
            surface,
            surface_config,
            window,
            &mut self.ui,
        );
        log::info!("engine shared systems: render scheduler init");

        self.gpu = Some(gpu);
        self.render = Some(render);
    }


    /// Before the root world's simulate.
    pub fn pre_simulate(&mut self)
    {
        self.input.collect_gamepad_button_input();
        self.time.start_of_frame();
        self.input.update_frame_actions();
        self.net.tick();
    }


    /// After the root world's simulate.
    pub fn post_simulate(&mut self)
    {
        self.net.flush();

        if let (Some(gpu), Some(render)) = (&self.gpu, &self.render)
        {
            self.ui.write_buffers_ui(&gpu.queue, render.surface_aspect());
        }

        self.input.reset_input();
        self.time.end_of_frame();
    }
}
