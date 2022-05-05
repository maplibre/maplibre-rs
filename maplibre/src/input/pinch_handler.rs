use super::UpdateState;

use crate::map_state::ViewState;
use crate::{MapState, MapWindow};
use std::time::Duration;

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
