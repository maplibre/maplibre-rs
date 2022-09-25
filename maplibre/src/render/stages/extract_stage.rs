//! Extracts data from the current state.

use crate::render::{RenderState, Renderer};
use crate::{
    context::MapContext, coords::ViewRegion, render::eventually::Eventually::Initialized,
    schedule::Stage,
};

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
            let view_region = view_state.create_view_region();

            if let Some(view_region) = &view_region {
                let zoom = view_state.zoom();
                tile_view_pattern.update_pattern(view_region, buffer_pool, zoom);
            }
        }
    }
}
