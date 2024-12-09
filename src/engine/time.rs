use std::time::Duration;
use web_time::Instant;

pub struct TimeSystem {
    pub target_frame_duration: Duration,
    pub prev_frame_duration: f32,
    pub average_frame_duration: f64,
    average_frame_duration_delta: f64,
    pub frame_counter: u64,
    timestamp_of_start_of_current_frame: web_time::Instant,
    pub timestamp_of_main_loop_start: web_time::Instant,

    server_time_in_millis: Option<u128>,
    joined_to_session_timestamp: Option<Instant>,
}

impl TimeSystem {

    pub fn new(target_frame_rate: u32) -> Self {
        TimeSystem {
            target_frame_duration: Duration::from_secs_f64(1_f64 / target_frame_rate as f64),
            average_frame_duration_delta: 0.0_f64,
            average_frame_duration: 0.0_f64,
            prev_frame_duration: 0.0_f32,
            frame_counter: 0_u64,
            timestamp_of_start_of_current_frame: Instant::now(),
            timestamp_of_main_loop_start: Instant::now(),
            server_time_in_millis: None,
            joined_to_session_timestamp: None
        }
    }

    pub fn init(&mut self) {
        self.timestamp_of_start_of_current_frame = Instant::now();
        self.timestamp_of_main_loop_start = Instant::now();
        self.frame_counter = 0_u64;
    }

    pub fn get_server_time(&self) -> u128
    {
        if self.server_time_in_millis.is_some()
        {
            self.joined_to_session_timestamp
                .expect("ERROR: have not connected_to_server_time timestamp but have server_start_time_in_millis")
                .elapsed()
                .as_millis()
                +
                self.server_time_in_millis.unwrap()
        }
        else
        {
            0_u128
        }
    }

    pub fn set_server_time(&mut self, start_time: u128)
    {
        self.joined_to_session_timestamp = Some(Instant::now());
        self.server_time_in_millis = Some(start_time);
    }

    #[inline]
    pub fn start_of_frame(&mut self) {
        self.average_frame_duration =
            self.timestamp_of_main_loop_start.elapsed().as_secs_f64() / self.frame_counter as f64;
        
        log::info!("avarange frame duration is {}", self.average_frame_duration);
        self.prev_frame_duration = self.timestamp_of_start_of_current_frame.elapsed().as_secs_f32();
        self.timestamp_of_start_of_current_frame = Instant::now();
    }

    #[inline]
    pub fn end_of_frame(&mut self) {
        self.frame_counter += 1_u64;
    }
}