use super::{
    engine::Engine,
    input::ActionsFrameState,
    actor::{
        Message,
        player::player_input_master::{
            InputMaster,
            LocalMaster
        },
    },
    transform::Transform,
    engine_handle::{
        Command,
        CommandType,
    },
};

use std::time::Duration;
use web_time::Instant;
use glam::Vec2;

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{self, Fullscreen}
};

pub struct 
MainLoop {
    pub event_loop: EventLoop<()>,
    pointer_is_grabbed: bool,
}

impl MainLoop {

    pub fn new() -> Self {
        MainLoop {
            event_loop: EventLoop::new().unwrap(),
            pointer_is_grabbed: false,
        }
    }

    pub async fn run<'a>(
        self,
        mut systems : Engine,
    ) {
        
        let systems = &mut systems;
        
        init(systems);

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
                            
                            main_loop(systems);

                            // set wake up time gof the next interation
                            elwt.set_control_flow(ControlFlow::WaitUntil(
                                Instant::from(
                                    systems.time.timestamp_of_main_loop_start +
                                    Duration::from_secs_f64(
                                        systems.time.target_frame_duration.as_secs_f64() *
                                        (systems.time.frame_counter + 1) as f64
                                    )
                                )
                            ));
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
                                            systems.render.window.set_cursor_grab(window::CursorGrabMode::None).unwrap();
                                            systems.render.window.set_fullscreen(None);
                                        }
                                    },
                                    KeyCode::Enter => {
                                        systems.render.window.set_cursor_grab(window::CursorGrabMode::Locked).unwrap();
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
                                        systems.render.window.set_cursor_grab(window::CursorGrabMode::Locked).unwrap();
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
fn main_loop(
    systems : &mut Engine,
) {
    systems.time.start_of_frame();

    systems.input.get_input(&mut systems.world, &mut systems.net);

    systems.world.process_input(&mut systems.engine_handle);

    systems.world.process_commands(&mut systems.engine_handle);

    systems.physic.process_physics(
        &mut systems.world, 
        systems.time.target_frame_duration.as_secs_f32()
    );

    systems.render.render_frame(&mut systems.world, &mut systems.time);

    systems.input.reset_input();

    systems.time.end_of_frame(); 


}

fn init(systems: &mut Engine) {
    let main_player = systems.world.add_and_spawn_new_player(
        InputMaster::LocalMaster(
            LocalMaster::new(ActionsFrameState::empty())
        ),
        systems.global_players_settings.clone()
    );

    systems.engine_handle.send_command(
        Command {
            sender: 0_u64,
            command_type: CommandType::SendMessage(
                main_player,
                Message::SetTransform(
                    Transform::new_from_vec4(systems.world.spawn_position),
                )
            )
        }
    );

    // systems.engine_handle.send_command(
    //     Command {
    //         sender: 0_u32,
    //         command_type: CommandType::SendMessage(
    //             main_player,
    //             Message::EnableCollider(
    //                 false
    //             )
    //         )
    //     }
    // );

    systems.world.main_camera_from = main_player;
}