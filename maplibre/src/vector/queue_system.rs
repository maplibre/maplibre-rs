//! Queues [PhaseItems](crate::render::render_phase::PhaseItem) for rendering.
use crate::{
    context::MapContext,
    ecs::tiles::Tile,
    render::{
        eventually::{Eventually, Eventually::Initialized},
        render_phase::{DrawState, LayerItem, RenderPhase, TileDebugItem, TileMaskItem},
        tile_view_pattern::WgpuTileViewPattern,
    },
    vector::{
        render_commands::{DrawDebugOutlines, DrawMasks, DrawVectorTiles},
        VectorBufferPool, VectorLayersIndicesComponent,
    },
};

pub fn queue_system(
    MapContext {
        world, renderer, ..
    }: &mut MapContext,
) {
    let Some((
        Initialized(tile_view_pattern),
        Initialized(buffer_pool),
        mask_phase,
        layer_item_phase,
        tile_debug_phase,
    )) = world.resources.query_mut::<(
        &mut Eventually<WgpuTileViewPattern>,
        &mut Eventually<VectorBufferPool>,
        &mut RenderPhase<TileMaskItem>,
        &mut RenderPhase<LayerItem>,
        &mut RenderPhase<TileDebugItem>,
    )>() else { return; };

    let buffer_pool_index = buffer_pool.index();

    for view_tile in tile_view_pattern.iter() {
        let coords = &view_tile.coords();
        tracing::trace!("Drawing tile at {coords}");

        // draw tile normal or the source e.g. parent or children
        view_tile.render(|source_shape| {
            // Draw masks for all source_shapes
            mask_phase.add(TileMaskItem {
                draw_function: Box::new(DrawState::<TileMaskItem, DrawMasks>::new()),
                source_shape: source_shape.clone(),
            });

            tile_debug_phase.add(TileDebugItem {
                draw_function: Box::new(DrawState::<TileDebugItem, DrawDebugOutlines>::new()),
                source_shape: source_shape.clone(),
            });

            if let Some(layer_entries) = buffer_pool_index.get_layers(source_shape.coords()) {
                for layer_entry in layer_entries {
                    // Draw tile
                    layer_item_phase.add(LayerItem {
                        draw_function: Box::new(DrawState::<LayerItem, DrawVectorTiles>::new()),
                        index: layer_entry.style_layer.index,
                        style_layer: layer_entry.style_layer.id.clone(),
                        tile: Tile {
                            coords: layer_entry.coords,
                        },
                        source_shape: source_shape.clone(),
                    });

                    let Some(mut vector_layers_indices) = world
                        .tiles
                        .query_mut::<&mut VectorLayersIndicesComponent>(layer_entry.coords) else { return; };

                    // FIXME tcs: Should be down in upload?
                    vector_layers_indices.layers.push(layer_entry.clone());
                }
            };
        });
    }
}
