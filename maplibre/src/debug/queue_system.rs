//! Queues [PhaseItems](crate::render::render_phase::PhaseItem) for rendering.
use crate::{
    context::MapContext,
    debug::{render_commands::DrawDebugOutlines, TileDebugItem},
    render::{
        eventually::{Eventually, Eventually::Initialized},
        render_phase::{DrawState, RenderPhase},
        tile_view_pattern::WgpuTileViewPattern,
    },
};

pub fn queue_system(MapContext { world, .. }: &mut MapContext) {
    let Some((Initialized(tile_view_pattern), tile_debug_phase)) = world.resources.query_mut::<(
        &mut Eventually<WgpuTileViewPattern>,
        &mut RenderPhase<TileDebugItem>,
    )>() else {
        return;
    };

    for view_tile in tile_view_pattern.iter() {
        let coords = &view_tile.coords();
        tracing::trace!("Drawing debug at {coords}");

        // draw tile normal or the source e.g. parent or children
        view_tile.render(|source_shape| {
            // Draw masks for all source_shapes
            tile_debug_phase.add(TileDebugItem {
                draw_function: Box::new(DrawState::<TileDebugItem, DrawDebugOutlines>::new()),
                source_shape: source_shape.clone(),
            });
        });
    }
}
