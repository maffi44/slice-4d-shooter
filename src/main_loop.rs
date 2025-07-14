use crate::{
    actor::{
        flag::Flag, move_w_bonus::{self, MoveWBonusSpot}, main_player::{
            self, player_input_master::{
                InputMaster,
                LocalMaster
            }, MainPlayer, PlayerMessage
        }, session_controller::{self, SessionController}, Actor, ActorWrapper, Message, SpecificActorMessage
    },
    engine::{
        engine_handle::{
            Command,
            CommandType
        }, input::ActionsFrameState, world, Engine
    },
};

use client_server_protocol::Team;
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
    }, window::{Fullscreen, Window, WindowAttributes}
};

pub struct 
MainLoop {
    pub event_loop: EventLoop<()>,
}

pub struct Slice4DShooter
{
    systems: Option<Engine>,
    init_level_closure: Option<Box<dyn FnOnce(&mut Engine)>>,
    with_ui_renderer: bool,
    it_is_2d_3d_example: bool,
    with_generated_raymarch_shader: bool,
    specific_backend: Option<Backend>,
}

impl Slice4DShooter
{
    pub fn new(
        with_ui_renderer: bool,
        it_is_2d_3d_example: bool,
        with_generated_raymarch_shader: bool,
        specific_backend: Option<Backend>,
        init_level: Box<dyn FnOnce(&mut Engine)>,
    ) -> Self
    {

        Slice4DShooter {
            systems: None,
            init_level_closure: Some(init_level),
            with_ui_renderer,
            it_is_2d_3d_example,
            with_generated_raymarch_shader,
            specific_backend,
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
                            self.with_ui_renderer,
                            self.it_is_2d_3d_example,
                            self.with_generated_raymarch_shader,
                            self.specific_backend,
                        )
                    )
                );

                self.init_level_closure.take().unwrap()(self.systems.as_mut().unwrap());
                
                log::info!("main: Engine systems init");

                self.systems.as_mut().unwrap().time.init();

                event_loop.set_control_flow(ControlFlow::WaitUntil(
                    Instant::now() + self.systems.as_mut().unwrap().time.target_frame_duration
                ));
            }
            StartCause::ResumeTimeReached {
                start,
                requested_resume
            } => {
                // set wake up time gof the next interation
                event_loop.set_control_flow(ControlFlow::WaitUntil(
                    Instant::now() + self.systems.as_mut().unwrap().time.target_frame_duration
                    // Instant::from(
                    //     systems.as_mut().unwrap().time.timestamp_of_main_loop_start +
                    //     Duration::from_secs_f64(
                    //         systems.as_mut().unwrap().time.target_frame_duration.as_secs_f64() *
                    //         (systems.as_mut().unwrap().time.frame_counter + 1) as f64
                    //     )
                    // )
                ));
                
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
            
            WindowEvent::KeyboardInput {event,is_synthetic, ..} => {

                if let PhysicalKey::Code(code) = event.physical_key {
                    match code {
                        KeyCode::Escape => {
                            if event.state.is_pressed() {
                                systems.render.window.set_cursor_visible(true);
                                // #[cfg(target_arch="wasm32")]
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
                        KeyCode::Numpad1 => {
                            systems.settings.decrease_mouse_sensitivity(
                                systems.time.get_prev_frame_duration()
                            );
                        },
                        KeyCode::Numpad2 => {
                            systems.settings.increase_mouse_sensitivity(
                                systems.time.get_prev_frame_duration()
                            );
                        },
                        KeyCode::Numpad4 => {
                            systems.audio.decrease_sound_volume(
                                systems.time.get_prev_frame_duration()
                            );
                        },
                        KeyCode::Numpad5 => {
                            systems.audio.increase_sound_volume(
                                systems.time.get_prev_frame_duration()
                            );
                        },
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
        with_ui_renderer: bool,
        it_is_2d_3d_example: bool,
        with_generated_raymarch_shader: bool,
        specific_backend: Option<Backend>,
        init_level: Box<dyn FnOnce(&mut Engine)>,
    ) {
        // #[cfg(target_arch="wasm32")]
        // let mut it_is_first_input_action = true;

        let mut slice_4d_shooter_app = Slice4DShooter::new(
            with_ui_renderer,
            it_is_2d_3d_example,
            with_generated_raymarch_shader,
            specific_backend,
            init_level,
        );
        
        let active_event_loop = self.event_loop.run_app(
            &mut slice_4d_shooter_app
        ).unwrap();
    }
   
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

    systems.world.send_messages_and_process_commands(
        &mut systems.net,
        &systems.physic,
        &mut systems.audio,
        &mut systems.ui,
        &mut systems.engine_handle,
        &mut systems.time,
        &mut systems.effects,
    );

    systems.physic.process_physics(
        &mut systems.world, 
        systems.time.get_prev_frame_duration(),
        &mut systems.engine_handle
    );

    systems.world.send_messages_and_process_commands(
        &mut systems.net,
        &systems.physic,
        &mut systems.audio,
        &mut systems.ui,
        &mut systems.engine_handle,
        &mut systems.time,
        &mut systems.effects,
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