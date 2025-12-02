// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2025  Timofei Molokov

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use crate::{
    actor::Actor,
    engine::{Engine, send_messages_and_process_commands, ui::UIElementType},
};

use web_time::Instant;
use glam::Vec2;
use gilrs;

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

pub struct 
MainLoop {
    pub event_loop: EventLoop<()>,
}

pub struct Slice4DShooter
{
    systems: Option<Engine>,
    start_level: Option<String>,
    with_ui_renderer: bool,
    it_is_2d_3d_example: bool,
    disable_net_system: bool,
    specific_backend: Option<Backend>,
}

impl Slice4DShooter
{
    pub fn new(
        start_level: String,
        with_ui_renderer: bool,
        it_is_2d_3d_example: bool,
        specific_backend: Option<Backend>,
        disable_net_system: bool,
    ) -> Self
    {

        Slice4DShooter {
            systems: None,
            start_level: Some(start_level),
            with_ui_renderer,
            it_is_2d_3d_example,
            specific_backend,
            disable_net_system,
        }
    }
}

impl ApplicationHandler for Slice4DShooter
{
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {}
    

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

                self.systems = Some(
                    pollster::block_on(
                    Engine::new(
                            window,
                            self.start_level.take().unwrap(),
                            self.with_ui_renderer,
                            self.it_is_2d_3d_example,
                            self.disable_net_system,

                            self.specific_backend,
                        )
                    )
                );
                
                log::info!("main: Engine systems init");

                self.systems.as_mut().unwrap().time.init();

                event_loop.set_control_flow(ControlFlow::WaitUntil(
                    Instant::now() + self.systems.as_mut().unwrap().time.target_frame_duration
                ));

                println!("init");
            }
            StartCause::ResumeTimeReached {
                start,
                requested_resume
            } => {
                // set wake up time gof the next interation
                event_loop.set_control_flow(ControlFlow::WaitUntil(
                    Instant::now() + self.systems.as_mut().unwrap().time.target_frame_duration
                ));

                collect_gamepad_input(self.systems.as_mut().unwrap());
                
                main_loop_tick(self.systems.as_mut().unwrap());

            }
            StartCause::WaitCancelled {
                start,
                requested_resume
            } => {


            }
            StartCause::Poll => {

            }
        }
    }


    fn device_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        device_id: DeviceId,
        event: DeviceEvent,
    ) {
        match event {
            DeviceEvent::MouseMotion {delta} => {             
                let (x,y) = delta;
                self.systems.as_mut().unwrap().input.add_mouse_delta(Vec2::new(x as f32, y as f32))

            },
            DeviceEvent::MouseWheel { delta } => {
                self.systems.as_mut().unwrap().input.set_mouse_wheel_input(delta);
            }
            _ => {}
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let systems = self.systems.as_mut().unwrap();

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::Resized(_) => {
                systems.render.resize_frame_buffer();
            },

            WindowEvent::ScaleFactorChanged {..} => {
                systems.render.resize_frame_buffer();
            },
            
            WindowEvent::KeyboardInput {event, ..} => {

                if let PhysicalKey::Code(code) = event.physical_key {
                    match code {
                        KeyCode::Escape => {
                            if event.state.is_pressed() {
                                systems.render.window.set_cursor_visible(true);
                                systems.render.window.set_cursor_grab(winit::window::CursorGrabMode::None).unwrap();
                                systems.render.window.set_fullscreen(None);
                            }
                        },
                        KeyCode::Enter => {
                            // #[cfg(target_arch="wasm32")]
                            systems.render.window.set_cursor_grab(winit::window::CursorGrabMode::Confined).or_else(
                                |_| systems.render.window.set_cursor_grab(winit::window::CursorGrabMode::Locked)
                            ).unwrap();
                            systems.render.window.set_cursor_visible(false);

                            if event.state.is_pressed() {
                                systems.render.window
                                    .set_fullscreen(Some(
                                        Fullscreen::Borderless(
                                            None
                                        )
                                    )
                                );
                            }
                        },
                        KeyCode::KeyJ => {
                            systems.settings.decrease_mouse_sensitivity(
                                systems.time.get_prev_frame_duration()
                            );
                        },
                        KeyCode::KeyK => {
                            systems.settings.increase_mouse_sensitivity(
                                systems.time.get_prev_frame_duration()
                            );
                        },
                        KeyCode::KeyN => {
                            systems.audio.decrease_sound_volume(
                                systems.time.get_prev_frame_duration()
                            );
                        },
                        KeyCode::KeyM => {
                            systems.audio.increase_sound_volume(
                                systems.time.get_prev_frame_duration()
                            );
                        },
                        KeyCode::KeyY => {
                            match event.state
                            {
                                ElementState::Pressed =>
                                {
                                    //temporary
                                    if systems.ui.get_ui_element(&UIElementType::TutorialWindow).get_ui_data().is_visible
                                    {
                                        // Temporary link for youtube video. There will be a link to the video tutorial later
                                        match opener::open_browser("https://youtu.be/Pl355uSTBLc")
                                        {
                                            Err(e) => eprintln!(
                                                "Can't open a browser"
                                            ),
                                            _ => {
                                                systems.render.window.set_cursor_visible(true);
                                                systems.render.window.set_cursor_grab(winit::window::CursorGrabMode::None).unwrap();
                                            }
                                        };
                                    }
                                },
                                ElementState::Released => {},
                            }
                            
                        }
                        KeyCode::KeyH => {
                            match event.state
                            {
                                ElementState::Pressed =>
                                {
                                    //temporary
                                    if systems.ui.get_ui_element(&UIElementType::TutorialWindow).get_ui_data().is_visible
                                    {
                                        // Temporary link for youtube video. There will be a link to the video tutorial later
                                        match opener::open_browser("https://slice4d.info")
                                        {
                                            Err(e) => eprintln!(
                                                "Can't open a browser"
                                            ),
                                            _ => {
                                                systems.render.window.set_cursor_visible(true);
                                                systems.render.window.set_cursor_grab(winit::window::CursorGrabMode::None).unwrap();
                                            }
                                        };
                                    }
                                },
                                ElementState::Released => {},
                            }
                            
                        }
                        _ => {
                            systems.input.set_keyboard_input(&event);
                        }
                    }
                }
            },

            WindowEvent::MouseInput {button, state,..} => {

                // if left click set cursor grabbed
                match button {
                    MouseButton::Left => {

                        if state.is_pressed() {

                            // it is necessary on web target because a browsers is prevent 
                            // to create an audio context before first input action
                            #[cfg(target_arch="wasm32")]
                            if it_is_first_input_action {
                                systems.audio.sound_engine.initialize_audio_output_device().unwrap();
                                it_is_first_input_action = false
                            }
                            
                            systems.render.window.set_cursor_grab(winit::window::CursorGrabMode::Confined).or_else(
                                |_| systems.render.window.set_cursor_grab(winit::window::CursorGrabMode::Locked)
                            ).unwrap();
                            systems.render.window.set_cursor_visible(false);
                        }
                    },
                    _ => {},
                }
                // set mouse input
                systems.input.set_mouse_button_input(&button, &state);
            },
            _ => {},
        }
    }
}

