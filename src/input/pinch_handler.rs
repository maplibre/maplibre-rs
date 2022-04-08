use super::UpdateState;


use crate::{MapState};
use std::time::Duration;

pub struct PinchHandler {}

impl UpdateState for PinchHandler {
    fn update_state<W>(&mut self, _state: &mut MapState<W>, _dt: Duration) {
        // TODO
    }
}

impl PinchHandler {
    pub fn new() -> Self {
        Self {}
    }
}
