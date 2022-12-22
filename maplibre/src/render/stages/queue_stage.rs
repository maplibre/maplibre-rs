//! Queues [PhaseItems](crate::render::render_phase::PhaseItem) for rendering.

use crate::{
    context::MapContext,
    render::{
        eventually::Eventually::Initialized, resource::IndexEntry, tile_view_pattern::HasTile,
        RenderState, Renderer,
    },
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
                            vector_tile_phase,
                            raster_tile_phase,
                            tile_view_pattern,
                            raster_resources,
                            buffer_pool,
                            ..
                        },
                    ..
                },
            ..
        }: &mut MapContext,
    ) {
        mask_phase.items.clear();
        vector_tile_phase.items.clear();
        raster_tile_phase.items.clear();

        let (Initialized(tile_view_pattern), Initialized(buffer_pool), Initialized(raster_resources)) =
            (tile_view_pattern, &buffer_pool, raster_resources) else { return; };

        let buffer_pool_index = buffer_pool.index();

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
                        vector_tile_phase.add((entry.clone(), source_shape.clone()));
                    }
                };

                if raster_resources.has_tile(&source_shape.coords()) {
                    raster_tile_phase.add(source_shape.clone())
                }
            });
        }
    }
}
