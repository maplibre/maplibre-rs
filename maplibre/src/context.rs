use crate::{render::Renderer, style::Style, world::World};

/// Stores the context of the map.
pub struct MapContext {
    pub style: Style,
    pub world: World,
    pub renderer: Renderer,
}
