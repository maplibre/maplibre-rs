use crate::{
    render::{view_state::ViewState, Renderer},
    style::Style,
    tcs::world::World,
    window::PhysicalSize,
};

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
    pub fn resize(&mut self, size: PhysicalSize, scale_factor: f64) {
        self.view_state.resize(size.to_logical(scale_factor));
        self.renderer.resize_surface(size)
    }
}
