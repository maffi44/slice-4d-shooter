use super::{
    engine::Engine,
    input::ActionsFrameState,
    player::{
        Message,
        player_input_master::{
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
};

pub struct 
MainLoop {
    pub event_loop: EventLoop<()>
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
        let main_player = systems.world.add_and_spawn_new_player(
            InputMaster::LocalMaster(
                LocalMaster::new(ActionsFrameState::empty())
            )
        );

        systems.engine_handle.send_command(
            Command {
                sender: 0_u32,
                command_type: CommandType::SendMessage(
                    main_player,
                    Message::SetTransform(
                        Transform::new(0.0, 2.0, 0.0, 0.0),
                    )
                )
            }
        );

        systems.world.main_camera_from = main_player;

        let systems = &mut systems;

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
                        
                        WindowEvent::KeyboardInput {event,..} => { 
                            systems.input.set_keyboard_input(event);
                        },

                        WindowEvent::MouseInput {button, state,..} => {
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

    systems.input.reset_axis_input();

    systems.time.end_of_frame(); 


}