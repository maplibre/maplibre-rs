//! Queues [PhaseItems](crate::render::render_phase::PhaseItem) for rendering.

use crate::context::MapContext;
use crate::coords::{ViewRegion, Zoom};
use crate::io::tile_cache::TileCache;
use crate::io::LayerTessellateMessage;
use crate::render::camera::ViewProjection;
use crate::render::resource::IndexEntry;
use crate::render::shaders::{
    ShaderCamera, ShaderFeatureStyle, ShaderGlobals, ShaderLayerMetadata, Vec4f32,
};
use crate::render::tile_view_pattern::TileInView;
use crate::render::util::Eventually::Initialized;
use crate::schedule::Stage;
use crate::{RenderState, Renderer, Style};
use std::iter;

#[derive(Default)]
pub struct QueueStage;

impl Stage for QueueStage {
    #[tracing::instrument(name = "QueueStage", skip_all)]
    fn run(
        &mut self,
        MapContext {
            view_state,
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
