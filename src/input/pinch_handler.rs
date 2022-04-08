use super::UpdateState;

use crate::render::render_state::RenderState;
use crate::{MapState, Scheduler};
use std::time::Duration;

pub struct PinchHandler {}

impl UpdateState for PinchHandler {
    fn update_state<W>(&mut self, state: &mut MapState<W>, dt: Duration) {
        // TODO
    }
}

impl PinchHandler {
    pub fn new() -> Self {
        Self {}
    }
}
