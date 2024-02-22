use winit::{window::WindowBuilder, platform::web::WindowBuilderExtWebSys};

use wasm_bindgen::JsValue;

use super::{
    render::RenderSystem,
    main_loop::MainLoop,
    input::InputSystem,
    physics::PhysicsSystem,
    time::TimeSystem,
    world::World,
    actor::player::player_settings::PlayerSettings,
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
    pub global_players_settings: PlayerSettings,
    // pub runtime: RuntimeSystem,
    // pub net: ClientNetSystem,
}

impl Engine {
    pub async fn new(
        cleint_main_loop: &MainLoop,
        // async_runtime: &Runtime,
    ) -> Engine {

        log::info!("engine systems: window init");
        
        let world = World::new().await;

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

        log::info!("engine systems: world init");
        
        let physic = PhysicsSystem::new(&world);
        
        log::info!("engine systems: physic init");

        let input = InputSystem::new();

        log::info!("engine systems: input init");

        let time = TimeSystem::new(60_u32);

        log::info!("engine systems: time init");

        let engine_handle = EngineHandle::new();

        log::info!("engine systems: engine_handle init");

        let net = NetSystem::new();

        log::info!("engine systems: net init");

        let global_players_settings = PlayerSettings::load_player_settings().await;

        log::info!("engine systems: global_players_settings init");

        let render = RenderSystem::new(window, &world).await;

        log::info!("engine systems: render init");

        Engine {
            physic,
            input,
            render,
            time,
            world,
            engine_handle,
            net,
            global_players_settings,
        }
    }
}
