use crate::{render::Renderer, style::Style, world::World};

/// Stores the context of the map.
pub struct MapContext {
    pub style: Style,
    pub world: World,
    pub renderer: Renderer,
}

impl MapContext {
    pub fn resize(&mut self, width: u32, height: u32) {
        self.world.view_state.resize(width, height);
        self.renderer.resize(width, height)
    }
}
