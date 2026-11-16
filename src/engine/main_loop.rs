// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2026  Timofei Molokov



use crate::engine::{Engine, shared_systems::ui::UIElementType};

use web_time::Instant;
use glam::Vec2;

use wgpu::Backend;
use winit::{
    application::ApplicationHandler, event::*, event_loop::{
        ControlFlow,
        EventLoop
    }, keyboard::{
        KeyCode,
        PhysicalKey
    }, window::{Fullscreen, WindowAttributes}
};


impl Engine
{
    /// Runs the application: the winit event loop creates the window, then calls `Engine::tick`
    /// at the engine's target rate. Returns when the application quits.
    /// Headless: no winit, a plain tick loop (`graphic_backend` is ignored).
    ///
    /// `initial_world`: the world loaded at startup (now the Capture the Flag map, later the main menu).
    pub fn run(mut self, initial_world: String, graphic_backend: Option<Backend>)
    {
        if self.headless
        {
            self.run_headless(initial_world);
            return;
        }

        let event_loop = EventLoop::new().expect("ERROR: can't create the event loop");

        let mut main_loop = MainLoop {
            engine: self,
            initial_world: Some(initial_world),
            graphic_backend,
        };

        event_loop.run_app(&mut main_loop).expect("ERROR: the event loop failed");

        // Persist user settings once more. The engine is dropped right after: root world, then shared
        // systems (the render thread is stopped and joined), then the window.
        main_loop.engine.shared.global_settings.save();
    }


    /// Headless main loop: no window, no winit. Ticks at the engine's target rate until a world
    /// requests to quit.
    fn run_headless(&mut self, initial_world: String)
    {
        self.start(None, initial_world, None);

        while !self.quit_requested
        {
            let next_tick = Instant::now() + self.shared.time.target_frame_duration;

            self.tick();

            std::thread::sleep(next_tick.saturating_duration_since(Instant::now()));
        }
    }
}


/// The winit application: creates the window and starts the engine, forwards events,
/// handles the window-level hotkeys and calls `Engine::tick` at the engine's target rate.
struct MainLoop
{
    engine: Engine,
    /// Startup parameters, used once the window is created (`StartCause::Init`).
    initial_world: Option<String>,
    graphic_backend: Option<Backend>,
}


impl ApplicationHandler for MainLoop
{
    fn resumed(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {}


    fn new_events(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, cause: StartCause) {
        match cause {
            StartCause::Init => {
                let window = event_loop
                    .create_window(
                        WindowAttributes::default()
                        .with_active(true)
                        .with_fullscreen(Some(winit::window::Fullscreen::Borderless(None)))
                        .with_title("Slice: 4D Shooter")
                    )
                    .unwrap();

                let engine = &mut self.engine;

                engine.start(
                    Some(window),
                    self.initial_world.take().unwrap(),
                    self.graphic_backend,
                );

                event_loop.set_control_flow(ControlFlow::WaitUntil(
                    Instant::now() + engine.shared.time.target_frame_duration
                ));
            }
            StartCause::ResumeTimeReached {..} => {
                let engine = &mut self.engine;

                event_loop.set_control_flow(ControlFlow::WaitUntil(
                    Instant::now() + engine.shared.time.target_frame_duration
                ));

                engine.tick();

                if engine.quit_requested()
                {
                    event_loop.exit();
                }
            }
            StartCause::WaitCancelled {..} => {}
            StartCause::Poll => {}
        }
    }


    fn device_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        let engine = &mut self.engine;

        match event {
            DeviceEvent::MouseMotion {delta} => {
                let (x,y) = delta;
                engine.shared.input.add_mouse_delta(Vec2::new(x as f32, y as f32))
            },
            DeviceEvent::MouseWheel { delta } => {
                engine.shared.input.set_mouse_wheel_input(delta);
            }
            _ => {}
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let engine = &mut self.engine;

        let Some(window) = engine.window().cloned() else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::Resized(size) => {
                engine.on_window_resized(size);
            },

            WindowEvent::ScaleFactorChanged {..} => {
                engine.on_window_resized(window.inner_size());
            },

            WindowEvent::KeyboardInput {event, ..} => {

                if let PhysicalKey::Code(code) = event.physical_key {
                    let shared = &mut engine.shared;

                    match code {
                        KeyCode::Escape => {
                            if event.state.is_pressed() {
                                window.set_cursor_visible(true);
                                window.set_cursor_grab(winit::window::CursorGrabMode::None).unwrap();
                                window.set_fullscreen(None);
                            }
                        },
                        KeyCode::Enter => {
                            window.set_cursor_grab(winit::window::CursorGrabMode::Confined).or_else(
                                |_| window.set_cursor_grab(winit::window::CursorGrabMode::Locked)
                            ).unwrap();
                            window.set_cursor_visible(false);

                            if event.state.is_pressed() {
                                window
                                    .set_fullscreen(Some(
                                        Fullscreen::Borderless(
                                            None
                                        )
                                    )
                                );
                            }
                        },
                        KeyCode::KeyJ | KeyCode::KeyK => {
                            let delta = shared.time.prev_frame_duration() * 0.4;
                            let sensitivity = &mut shared.global_settings.user.mouse_sensitivity;

                            *sensitivity = if code == KeyCode::KeyK {*sensitivity + delta} else {*sensitivity - delta};
                            *sensitivity = sensitivity.max(0.0);

                            if !event.state.is_pressed() {
                                shared.global_settings.save();
                            }
                        },
                        KeyCode::KeyN | KeyCode::KeyM => {
                            let delta = shared.time.prev_frame_duration();

                            if code == KeyCode::KeyM {
                                shared.audio.increase_sound_volume(delta);
                            } else {
                                shared.audio.decrease_sound_volume(delta);
                            }

                            shared.global_settings.user.master_volume = shared.audio.master_volume;

                            if !event.state.is_pressed() {
                                shared.global_settings.save();
                            }
                        },
                        KeyCode::KeyY | KeyCode::KeyH => {
                            if event.state == ElementState::Pressed
                                && shared.ui.get_ui_element(&UIElementType::TutorialWindow).get_ui_data().is_visible
                            {
                                let url = if code == KeyCode::KeyY {"https://youtu.be/Pl355uSTBLc"} else {"https://slice4d.info"};

                                match opener::open_browser(url)
                                {
                                    Err(_) => eprintln!(
                                        "can't open the browser"
                                    ),
                                    _ => {
                                        window.set_cursor_visible(true);
                                        window.set_cursor_grab(winit::window::CursorGrabMode::None).unwrap();
                                    }
                                };
                            }
                        }
                        _ => {
                            shared.input.set_keyboard_input(&event);
                        }
                    }
                }
            },

            WindowEvent::MouseInput {button, state,..} => {

                if let MouseButton::Left = button {
                    if state.is_pressed() {
                        window.set_cursor_grab(winit::window::CursorGrabMode::Confined).or_else(
                            |_| window.set_cursor_grab(winit::window::CursorGrabMode::Locked)
                        ).unwrap();
                        window.set_cursor_visible(false);
                    }
                }

                engine.shared.input.set_mouse_button_input(&button, &state);
            },
            _ => {},
        }
    }
}
