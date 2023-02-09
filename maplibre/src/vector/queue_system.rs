//! Queues [PhaseItems](crate::render::render_phase::PhaseItem) for rendering.
use crate::{
    context::MapContext,
    ecs::tiles::Tile,
    render::{
        eventually::{Eventually, Eventually::Initialized},
        render_phase::{DrawState, LayerItem, RenderPhase, TileMaskItem},
    },
    vector::{
        render_commands::{DrawMasks, DrawVectorTiles},
        VectorBufferPool, VectorLayersIndicesComponent, WgpuTileViewPattern,
    },
};

pub fn queue_system(
    MapContext {
        world, renderer, ..
    }: &mut MapContext,
) {
    let (
        Initialized(tile_view_pattern),
        Initialized(buffer_pool),
        mask_phase,
        vector_tile_phase,
    ) = world.resources.query_mut::<(
            &mut Eventually<WgpuTileViewPattern>,
            &mut Eventually<VectorBufferPool>,
            &mut RenderPhase<TileMaskItem>,
            &mut RenderPhase<LayerItem>,
        )>()
        .unwrap() else { return; }; // FIXME tcs: Unwrap

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

            if let Some(layer_entries) = buffer_pool_index.get_layers(&source_shape.coords()) {
                for layer_entry in layer_entries {
                    // Draw tile
                    vector_tile_phase.add(LayerItem {
                        draw_function: Box::new(DrawState::<LayerItem, DrawVectorTiles>::new()),
                        index: layer_entry.style_layer.index,
                        style_layer: layer_entry.style_layer.id.clone(),
                        tile: Tile {
                            coords: layer_entry.coords,
                        },
                        source_shape: source_shape.clone(),
                    });

                    let Some(mut vector_layers_indices) =
                        world.tiles.query_mut::<&mut VectorLayersIndicesComponent>(layer_entry.coords) else { return; };

                    // FIXME tcs
                    vector_layers_indices.layers.push(layer_entry.clone());
                }
            };
        });
    }
}
