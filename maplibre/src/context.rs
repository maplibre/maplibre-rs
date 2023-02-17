use crate::{render::Renderer, style::Style, tcs::world::World, view_state::ViewState};

/// Stores the context of the map.
///
/// This struct should not depend on the [`crate::environment::Environment`] trait. Else types
/// throughout the crate get messy quickly.
pub struct MapContext {
    pub style: Style,
    pub world: World,
    pub view_state: ViewState,
    pub renderer: Renderer,
}

impl MapContext {
    pub fn resize(&mut self, width: u32, height: u32) {
        self.view_state.resize(width, height);
        self.renderer.resize_surface(width, height)
    }
}
