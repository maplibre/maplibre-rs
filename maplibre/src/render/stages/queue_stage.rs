//! Queues [PhaseItems](crate::render::render_phase::PhaseItem) for rendering.

use crate::{
    context::MapContext,
    render::{eventually::Eventually::Initialized, resource::IndexEntry, RenderState, Renderer},
    schedule::Stage,
};

#[derive(Default)]
pub struct QueueStage;

impl Stage for QueueStage {
    #[tracing::instrument(name = "QueueStage", skip_all)]
    fn run(
        &mut self,
        MapContext {
            renderer:
                Renderer {
                    state:
                        RenderState {
                            mask_phase,
                            tile_phase,
                            symbol_buffer_pool,
                            symbol_tile_phase,
                            tile_view_pattern,
                            buffer_pool,
                            ..
                        },
                    ..
                },
            ..
        }: &mut MapContext,
    ) {
        mask_phase.items.clear();
        tile_phase.items.clear();
        symbol_tile_phase.items.clear();

        let (Initialized(tile_view_pattern), Initialized(buffer_pool), Initialized(symbol_buffer_pool)) =
            (tile_view_pattern, &buffer_pool, symbol_buffer_pool) else { return; };

        let buffer_pool_index = buffer_pool.index();
        let symbol_pool_index = symbol_buffer_pool.index();

        for view_tile in tile_view_pattern.iter() {
            let coords = &view_tile.coords();
            tracing::trace!("Drawing tile at {coords}");

            // draw tile normal or the source e.g. parent or children
            view_tile.render(|source_shape| {
                // Draw masks for all source_shapes
                mask_phase.add(source_shape.clone());

                if let Some(entries) = buffer_pool_index.get_layers(&source_shape.coords()) {
                    let mut layers_to_render: Vec<&IndexEntry> = Vec::from_iter(entries);
                    layers_to_render.sort_by_key(|entry| entry.style_layer.index);

                    for entry in layers_to_render {
                        // Draw tile
                        tile_phase.add((entry.clone(), source_shape.clone()));
                    }
                };

                if let Some(entries) = symbol_pool_index.get_layers(&source_shape.coords()) {
                    log::info!("queueing {:?}", &source_shape.coords());

                    let mut layers_to_render: Vec<&IndexEntry> = Vec::from_iter(entries);
                    layers_to_render.sort_by_key(|entry| entry.style_layer.index);

                    for entry in layers_to_render {
                        // Draw tile
                        symbol_tile_phase.add((entry.clone(), source_shape.clone()));
                    }
                };
            });
        }
    }
}
