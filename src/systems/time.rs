use std::time::Duration;
use instant::Instant;
pub struct TimeSystem {
    pub target_frame_duration: Duration,
    pub prev_frame_duration: f64,
    pub average_frame_duration: f64,
    average_frame_duration_delta: f64,
    pub frame_counter: u64,
    timestamp_of_start_of_current_frame: Instant,
    pub timestamp_of_start_of_main_loop: Instant,

}

impl TimeSystem {

    pub fn new(target_frame_rate: u32) -> Self {
        TimeSystem {
            target_frame_duration: Duration::from_secs_f64(1_f64 / target_frame_rate as f64),
            average_frame_duration_delta: 0.0_f64,
            average_frame_duration: 0.0_f64,
            prev_frame_duration: 0.0_f64,
            frame_counter: 0_u64,
            timestamp_of_start_of_current_frame: Instant::now(),
            timestamp_of_start_of_main_loop: Instant::now(),
        }
    }

    pub fn init(&mut self) {
        self.timestamp_of_start_of_current_frame = Instant::now();
        self.timestamp_of_start_of_main_loop = Instant::now();
        self.frame_counter = 0_u64;
    }

    #[inline]
    pub fn start_of_frame(&mut self) {
        self.average_frame_duration =
            self.timestamp_of_start_of_main_loop.elapsed().as_secs_f64() / self.frame_counter as f64;
    }

    #[inline]
    pub fn end_of_frame(&mut self) {
        self.frame_counter += 1_u64;
    }
}