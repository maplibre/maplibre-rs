use std::time::Duration;

use maplibre::context::MapContext;

use super::UpdateState;

#[derive(Default)]
pub struct PinchHandler {}

impl UpdateState for PinchHandler {
    fn update_state(&mut self, _map_context: &mut MapContext, _dt: Duration) {
        // TODO
    }
}
