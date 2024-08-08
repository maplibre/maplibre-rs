//! Queues [PhaseItems](crate::render::render_phase::PhaseItem) for rendering.
use crate::{
    context::MapContext,
    render::{
        eventually::{Eventually, Eventually::Initialized},
        render_phase::{DrawState, RenderPhase, TranslucentItem},
        tile_view_pattern::WgpuTileViewPattern,
    },
    sdf::{render_commands::DrawSymbols, SymbolBufferPool},
    tcs::tiles::Tile,
};

pub fn queue_system(MapContext { world, .. }: &mut MapContext) {
    let Some((Initialized(tile_view_pattern), translucent_phase, Initialized(symbol_buffer_pool))) =
        world.resources.query_mut::<(
            &mut Eventually<WgpuTileViewPattern>,
            &mut RenderPhase<TranslucentItem>,
            &mut Eventually<SymbolBufferPool>,
        )>()
    else {
        return;
    };

    for view_tile in tile_view_pattern.iter() {
        let coords = &view_tile.coords();
        tracing::trace!("Drawing tile at {coords}");

        // draw tile normal or the source e.g. parent or children
        view_tile.render(|source_shape| {
            if let Some(layer_entries) =
                symbol_buffer_pool.index().get_layers(source_shape.coords())
            {
                for layer_entry in layer_entries {
                    // Draw tile
                    translucent_phase.add(TranslucentItem {
                        draw_function: Box::new(DrawState::<TranslucentItem, DrawSymbols>::new()),
                        index: layer_entry.style_layer.index,
                        style_layer: layer_entry.style_layer.id.clone(),
                        tile: Tile {
                            coords: layer_entry.coords,
                        },
                        source_shape: source_shape.clone(),
                    });
                }
            };
        });
    }
}
