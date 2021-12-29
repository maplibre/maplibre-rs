use crate::platform::Instant;
use log::trace;

pub struct Measure {
    now: Instant,
}

impl Measure {
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

impl Drop for Measure {
    fn drop(&mut self) {
        self.breadcrumb("Drop");
    }
}
