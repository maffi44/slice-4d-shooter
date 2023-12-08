use std::time::Duration;
// use std::os::raw::c_void;

// use wasm_bindgen::JsCast;
// use wasm_bindgen::JsValue;
// use web_sys::Performance;

// use web_sys::console;

use instant::Instant;

/// Time System
pub struct TimeSystem {
    pub target_frame_duration: Duration,

    pub prev_frame_duration: f64,
    
    pub average_frame_duration: f64,

    // perf: web_sys::Performance,

    // js_window: web_sys::Window,

    average_frame_duration_delta: f64,

    pub frame_counter: u64,

    timestamp_of_start_of_current_frame: Instant,

    pub timestamp_of_start_of_main_loop: Instant,

    /// Specific for windows timer to imitate nanosleep in linux
    /// Need it because normal sleep in windows working with very low accuracy
    #[cfg(target_os = "windows")]
    windows_timer: isize,
}

impl TimeSystem {
    // pub fn get_avarage_frame_duration(&self) -> f64 {
    //     self.average_frame_duration
    // }

    // pub fn get_prev_frame_duration(&self) -> f64 {
    //     self.prev_frame_duration
    // }

    // pub fn get_target_frame_duration(&self) -> f64 {
    //     self.target_frame_duration
    // }

    pub fn new(target_frame_rate: u32) -> Self {

        // Configuration windows OS to increase process(or thread?) wake up timer
        // resolution for current process(or thread?) to get maximum update speed
        #[cfg(target_os = "windows")]
        unsafe {windows_sys::Win32::Media::timeBeginPeriod(1);}

        // let js_window = web_sys::window().expect("Can't get JS window");

        // let perf = js_window.performance().expect("Can't get performance obj from JS window");

        // let time = perf.now();

        // web_sys::console::log_1(&(time.into()));




        TimeSystem {
            target_frame_duration: Duration::from_secs_f64(1_f64 / target_frame_rate as f64),
            average_frame_duration_delta: 0.0_f64,
            average_frame_duration: 0.0_f64,
            prev_frame_duration: 0.0_f64,
            frame_counter: 0_u64,
            // js_window,
            // perf,
            timestamp_of_start_of_current_frame: Instant::now(),
            timestamp_of_start_of_main_loop: Instant::now(),
            
            //creating specific for windows timer to imitate nanosleep in linux
            #[cfg(target_os = "windows")]
            windows_timer: unsafe {
                windows_sys::Win32::System::Threading::CreateWaitableTimerW(
                    0 as *const windows_sys::Win32::Security::SECURITY_ATTRIBUTES,
                    1,
                    0 as *const u16
                )
            },
        }
    }

    pub fn init(&mut self) {
        // self.average_frame_duration = self.target_frame_duration;
        // self.prev_frame_duration = self.target_frame_duration ;
        self.timestamp_of_start_of_current_frame = Instant::now();
        self.timestamp_of_start_of_main_loop = Instant::now();
        self.frame_counter = 0_u64;
    }

    pub fn start_of_frame(&mut self) {
        // self.prev_frame_duration =
        //     self.timestamp_of_start_of_current_frame -
        //     self.timestamp_of_start_of_main_loop;

        self.timestamp_of_start_of_current_frame = Instant::now();

        

        // self.average_frame_duration =
        //     self.timestamp_of_start_of_main_loop / self.frame_counter as f64;
        
        // self.average_frame_duration_delta =
        //     self.average_frame_duration - self.target_frame_duration;

        // self.average_frame_duration_delta =
        //     (
        //         self.timestamp_of_start_of_current_frame -
        //         self.timestamp_of_start_of_main_loop
        //     ) - 
        //     self.frame_counter as f64 * self.target_frame_duration;

        // println!("dt: {}, av dt: {}, dur dt: {}",
        //     self.prev_frame_duration * 1000.0,
        //     self.average_frame_duration * 1000.0,
        //     self.average_frame_duration_delta * 1000.0,
        // );
        // println!("current time {}", self.timestamp_of_start_of_main_loop.elapsed().as_millis())

    }

    pub fn end_of_frame(&mut self) {

        self.frame_counter += 1_u64;

        // let duration_of_calculation_in_current_frame =
        //     self.perf.now() - 
        //     self.timestamp_of_start_of_current_frame;
        
        // Calculate thread sleeping time to make duration
        // of this frame is close to target frame duration
        // let sleeping_duration = (
        //         self.target_frame_duration -
        //         duration_of_calculation_in_current_frame -
        //         self.average_frame_duration_delta
        //     ).max(0.0_f64);
        
        // let ts = Instant::now();
        // println!("sleeping time: {}", ts.elapsed().as_secs_f32() * 1000.0);  
        
    }

    fn thread_sleep(&mut self, mut duration: f32) {
        #[cfg(target_os = "windows")]
        unsafe {
            // this is necessary because on windows in winit
            // after iteration of main loop will be a delay 
            // about 1ms and we need to substract these 1ms
            // from the sleep duration 
            duration = duration - 0.0001_f32;

            let delay = (duration * -10000000.0_f32).min(0.0_f32) as i64;

            if delay < 0_i64 {
                //schedule a timer wake-up time (in relative time)
                windows_sys::Win32::System::Threading::SetWaitableTimer(
                    self.windows_timer,
                    &(delay),
                    0,
                    None,
                    0 as *const c_void,
                    0,
                );
    
                //waiting for a timer to wake up
                windows_sys::Win32::System::Threading::WaitForSingleObject(
                    self.windows_timer,
                    !0 as u32,
                );
            }
        }

        // TODO: make nanosleep for linux, mac and wasm
        #[cfg(not(target_os = "windows"))]
        std::thread::sleep(Duration::from_secs_f32(duration))

    }
}

impl Drop for TimeSystem {
    fn drop(&mut self) {
        #[cfg(target_os = "windows")]
        unsafe {
            windows_sys::Win32::Foundation::CloseHandle(self.windows_timer);
        }
    }
}