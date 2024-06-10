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

use std::sync::{Arc, Mutex};
#[cfg(target_arch = "wasm32")]
use std::{future::Future, pin::Pin, rc::Rc, task::{Context, Poll}};

use crate::{
    actor::player::player_settings::PlayerSettings, engine::audio::AudioSystem, main_loop::MainLoop
};

use self::{
    render::RenderSystem,
    input::InputSystem,
    physics::PhysicsSystem,
    time::TimeSystem,
    world::World,
    engine_handle::EngineHandle,
    net::NetSystem,
};

use egui::ViewportId;
use render::{render_data::RenderData, renderer::Renderer};
use tokio::runtime::Runtime;
use ui::UISystem;
use winit::window::{Window, WindowBuilder};

#[cfg(target_arch = "wasm32")]
use winit::platform::web::WindowBuilderExtWebSys;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{closure::Closure, JsCast, JsValue};

pub struct Engine {
    pub render: RenderSystem,
    pub input: InputSystem,
    pub physic: PhysicsSystem,
    pub time: TimeSystem,
    pub world: World,
    pub engine_handle: EngineHandle,
    pub net: NetSystem,
    #[cfg(not(target_arch = "wasm32"))]
    pub runtime: tokio::runtime::Runtime,
    pub audio: AudioSystem,
    pub ui: UISystem,
    // pub runtime: RuntimeSystem,
    // pub net: ClientNetSystem,
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
        cleint_main_loop: &MainLoop,
        // async_runtime: &Runtime,
    ) -> Engine {

        let window;

        #[cfg(not(target_arch = "wasm32"))]
        let mut runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
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
                // .with_fullscreen(Some(winit::window::Fullscreen::Borderless(None)))
                // .with_inner_size(PhysicalSize::new(1200, 800))
                .build(&cleint_main_loop.event_loop)
                .unwrap();
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let window_builder = WindowBuilder::new();
            window = window_builder
                .with_active(true)
                .with_fullscreen(Some(winit::window::Fullscreen::Borderless(None)))
                // .with_inner_size(PhysicalSize::new(1200, 800))
                .build(&cleint_main_loop.event_loop)
                .unwrap();
        }
        log::info!("engine systems: window init");

        #[cfg(target_arch = "wasm32")]
        {
            // it is necessary because immidiatly after creating the window the inner size of the this window
            // is zero. It will make an error when creating the wgpu surface when initializing the render system
            let window_ready_future = WindowReadyFuture {window: &window};
            window_ready_future.await;
            log::info!("window is ready");
        }

        let mut engine_handle = EngineHandle::new();
        log::info!("engine systems:engine_handle init");

        let global_players_settings = PlayerSettings::load_player_settings().await;
        log::info!("engine systems: global_players_settings init");
        
        let world = World::new(&mut engine_handle, global_players_settings).await;
        log::info!("engine systems: world init");
        
        let physic = PhysicsSystem::new(&world);
        log::info!("engine systems: physic init");

        let input = InputSystem::new();
        log::info!("engine systems: input init");

        let time = TimeSystem::new(60_u32);
        log::info!("engine systems: time init");

        let net = NetSystem::new(
            #[cfg(not(target_arch = "wasm32"))]
            &mut runtime
        ).await;
        log::info!("engine systems: net init");

        let (render, ui) = create_render_and_ui_systems(
            window,
            &world,
            &time,
            #[cfg(not(target_arch = "wasm32"))]
            &mut runtime
        ).await;

        let audio = AudioSystem::new().await;


        Engine {
            physic,
            input,
            render,
            time,
            world,
            engine_handle,
            net,
            audio,
            ui,
            #[cfg(not(target_arch = "wasm32"))]
            runtime,
        }
    }
}

async fn create_render_and_ui_systems(
    window: Window,
    world: &World,
    time: &TimeSystem,
    #[cfg(not(target_arch = "wasm32"))]
    runtime: &mut Runtime,
) -> (RenderSystem, UISystem) {

    let render_data = RenderData::new(world, time, &window);

    let egui_paint_jobs = Arc::new(Mutex::new(Vec::new()));
    
    let (renderer, window) = Renderer::new(
        window,
        &render_data,
        time.target_frame_duration.as_secs_f64(),
        egui_paint_jobs.clone(),
    ).await;

    let renderer = Arc::new(
        Mutex::new(
            renderer
        )
    );

    // spawn async tusk for renderer
    let async_renderer = renderer.clone();
    #[cfg(not(target_arch="wasm32"))]
    runtime.spawn(async move {
        loop {

            match async_renderer.try_lock() {
                Ok(mut renderer_lock) => {
                    if let Err(err) = renderer_lock.render(/*&self.window*/) {
                        match err {
                            // wgpu::SurfaceError::Lost => renderer_lock.resize(self.window.inner_size()),
            
                            // The system is out of memory, we should probably quit
                            wgpu::SurfaceError::OutOfMemory => panic!("Out of GPU memory"),
            
                            // All other errors (Outdated, Timeout) should be resolved by the next frame
                            _ => log::error!("{:?}", err),
                        }
                    }
                }
                Err(_) => {}
            }

            tokio::time::sleep(tokio::time::Duration::from_micros(500)).await;
        }
    });


    log::info!("render system: renderer init");

    
    let ui = UISystem {};
    
    let render = RenderSystem {
        window,
        renderer,

        render_data,
    };

    (render, ui)
}
