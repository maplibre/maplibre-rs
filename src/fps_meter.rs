use std::time::Duration;

use log::info;

use crate::platform::Instant;

pub struct FPSMeter {
    next_report: Instant,
    frame_count: u32,
}

impl FPSMeter {
    pub fn new() -> Self {
        let start = Instant::now();
        Self {
            next_report: start + Duration::from_secs(1),
            frame_count: 0,
        }
    }

    pub fn update_and_print(&mut self) {
        self.frame_count += 1;
        let now = Instant::now();
        if now >= self.next_report {
            info!("{} FPS", self.frame_count);
            self.frame_count = 0;
            self.next_report = now + Duration::from_secs(1);
        }
    }
}
