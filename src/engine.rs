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
    actor::main_player::player_settings::PlayerSettings,
    main_loop::{self, MainLoop}
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

use effects::EffectsSystem;
// use winit::window::WindowBuilder;

#[cfg(target_arch = "wasm32")]
use winit::platform::web::WindowBuilderExtWebSys;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use winit::{event_loop::ActiveEventLoop, monitor::MonitorHandle, window::{Cursor, Fullscreen, Window, WindowAttributes, WindowButtons}};

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
        cleint_main_loop: &MainLoop,
        it_is_2d_3d_example: bool,
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
            window = cleint_main_loop.event_loop.create_window(
                WindowAttributes::default()
                .with_active(true)
                .with_fullscreen(Some(winit::window::Fullscreen::Borderless(None)))
                .with_title("Just 4D Shooter")
            ).unwrap();
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
        
        let world = World::new(
            &mut engine_handle,
            global_players_settings,
            if it_is_2d_3d_example
            {
                "map_2d_3d".to_string()
            }
            else
            {
                "map".to_string()    
            }
        ).await;
        log::info!("engine systems: world init");
        
        let physic = PhysicsSystem::new(&world);
        log::info!("engine systems: physic init");

        let input = InputSystem::new();
        log::info!("engine systems: input init");

        let time = TimeSystem::new(60_u32);
        log::info!("engine systems: time init");

        let mut pre_initialized_ui = UISystem::new(); 

        let render = RenderSystem::new(
            window,
            &world,
            &time,
            &mut pre_initialized_ui,
            #[cfg(not(target_arch = "wasm32"))]
            &mut runtime,
            it_is_2d_3d_example,
        ).await;
        log::info!("engine systems: render init");

        let initialized_ui = pre_initialized_ui;

        let audio = AudioSystem::new().await;

        let effects = EffectsSystem::new();

        let net = NetSystem::new(
            &world.players_settings,
            &world.level.w_levels,
            #[cfg(not(target_arch = "wasm32"))]
            &mut runtime
        ).await;
        log::info!("engine systems: net init");


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
            
            #[cfg(not(target_arch = "wasm32"))]
            runtime,
        }
    }
}