impl MainLoop {

    pub fn new() -> Self {
        MainLoop {
            event_loop: EventLoop::new().unwrap(),
        }
    }

    pub async fn run(
        self,
        start_level: String,
        with_ui_renderer: bool,
        it_is_2d_3d_example: bool,
        specific_backend: Option<Backend>,
        disable_net_system: bool,
    ) {
        // #[cfg(target_arch="wasm32")]
        // let mut it_is_first_input_action = true;

        let mut slice_4d_shooter_app = Slice4DShooter::new(
            start_level,
            with_ui_renderer,
            it_is_2d_3d_example,
            specific_backend,
            disable_net_system,
        );
        
        let active_event_loop = self.event_loop.run_app(
            &mut slice_4d_shooter_app
        ).unwrap();
    }
   
}


#[inline]
fn collect_gamepad_input (
    systems : &mut Engine,
) {
    systems.input.collect_gamepad_button_input();
}


#[inline]
fn main_loop_tick(
    systems : &mut Engine,
) {
    systems.time.start_of_frame();

    #[cfg(target_arch= "wasm32")]
    systems.net.tick(
        &mut systems.engine_handle,
        &mut systems.audio
    );

    #[cfg(not(target_arch= "wasm32"))]
    systems.net.tick(
        systems.input.get_input(),
        &mut systems.engine_handle,
        &mut systems.runtime,
        &mut systems.audio,
        &mut systems.ui,
    );

    systems.input.set_input_to_controlled_actors(&mut systems.world, &mut systems.net);

    systems.world.tick(
        &systems.physic,
        &mut systems.engine_handle,
        &mut systems.audio,
        &mut systems.ui,
        &mut systems.time,
        &mut systems.effects,
    );

    send_messages_and_process_commands(
        &mut systems.world,
        &mut systems.net,
        &mut systems.physic,
        &mut systems.audio,
        &mut systems.ui,
        &mut systems.engine_handle,
        &mut systems.time,
        &mut systems.effects,
        Some(&mut systems.render),
        &mut systems.runtime,
    );

    systems.physic.process_physics(
        &mut systems.world, 
        systems.time.get_prev_frame_duration(),
        &mut systems.engine_handle
    );

    send_messages_and_process_commands(
        &mut systems.world,
        &mut systems.net,
        &mut systems.physic,
        &mut systems.audio,
        &mut systems.ui,
        &mut systems.engine_handle,
        &mut systems.time,
        &mut systems.effects,
        Some(&mut systems.render),
        &mut systems.runtime,
    );

    systems.render.process_player_input(
        systems.input.get_input(),
    );

    systems.render.send_data_to_renderer(
        &systems.world,
        &systems.time,
        &systems.ui,
    );

    systems.input.reset_input();

    systems.time.end_of_frame();
}