use std::time::{Duration, Instant};

pub struct FPSMeter {
    start: Instant,
    next_report: Instant,
    frame_count: u32,
    pub time_secs: f32,
}

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
            println!("{} FPS", self.frame_count);
            self.frame_count = 0;
            self.next_report = now + Duration::from_secs(1);
        }
    }
}
