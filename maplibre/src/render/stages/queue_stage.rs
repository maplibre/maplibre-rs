//! Queues [PhaseItems](crate::render::render_phase::PhaseItem) for rendering.

use crate::{
    context::MapContext,
    render::{
        eventually::Eventually::Initialized, resource::IndexEntry, tile_view_pattern::TileInView,
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
                            tile_phase,
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

        if let (Initialized(tile_view_pattern), Initialized(buffer_pool)) =
            (tile_view_pattern, &buffer_pool)
        {
            let index = buffer_pool.index();

            for tile_in_view in tile_view_pattern.iter() {
                let TileInView { shape, fallback } = &tile_in_view;
                let coords = shape.coords;
                tracing::trace!("Drawing tile at {coords}");

                let shape_to_render = fallback.as_ref().unwrap_or(shape);

                // Draw mask
                mask_phase.add(tile_in_view.clone());

                if let Some(entries) = index.get_layers(&shape_to_render.coords) {
                    let mut layers_to_render: Vec<&IndexEntry> = Vec::from_iter(entries);
                    layers_to_render.sort_by_key(|entry| entry.style_layer.index);

                    for entry in layers_to_render {
                        // Draw tile
                        tile_phase.add((entry.clone(), shape_to_render.clone()))
                    }
                } else {
                    tracing::trace!("No layers found at {}", &shape_to_render.coords);
                }
            }
        }
    }
}
