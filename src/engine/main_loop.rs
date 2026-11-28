// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2026  Timofei Molokov



use std::{sync::{mpsc::TryRecvError, Arc}, time::Instant};

use crate::engine::{
    Engine, EngineLoader,
    loader::{LoadEvent, WindowSurface},
    shared_systems::render::WindowPresentation, world::ui::UIElementType,
};

use glam::Vec2;

use winit::{
    application::ApplicationHandler, event::*, event_loop::{
        ControlFlow,
        EventLoop
    }, keyboard::{
        KeyCode,
        PhysicalKey
    }, window::{Fullscreen, Window, WindowAttributes}
};


/// The application runner: loading/event delivery first, simulation timing only once ready.
pub struct MainLoop
{
    state: State,
    presentation: Option<WindowPresentation>,
    closing: bool,
    failure: Option<String>,
    // Last owner on the main thread, including cancellation and late loading results.
    window: Option<Arc<Window>>,
}

pub(in crate::engine) enum AppEvent
{
    Loading,
    RenderStarted(WindowPresentation),
    ReplaceSurface,
    RenderFailed(String),
}

enum State
{
    Loading(EngineLoader),
    Running(Engine),
    Stopped,
}

impl MainLoop
{
    pub fn run(loading: EngineLoader) -> Result<(), String>
    {
        if loading.headless
        {
            let result = loading.receiver.as_ref().unwrap().recv()
                .expect("ERROR: engine loader disconnected");
            let LoadEvent::Finished(Ok(Some(loaded))) = result else {
                if let LoadEvent::Finished(Err(error)) = result { panic!("Engine loading failed: {error}"); }
                panic!("Headless engine loading did not complete");
            };
            let mut engine = Engine::new(loaded, None);
            drop(loading);
            while !engine.quit_requested()
            {
                let next_tick = Instant::now() + engine.shared.time.target_frame_duration;
                engine.tick();
                std::thread::sleep(next_tick.saturating_duration_since(Instant::now()));
            }
            engine.shared.global_settings.save();
            engine.shared.runtime.block_on(engine.shared.global_settings.flush());
            return Ok(());
        }

        let event_loop = EventLoop::<AppEvent>::with_user_event().build()
            .expect("ERROR: can't create the event loop");
        loading.set_waker(event_loop.create_proxy());
        let mut main_loop = MainLoop {
            state: State::Loading(loading),
            closing: false,
            presentation: None,
            failure: None,
            window: None,
        };
        event_loop.set_control_flow(ControlFlow::Wait);
        event_loop.run_app(&mut main_loop).expect("ERROR: the event loop failed");
        if let State::Running(engine) = &main_loop.state
        {
            engine.shared.global_settings.save();
            engine.shared.runtime.block_on(engine.shared.global_settings.flush());
        }
        if let Some(error) = main_loop.failure.take() {
            log::error!("{error}");
            return Err(error);
        }
        Ok(())
    }

    fn receive_loading_events(&mut self, event_loop: &winit::event_loop::ActiveEventLoop)
    {
        // A wake can arrive before the first resumed. Leave queued requests for resumed to drain.
        if self.window.is_none() { return; }
        loop
        {
            let State::Loading(loading) = &self.state else { return; };
            let event = match loading.receiver.as_ref().unwrap().try_recv()
            {
                Ok(event) => event,
                Err(TryRecvError::Empty) => return,
                Err(TryRecvError::Disconnected) => {
                    self.failure = Some("engine loader disconnected".into());
                    event_loop.exit();
                    return;
                }
            };
            match event
            {
                LoadEvent::CreateSurface { instance, reply } => {
                    if self.closing { continue; } // Dropping the reply releases the waiting worker.
                    let window = self.window.as_ref().unwrap().clone();
                    match instance.create_surface(window.clone())
                    {
                        Ok(surface) => {
                            let size = window.inner_size();
                            let _ = reply.send(WindowSurface { window, surface, size });
                        }
                        Err(error) => {
                            self.failure = Some(format!("can't create the window surface: {error}"));
                            self.close(event_loop);
                        }
                    }
                }
                LoadEvent::Finished(result) => {
                    match result
                    {
                        Ok(Some(loaded)) if !self.closing => {
                            let mut engine = Engine::new(loaded, self.window.clone());
                            // The window may have been resized/minimized during loading.
                            engine.on_window_resized(self.window.as_ref().unwrap().inner_size());
                            event_loop.set_control_flow(ControlFlow::WaitUntil(
                                Instant::now() + engine.shared.time.target_frame_duration
                            ));
                            self.state = State::Running(engine);
                        }
                        result => {
                            if let Err(error) = result { self.failure.get_or_insert(error); }
                            self.state = State::Stopped;
                            event_loop.exit();
                        }
                    }
                    return;
                }
            }
        }
    }

