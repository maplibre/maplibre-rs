use std::time::Duration;

use maplibre::world::ViewState;

use super::UpdateState;

pub struct PinchHandler {}

impl UpdateState for PinchHandler {
    fn update_state(&mut self, _state: &mut ViewState, _dt: Duration) {
        // TODO
    }
}

impl PinchHandler {
    pub fn new() -> Self {
        Self {}
    }
}
