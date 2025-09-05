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

use std::time::Duration;
use web_time::Instant;

pub struct TimeSystem {
    pub target_frame_duration: Duration,
    prev_frame_duration: f32,
    pub average_frame_duration: f64,
    average_frame_duration_delta: f64,
    pub frame_counter: u64,
    timestamp_of_start_of_current_frame: web_time::Instant,
    pub timestamp_of_main_loop_start: web_time::Instant,
    pub current_frame_duration: f64,

    server_time_in_millis: Option<u128>,
    joined_to_session_timestamp: Option<Instant>,
}

impl TimeSystem {

    pub fn new(target_frame_rate: u32) -> Self {

        // #[cfg(target_os = "windows")]
        // unsafe {windows_sys::Win32::Media::timeBeginPeriod(1);}
        
        TimeSystem {
            target_frame_duration: Duration::from_secs_f64(1_f64 / target_frame_rate as f64),
            average_frame_duration_delta: 0.0_f64,
            average_frame_duration: 0.0_f64,
            prev_frame_duration: 0.0_f32,
            frame_counter: 0_u64,
            timestamp_of_start_of_current_frame: Instant::now(),
            timestamp_of_main_loop_start: Instant::now(),
            server_time_in_millis: None,
            joined_to_session_timestamp: None,
            current_frame_duration: 0.0_f64,
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

    #[inline]
    pub fn set_server_time(&mut self, start_time: u128)
    {
        self.joined_to_session_timestamp = Some(Instant::now());
        self.server_time_in_millis = Some(start_time);
    }

    #[inline]
    pub fn start_of_frame(&mut self) {
        self.average_frame_duration =
            self.timestamp_of_main_loop_start.elapsed().as_secs_f64() / self.frame_counter as f64;
        
        // println!("avarange frame duration is {}", self.average_frame_duration);
        self.prev_frame_duration = self.timestamp_of_start_of_current_frame.elapsed().as_secs_f32();
        self.timestamp_of_start_of_current_frame = Instant::now();
    }

    pub fn get_prev_frame_duration(&self) -> f32
    {
        if self.prev_frame_duration > 0.35
        {
            0.0166666
        }
        else
        {
            self.prev_frame_duration
        }
    }

    #[inline]
    pub fn end_of_frame(&mut self) {
        self.frame_counter += 1_u64;
        self.current_frame_duration = self.timestamp_of_start_of_current_frame.elapsed().as_secs_f64();
    }
}