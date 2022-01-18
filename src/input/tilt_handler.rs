use super::UpdateState;
use crate::render::render_state::RenderState;
use std::time::Duration;

pub struct TiltHandler {
    speed: f64,
    sensitivity: f64,
}

impl UpdateState for TiltHandler {
    fn update_state(&mut self, state: &mut RenderState, dt: Duration) {
        // TODO
    }
}

impl TiltHandler {
    pub fn new(speed: f64, sensitivity: f64) -> Self {
        Self { speed, sensitivity }
    }
}
