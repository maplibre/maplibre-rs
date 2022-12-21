//! Extracts data from the current state.

use crate::{
    context::MapContext,
    render::{eventually::Eventually::Initialized, RenderState, Renderer},
    schedule::Stage,
    world::World,
};

#[derive(Default)]
pub struct ExtractStage;

impl Stage for ExtractStage {
    fn run(
        &mut self,
        MapContext {
            world: World { view_state, .. },
            renderer:
                Renderer {
                    state:
                        RenderState {
                            tile_view_pattern,
                            buffer_pool,
                            raster_resources,
                            ..
                        },
                    ..
                },
            ..
        }: &mut MapContext,
    ) {
        let (Initialized(tile_view_pattern), Initialized(buffer_pool), Initialized(raster_resources)) =
            (tile_view_pattern, buffer_pool, raster_resources) else { return; };

        let view_region = view_state.create_view_region();

        if let Some(view_region) = &view_region {
            let zoom = view_state.zoom();
            //tile_view_pattern.update_pattern(view_region, buffer_pool.index(), zoom);
            tile_view_pattern.update_pattern(view_region, raster_resources, zoom);
        }
    }
}
