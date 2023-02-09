//! Describes the concept of a [`RenderPhase`] and [`PhaseItem`]

mod draw;

pub use draw::*;

use crate::{ecs::tiles::Tile, render::tile_view_pattern::TileShape};

/// A resource to collect and sort draw requests for specific [`PhaseItems`](PhaseItem).
pub struct RenderPhase<I: PhaseItem> {
    pub items: Vec<I>,
}

impl<I: PhaseItem> Default for RenderPhase<I> {
    fn default() -> Self {
        Self { items: Vec::new() }
    }
}

impl<I: PhaseItem> RenderPhase<I> {
    /// Adds a [`PhaseItem`] to this render phase.
    #[inline]
    pub fn add(&mut self, item: I) {
        self.items.push(item);
    }

    /// Sorts all of its [`PhaseItems`](PhaseItem).
    pub fn sort(&mut self) {
        self.items.sort_by_key(|d| d.sort_key());
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }
}

pub struct LayerItem {
    pub draw_function: Box<dyn Draw<LayerItem>>,
    pub index: u32,

    pub style_layer: String,

    pub tile: Tile,
    pub source_shape: TileShape,
}

impl PhaseItem for LayerItem {
    type SortKey = u32;

    #[inline]
    fn sort_key(&self) -> Self::SortKey {
        self.index
    }

    #[inline]
    fn draw_function(&self) -> &dyn Draw<LayerItem> {
        self.draw_function.as_ref()
    }
}

pub struct TileMaskItem {
    pub draw_function: Box<dyn Draw<TileMaskItem>>,
    pub source_shape: TileShape,
}

impl PhaseItem for TileMaskItem {
    type SortKey = u32;

    #[inline]
    fn sort_key(&self) -> Self::SortKey {
        0
    }

    #[inline]
    fn draw_function(&self) -> &dyn Draw<TileMaskItem> {
        self.draw_function.as_ref()
    }
}
