use super::UpdateState;
use crate::render::render_state::RenderState;
use std::time::Duration;

pub struct PinchHandler {}

impl UpdateState for PinchHandler {
    fn update_state(&mut self, state: &mut RenderState, dt: Duration) {
        // TODO
    }
}

impl PinchHandler {
    pub fn new() -> Self {
        Self {}
    }
}
