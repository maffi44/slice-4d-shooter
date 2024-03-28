pub mod render;
pub mod input;
pub mod net;
pub mod time;
pub mod physics;
pub mod effects;
pub mod world;
pub mod engine_handle;

use std::{future::Future, pin::Pin, rc::Rc, task::{Context, Poll}};

use crate::{
    actor::player::player_settings::PlayerSettings,
    main_loop::MainLoop,
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

use wasm_bindgen_futures::JsFuture;
use web_sys::js_sys::Promise;
use winit::{
    platform::web::WindowBuilderExtWebSys, window::{Window, WindowBuilder}
};

use wasm_bindgen::{closure::Closure, JsCast, JsValue};

pub struct Engine {
    pub render: RenderSystem,
    pub input: InputSystem,
    pub physic: PhysicsSystem,
    pub time: TimeSystem,
    pub world: World,
    pub engine_handle: EngineHandle,
    pub net: NetSystem,
    // pub runtime: RuntimeSystem,
    // pub net: ClientNetSystem,
}

struct WindowReadyFuture<'a> {
    window: &'a Window
}

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

        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document.get_element_by_id("game_canvas").unwrap();
        let canvas: web_sys::HtmlCanvasElement = JsValue::from(canvas).into();

        let window_builder = WindowBuilder::new();
        let window = window_builder
            .with_canvas(Some(canvas))
            .with_active(true)
            // .with_fullscreen(Some(winit::window::Fullscreen::Borderless(None)))
            // .with_inner_size(PhysicalSize::new(1200, 800))
            .build(&cleint_main_loop.event_loop)
            .unwrap();
        log::info!("engine systems: window init");


        // it is necessary because immidiatly after creating the window the inner size of the this window
        // is zero. It will make an error when creating the wgpu surface when initializing the render system
        let window_ready_future = WindowReadyFuture {window: &window};
        window_ready_future.await;
        log::info!("window is ready");

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

        let net = NetSystem::new().await;
        log::info!("engine systems: net init");

        let render = RenderSystem::new(window, &world, &time).await;
        log::info!("engine systems: render init");

        Engine {
            physic,
            input,
            render,
            time,
            world,
            engine_handle,
            net,
        }
    }
}
