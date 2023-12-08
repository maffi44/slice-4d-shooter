// Make it simple!
// use web_time::{Instant, Duration};

use super::{
    engine::Engine,
    render_system::{
        RenderSystem,
        FrameRenderData,
    },
};

use instant::Instant;

use std::time::Duration;

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
            event_loop: EventLoop::new(),
        }
    }

    pub async fn run(
        self,
        mut systems : Engine,
        // mut runtime: RuntimeSystem,
    ) {
        #[cfg(debug_assertions)]
        {
            #[cfg(target_arch = "wasm32")]
            {
                std::panic::set_hook(Box::new(console_error_panic_hook::hook));
                // console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
            }
            // #[cfg(not(target_arch = "wasm32"))]
            // env_logger::init(); //need for wgpu logging
        }
        
        let _ = self.event_loop.run(move |event, _, cntrl_flow| {

            
            let systems = &mut systems;

            let mut ready_to_engine_tick = true;
            
            match event {
                Event::NewEvents(cause) => {
                    match cause {
                        StartCause::Init => {
                            systems.time.init();

                            *cntrl_flow = ControlFlow::WaitUntil(
                                Instant::now() + systems.time.target_frame_duration
                            );
                        }
                        StartCause::ResumeTimeReached {
                            start,
                            requested_resume
                        } => {
                            ready_to_engine_tick = true;

                            // set wake up time gof the next interation
                            *cntrl_flow = ControlFlow::WaitUntil(
                                Instant::from(
                                    systems.time.timestamp_of_start_of_main_loop +
                                    Duration::from_secs_f64(
                                        systems.time.target_frame_duration.as_secs_f64() *
                                        (systems.time.frame_counter + 1) as f64
                                    )
                                )
                            )
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
                Event::MainEventsCleared => {
                    // engine main loop here
                    if ready_to_engine_tick {
                        ready_to_engine_tick = false;
                        
                        main_loop(systems);
                        *cntrl_flow = 
                            ControlFlow::WaitUntil(
                                Instant::now() + systems.time.target_frame_duration
                            );
                    };
                }
                Event::WindowEvent {
                    ref event,
                    ..
                } => {
                    match event {
                        WindowEvent::CloseRequested => *cntrl_flow = ControlFlow::Exit,

                        WindowEvent::Resized(_) => {
                            systems.render.resize_frame_buffer();
                        },

                        WindowEvent::ScaleFactorChanged {..} => {
                            systems.render.resize_frame_buffer();
                        },
                        
                        WindowEvent::KeyboardInput {input,..} => {
                            systems.input.get_keyboard_input(input);
                        },

                        WindowEvent::MouseInput {button, state,..} => {
                            systems.input.get_mouse_button_input(button, state);
                        },
                        _ => {},
                    }
                }
                
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

    systems.render.render_frame(FrameRenderData::new());

    systems.time.end_of_frame(); 

}