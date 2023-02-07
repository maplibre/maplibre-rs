//! Queues [PhaseItems](crate::render::render_phase::PhaseItem) for rendering.

use log::info;

use crate::{
    context::MapContext,
    ecs::world::Tile,
    render::{
        eventually::{Eventually, Eventually::Initialized},
        render_phase::{DrawState, LayerItem, RenderPhase, TileMaskItem},
        tile_view_pattern::TileViewPattern,
    },
    vector::{
        render_commands::{DrawMasks, DrawVectorTiles},
        VectorBufferPool, VectorLayersComponent,
    },
};

pub fn queue_system(
    MapContext {
        world, renderer, ..
    }: &mut MapContext,
) {
    // TODO duplicate
    let collection = world.resources.collect_mut4::<
        Eventually<TileViewPattern<wgpu::Queue, wgpu::Buffer>>,
        Eventually<VectorBufferPool>,
        RenderPhase<TileMaskItem>,
        RenderPhase<LayerItem>,
    >().unwrap();

    let (
        Initialized(tile_view_pattern),
        Initialized(buffer_pool),
        mask_phase,
        vector_tile_phase,
    ) = collection else { return; };

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
                    })
                }
            };
        });
    }
}