    fn close(&mut self, event_loop: &winit::event_loop::ActiveEventLoop)
    {
        self.closing = true;
        if let State::Loading(loading) = &self.state
        {
            loading.cancel();
            if let Some(window) = &self.window { window.set_visible(false); }
            // Keep dispatching while an in-flight driver call finishes. Never join it in a callback.
            self.receive_loading_events(event_loop);
        }
        else { event_loop.exit(); }
    }
}

impl ApplicationHandler<AppEvent> for MainLoop
{
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop)
    {
        if self.window.is_none()
        {
            self.window = Some(Arc::new(event_loop.create_window(
                WindowAttributes::default()
                    .with_active(true)
                    .with_fullscreen(Some(Fullscreen::Borderless(None)))
                    .with_title("Slice: 4D Shooter")
            ).expect("ERROR: can't create the window")));
        }
        self.receive_loading_events(event_loop);
    }

    fn user_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, event: AppEvent)
    {
        match event
        {
            AppEvent::Loading => self.receive_loading_events(event_loop),
            _ if self.closing => {}
            AppEvent::RenderStarted(presentation) => {
                let window = self.window.as_ref().expect("renderer requires a window");
                presentation.resize(window.inner_size());
                self.presentation = Some(presentation);
            }
            AppEvent::ReplaceSurface => {
                let presentation = self.presentation.as_ref().expect("renderer registered before first frame");
                let window = self.window.as_ref().expect("renderer requires a window");
                if let Err(error) = presentation.replace_surface(window) {
                    self.failure = Some(format!("Cannot recreate the window surface: {error}"));
                    self.close(event_loop);
                }
            }
            AppEvent::RenderFailed(error) => {
                self.failure = Some(error);
                self.close(event_loop);
            }
        }
    }

    fn new_events(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, cause: StartCause)
    {
        if let (StartCause::ResumeTimeReached {..}, State::Running(engine)) = (cause, &mut self.state)
        {
            event_loop.set_control_flow(ControlFlow::WaitUntil(
                Instant::now() + engine.shared.time.target_frame_duration
            ));
            engine.tick();
            if engine.quit_requested() { event_loop.exit(); }
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        let State::Running(engine) = &mut self.state else { return; };

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
        if self.closing { return; }
        if matches!(event, WindowEvent::CloseRequested)
        {
            self.close(event_loop);
            return;
        }
        let size = match &event {
            WindowEvent::Resized(size) => Some(*size),
            WindowEvent::ScaleFactorChanged {..} => self.window.as_ref().map(|window| window.inner_size()),
            _ => None,
        };
        if let Some(size) = size {
            if let Some(presentation) = &self.presentation { presentation.resize(size); }
            if let State::Running(engine) = &mut self.state { engine.on_window_resized(size); }
            return;
        }
        let State::Running(engine) = &mut self.state else { return; };

        let Some(window) = engine.window().cloned() else {
            return;
        };

        match event {
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
                                && engine.root_world.as_ref().and_then(|w| w.ui.ui_elements.get(&UIElementType::TutorialWindow)).is_some_and(|element| element.get_ui_data().is_visible)
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
