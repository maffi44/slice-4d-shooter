use web_sys::HtmlCanvasElement;
use winit::{window::WindowBuilder, dpi::LogicalSize};

use winit::dpi::PhysicalSize;

use crate::common_systems::{
    physics_system::PhysicsSystem,
    timeSystem::TimeSystem,
};

use super::{
    render_system::RenderSystem,
    main_loop::MainLoop,
    input_system::InputSystem,
};

pub struct Engine {
    pub render: RenderSystem,
    pub input: InputSystem,
    pub physic: PhysicsSystem,
    pub time: TimeSystem,
    // pub runtime: RuntimeSystem,
    // pub net: ClientNetSystem,
}

impl Engine {
    pub async fn new(
        cleint_main_loop: &MainLoop,
        // async_runtime: &Runtime,
    ) -> Engine {

        let window_builder = WindowBuilder::new();
        let window = window_builder
            .with_inner_size(PhysicalSize::new(450, 400))
            .build(&cleint_main_loop.event_loop)
            .unwrap();

        // window.set_resize_increments(PhysicalSize::new(450, 400).into());

        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("4d-shooter")?;
                let canvas = web_sys::Element::from(window.canvas());
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.");

        let physic = PhysicsSystem::new();
        
        let render = RenderSystem::new(window).await;
 
        // let net = ClientNetSystem::new().await;
        
        let input = InputSystem::new();

        let time = TimeSystem::new(60_u32);

        Engine {
            physic,
            input,
            render,
            time,
            // net,
        }
    }
}
