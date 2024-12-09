use crate::{
    actor::{
        player::{
            player_input_master::{
                InputMaster,
                LocalMaster
            }, Player
        },
        Actor,
        ActorWrapper,
    },
    engine::{
        input::ActionsFrameState, Engine
    },
};

use web_time::Instant;
use glam::Vec2;

use winit::{
    event::*,
    event_loop::{
        ControlFlow,
        EventLoop
    },
    keyboard::{
        KeyCode,
        PhysicalKey
    },
    window::Fullscreen
};

pub struct 
MainLoop {
    pub event_loop: EventLoop<()>,
}

impl MainLoop {

    pub fn new() -> Self {
        MainLoop {
            event_loop: EventLoop::new().unwrap(),
        }
    }

    pub async fn run(
        self,
        mut systems : Engine,
    ) {
        
        let systems = &mut systems;
        
        init(systems);

        log::info!("init(systems) called");

        #[cfg(target_arch="wasm32")]
        let mut it_is_first_action = true;

        let _ = self.event_loop.run(move |event, elwt|{
            match event {
                Event::NewEvents(cause) => {
                    match cause {
                        StartCause::Init => {
                            systems.time.init();

                            elwt.set_control_flow(ControlFlow::WaitUntil(
                                Instant::now() + systems.time.target_frame_duration
                            ));
                        }
                        StartCause::ResumeTimeReached {
                            start,
                            requested_resume
                        } => {
                            // set wake up time gof the next interation
                            elwt.set_control_flow(ControlFlow::WaitUntil(
                                Instant::now() + systems.time.target_frame_duration
                                // Instant::from(
                                //     systems.time.timestamp_of_main_loop_start +
                                //     Duration::from_secs_f64(
                                //         systems.time.target_frame_duration.as_secs_f64() *
                                //         (systems.time.frame_counter + 1) as f64
                                //     )
                                // )
                            ));
                            
                            main_loop_tick(systems);

                        }
                        StartCause::WaitCancelled {
                            start,
                            requested_resume
                        } => {


                        }
                        StartCause::Poll => {

                        }
                    } {
                        
                    }
                }
                Event::Suspended => {

                }
                Event::Resumed => {

                }
                Event::AboutToWait => {

                }
                Event::WindowEvent {
                    ref event,
                    ..
                } => {
                    match event {
                        WindowEvent::CloseRequested => elwt.exit(),

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
                                    _ => {}
                                }
                            }

                            systems.input.set_keyboard_input(event);
                        },

                        WindowEvent::MouseInput {button, state,..} => {

                            // if left click set cursor grabbed
                            match button {
                                MouseButton::Left => {

                                    if state.is_pressed() {

                                        // it is necessary on web target because a browsers is prevent 
                                        // to create an audio context before first input action
                                        #[cfg(target_arch="wasm32")]
                                        if it_is_first_action {
                                            systems.audio.sound_engine.initialize_audio_output_device().unwrap();
                                            it_is_first_action = false
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
                            systems.input.set_mouse_button_input(button, state);
                        },
                        _ => {},
                    }
                },
                Event::DeviceEvent {
                    device_id, event
                } => {
                    match event {
                        DeviceEvent::MouseMotion {delta} => {             
                            let (x,y) = delta;
                            systems.input.add_mouse_delta(Vec2::new(x as f32, y as f32))
        
                        },
                        _ => {}
                    }
                },
                Event::UserEvent(event) => {

                },
                _ => {}
            }
        });
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
        &mut systems.engine_handle,
        &mut systems.runtime,
        &mut systems.audio
    );

    systems.input.get_input(&mut systems.world, &mut systems.net);

    systems.world.tick(
        &systems.physic,
        &mut systems.engine_handle,
        &mut systems.audio,
        &mut systems.ui,
        &mut systems.time,
    );

    systems.world.send_messages_and_process_commands(
        &mut systems.net,
        &systems.physic,
        &mut systems.audio,
        &mut systems.ui,
        &mut systems.engine_handle,
        &mut systems.time,
    );

    systems.physic.process_physics(
        &mut systems.world, 
        systems.time.prev_frame_duration,
        &mut systems.engine_handle
    );

    systems.world.send_messages_and_process_commands(
        &mut systems.net,
        &systems.physic,
        &mut systems.audio,
        &mut systems.ui,
        &mut systems.engine_handle,
        &mut systems.time
    );

    systems.render.send_data_to_renderer(
        &systems.world,
        &systems.time,
        &systems.ui,
    );

    systems.input.reset_input();

    systems.time.end_of_frame();
}



fn init(systems: &mut Engine) {

    let mut main_player = Player::new(
        InputMaster::LocalMaster(
            LocalMaster::new(ActionsFrameState::empty())
        ),
        systems.world.players_settings.clone(),
        &mut systems.audio,
        systems.world.level.w_levels.clone()
    );

    let spawn = systems
        .world
        .level
        .get_random_spawn_position(main_player.get_team());

    main_player.get_mut_transform().set_position(
        spawn.spawn_position
    );

    main_player.set_current_w_level(spawn.w_level);

    let main_player_id = systems.world.add_actor_to_world(
        ActorWrapper::Player(main_player),
        &mut systems.engine_handle,
    );

    systems.world.main_player_id = main_player_id;
}