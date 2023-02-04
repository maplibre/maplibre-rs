//! Extracts data from the current state.

use std::ops::Deref;

use crate::{
    context::MapContext,
    ecs::world::World,
    render::{
        eventually::{Eventually, Eventually::Initialized},
        resource::RasterResources,
        tile_view_pattern::TileViewPattern,
        RenderState, Renderer,
    },
    schedule::Stage,
    vector::VectorBufferPool,
};

pub fn tile_view_pattern_system(
    MapContext {
        world, renderer, ..
    }: &mut MapContext,
) {
    // TODO duplicate
    let (Initialized(tile_view_pattern), Initialized(buffer_pool), Initialized(raster_resources)) =
        (
            world.get_resource_mut::<Eventually<TileViewPattern<wgpu::Queue, wgpu::Buffer>>>(),
            world.get_resource::<Eventually<VectorBufferPool>>(),
            world.get_resource::<Eventually<RasterResources>>(),
        ) else { return; };

    let view_state = &world.view_state;

    let view_region = view_state.create_view_region();

    if let Some(view_region) = &view_region {
        let zoom = view_state.zoom();
        //tile_view_pattern.update_pattern(view_region, buffer_pool.index(), zoom);
        tile_view_pattern.update_pattern(
            view_region,
            &(raster_resources.deref(), buffer_pool.index()),
            zoom,
        );
    }
}
