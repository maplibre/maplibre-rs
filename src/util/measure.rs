use crate::platform::Instant;
use log::trace;

pub struct TimeMeasure {
    now: Instant,
}

impl TimeMeasure {
    pub fn time() -> Self {
        Self {
            now: Instant::now(),
        }
    }

    pub fn breadcrumb(&mut self, name: &'static str) {
        trace!(
            "Measurement \"{}\": {}ms",
            name,
            self.now.elapsed().as_millis()
        );
        self.now = Instant::now();
    }
}

impl Drop for TimeMeasure {
    fn drop(&mut self) {
        self.breadcrumb("Drop");
    }
}
