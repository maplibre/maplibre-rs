use std::time::{Duration, Instant};

#[cfg(target_arch = "wasm32")]
use js_sys;
use log::{info, trace};

#[cfg(target_arch = "wasm32")]
pub struct FPSMeter {
    start: f64,
    next_report: f64,
    frame_count: u32,
    pub time_secs: f64,
}

#[cfg(target_arch = "wasm32")]
impl FPSMeter {
    pub fn new() -> Self {
        let start = (js_sys::Date::now() / 1000.0) as f64;
        Self {
            start,
            next_report: start + 1.0,
            frame_count: 0,
            time_secs: 0.0,
        }
    }

    pub fn update_and_print(&mut self) {
        self.frame_count += 1;
        let now = (js_sys::Date::now() / 1000.0) as f64;
        self.time_secs = now - self.start;
        if now >= self.next_report {
            info!("{} FPS", self.frame_count);
            self.frame_count = 0;
            self.next_report = now + 1.0;
        }
    }
}


#[cfg(not(target_arch = "wasm32"))]
pub struct FPSMeter {
    start: Instant,
    next_report: Instant,
    frame_count: u32,
    pub time_secs: f32,
}

#[cfg(not(target_arch = "wasm32"))]
impl FPSMeter {
    pub fn new() -> Self {
        let start = Instant::now();
        Self {
            start,
            next_report: start + Duration::from_secs(1),
            frame_count: 0,
            time_secs: 0.0,
        }
    }

    pub fn update_and_print(&mut self) {
        self.frame_count += 1;
        let now = Instant::now();
        self.time_secs = (now - self.start).as_secs_f32();
        if now >= self.next_report {
            info!("{} FPS", self.frame_count);
            self.frame_count = 0;
            self.next_report = now + Duration::from_secs(1);
        }
    }
}
