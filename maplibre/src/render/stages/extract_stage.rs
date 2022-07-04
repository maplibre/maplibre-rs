//! Extracts data from the current state.

use crate::context::MapContext;
use crate::coords::ViewRegion;

use crate::render::eventually::Eventually::Initialized;
use crate::schedule::Stage;
use crate::{RenderState, Renderer};

#[derive(Default)]
pub struct ExtractStage;

impl Stage for ExtractStage {
    fn run(
        &mut self,
        MapContext {
            view_state,
            renderer:
                Renderer {
                    state:
                        RenderState {
                            mask_phase: _,
                            tile_phase: _,
                            tile_view_pattern,
                            buffer_pool,
                            ..
                        },
                    ..
                },
            ..
        }: &mut MapContext,
    ) {
        if let (Initialized(tile_view_pattern), Initialized(buffer_pool)) =
            (tile_view_pattern, &buffer_pool)
        {
            let visible_level = view_state.visible_level();

            let view_proj = view_state.view_projection();

            let view_region = view_state
                .camera
                .view_region_bounding_box(&view_proj.invert())
                .map(|bounding_box| {
                    ViewRegion::new(bounding_box, 0, *view_state.zoom, visible_level)
                });

            if let Some(view_region) = &view_region {
                let zoom = view_state.zoom();
                tile_view_pattern.update_pattern(view_region, buffer_pool, zoom);
            }
        }
    }
}
