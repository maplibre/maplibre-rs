use super::UpdateState;
use crate::io::tile_cache::TileCache;
use crate::render::render_state::RenderState;
use std::time::Duration;

pub struct PinchHandler {}

impl UpdateState for PinchHandler {
    fn update_state(&mut self, _state: &mut RenderState, _tile_cache: &TileCache, _dt: Duration) {
        // TODO
    }
}

impl PinchHandler {
    pub fn new() -> Self {
        Self {}
    }
}
