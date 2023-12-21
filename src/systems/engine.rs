use winit::{window::WindowBuilder, platform::web::WindowBuilderExtWebSys};

use winit::dpi::PhysicalSize;
use wasm_bindgen::JsValue;

use super::{
    render::RenderSystem,
    main_loop::MainLoop,
    input::InputSystem,
    physics::PhysicsSystem,
    time::TimeSystem,
    world::World,
    engine_handle::EngineHandle, net::NetSystem,
};

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
            .with_inner_size(PhysicalSize::new(1200, 800))
            .build(&cleint_main_loop.event_loop)
            .unwrap();

        // window.set_resize_increments(PhysicalSize::new(1200, 800).into());

        // use winit::platform::web::WindowExtWebSys;
        // web_sys::window()
        //     .and_then(|win| win.document())
        //     .and_then(|doc| {
        //         let dst = doc.get_element_by_id("4d-shooter")?;
        //         let canvas = web_sys::Element::from(window.canvas());
        //         dst.append_child(&canvas).ok()?;
        //         Some(())
        //     })
        //     .expect("Couldn't append canvas to document body.");

        let physic = PhysicsSystem::new();
        
        let render = RenderSystem::new(window).await;
 
        // let net = ClientNetSystem::new().await;
        
        let input = InputSystem::new();

        let time = TimeSystem::new(60_u32);

        let world = World::new();

        let engine_handle = EngineHandle::new();

        let net = NetSystem::new();

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
