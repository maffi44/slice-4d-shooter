mod renderer;

use renderer::Renderer;

use wgpu::SurfaceError;

use super::engine::Engine;

use std::{
    sync::{
        Mutex,
        Arc,
        mpsc,
        mpsc::{Sender, Receiver},
    },
    time::Duration
};

use winit::{window::Window, dpi::PhysicalSize};

#[derive(Clone, Copy)]
pub struct FrameRenderData {

}

impl FrameRenderData {
    pub fn new() -> Self {
        FrameRenderData {

        }
    }
}

pub struct RenderSystem {
    // current_render_data: Arc<Mutex<Option<FrameRenderData>>>,
    // render_result_reciever: Receiver<Result<(), SurfaceError>>,
    // resize_request_sender: Sender<winit::dpi::PhysicalSize<u32>>,
    window: Window,
    renderer: Renderer,
}

impl RenderSystem {

    // #[inline]
    // pub fn resize_buffers(&mut self) {
    //     let res = self.resize_request_sender.send(self.window.inner_size());

    //     match res {
    //         Ok(()) => {},
    //         Err(e) => {
    //             panic!("Resize render thread channel is disconnected")
    //         }
    //     }
    // }

    // #[inline]
    // pub fn send_data_to_renderer(systems: &mut Engine, frame_data: FrameRenderData) {
    //     // first check status of previous rendered frame
    //     let render_result: Result<Result<(), SurfaceError>, mpsc::TryRecvError> = systems.render.render_result_reciever.try_recv();

    //     match render_result {
    //         Ok(result) => {
    //             match result {
    //                 Ok(()) => {
    //                     // Everything is ok, previous frame is rendered, let's move on
    //                 },
    //                 // The window size has probably been changed. Change the size of the target rendering buffers.
    //                 Err(wgpu::SurfaceError::Lost) => systems.render.resize_buffers(),
    //                 // The system is out of memory, we should probably quit
    //                 Err(wgpu::SurfaceError::OutOfMemory) => panic!("Out of GPU memory"),
    //                 // All other errors (Outdated, Timeout) should be resolved by the next frame
    //                 Err(e) => eprintln!("{:?}", e),
    //             }
    //         },
    //         Err(error) => {
    //             match error {
    //                 mpsc::TryRecvError::Empty => {
    //                     // It means previous render still rendering. No problem we will still send new frame_data to the render thread
    //                 },
    //                 mpsc::TryRecvError::Disconnected => {
    //                     panic!("Disconnected render_result mpsc channel between main and render threads")
    //                 },

    //             }
    //         }
    //     }
    //     // Then we clone last calculated frame data into Mutex. This cloned Mutex will be checked 
    //     // in render async task and rendered when wgpu_state will be ready to render next frame.
    //     // If wgpu does not have time render this frame_data this frame_data will be skiped.
    //     *(systems.render.current_render_data.lock().unwrap()) = Some(frame_data);

    // }

    pub async fn new(
        window: Window,
    ) -> Self {
        let renderer = Renderer::new(&window).await;

        // let current_render_data = Arc::new(Mutex::new(Some(FrameRenderData::new())));
        // let current_render_data_clone = current_render_data.clone();


        // let (render_result_sender, render_result_reciever) =
        //     mpsc::channel::<Result<(), SurfaceError>>();

        // let (resize_request_sender, resize_request_reciever) =
        //     mpsc::channel::<winit::dpi::PhysicalSize<u32>>();

        
            
        // std::thread::spawn(move || {
                
        //     //Thread configuration
            
        //     // Configuration windows OS to increase process(or thread?) wake up timer
        //     // resolution for current process(or thread?) to get maximum update speed
        //     // #[cfg(target_os = "windows")]
        //     // unsafe {windows_sys::Win32::Media::timeBeginPeriod(1);}
            
        //     // #[cfg(target_os = "windows")]
        //     // let windows_timer = unsafe {
        //     //     windows_sys::Win32::System::Threading::CreateWaitableTimerW(
        //     //         0 as *const windows_sys::Win32::Security::SECURITY_ATTRIBUTES,
        //     //         1,
        //     //         0 as *const u16
        //     //     )
        //     // };
            
        //     let mut frame_counter = 0u64;
        //     let time = std::time::Instant::now();

        //     loop {
        //         if renderer.device.poll(wgpu::MaintainBase::Poll) {
        //             if let Some(data) = current_render_data_clone.lock().unwrap().take() {
        //                 if let Ok(new_size)= resize_request_reciever.try_recv() {
        //                     renderer.resize(new_size);
        //                 }
                        
        //                 //submit render data to the render queue (render frame)
        //                 let result = renderer.render(data);
        //                 if result.is_ok() {
        //                     frame_counter += 1u64;
        //                     let av_frame_dur = time.elapsed().as_secs_f32() / frame_counter as f32;
        //                     // println!("av render time {}", av_frame_dur * 1000.0);
        //                 }
    
        //                 if let Err(err) = render_result_sender.send(result) {
        //                     panic!("{}", err);
        //                 }
        //             }
        //         }


        //         // 500 micros delay before next iteration of the loop
        //         #[cfg(target_os = "windows")]
        //         unsafe {
        //             if delay < 0_i64 {
        //                 //schedule a timer wake-up time (in relative time)
        //                 windows_sys::Win32::System::Threading::SetWaitableTimer(
        //                     windows_timer,
        //                     &(-5000_i64),
        //                     0,
        //                     None,
        //                     0 as *const c_void,
        //                     0,
        //                 );
            
        //                 //waiting for a timer to wake up
        //                 windows_sys::Win32::System::Threading::WaitForSingleObject(
        //                     windows_timer,
        //                     !0 as u32,
        //                 );
        //             }
        //         }

        //         // TODO: make nanosleep for linux, mac and wasm
        //         #[cfg(not(target_os = "windows"))]
        //         std::thread::sleep(std::time::Duration::from_micros(500))
        //     }
        // });

        RenderSystem {
            // current_render_data,
            // resize_request_sender,
            // render_result_reciever,
            window,
            renderer,
        }
    }

    pub fn render_frame(&mut self, data: FrameRenderData) {
        self.renderer.render(data);
    }

    pub fn resize_frame_buffer(&mut self) {
        self.renderer.resize(self.window.inner_size());
    }

}