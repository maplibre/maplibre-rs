use crate::{render::Renderer, style::Style, tcs::world::World};

/// Stores the context of the map.
///
/// This struct should not depend on the [`crate::environment::Environment`] trait. Else types
/// throughout the create get messy quickly.
pub struct MapContext {
    pub style: Style, // TODO: Move to ECS
    pub world: World,
    pub renderer: Renderer,
}

impl MapContext {
    pub fn resize(&mut self, width: u32, height: u32) {
        self.world.view_state.resize(width, height);
        self.renderer.resize(width, height)
    }
}
