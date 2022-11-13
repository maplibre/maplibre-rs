use std::time::Duration;

use instant::Instant;

/// Measures the frames per second.
///
/// # Example
/// ```
/// use maplibre::util::FPSMeter;
///
/// let mut meter = FPSMeter::new();
///
/// // call the following the the render loop
/// meter.update_and_print();
/// ```
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
            log::info!("{} FPS", self.frame_count);
            self.frame_count = 0;
            self.next_report = now + Duration::from_secs(1);
        }
    }
}

impl Default for FPSMeter {
    fn default() -> Self {
        Self::new()
    }
}
