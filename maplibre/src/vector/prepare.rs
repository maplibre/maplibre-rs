//! Queues [PhaseItems](crate::render::render_phase::PhaseItem) for rendering.

use crate::{
    context::MapContext,
    ecs::world::World,
    render::{
        eventually::{Eventually, Eventually::Initialized},
        resource::{IndexEntry, RasterResources},
        tile_view_pattern::{HasTile, TileViewPattern},
    },
    vector::{MaskRenderPhase, RasterTilePhase, VectorBufferPool, VectorTilePhase},
};

pub fn prepare_system(
    MapContext {
        world, renderer, ..
    }: &mut MapContext,
) {
    let mask_phase = world.get_resource_mut::<MaskRenderPhase>();
    mask_phase.clear();

    let vector_tile_phase = world.get_resource_mut::<VectorTilePhase>();
    vector_tile_phase.clear();

    let raster_tile_phase = world.get_resource_mut::<RasterTilePhase>();
    raster_tile_phase.clear();

    // TODO duplicate
    let (Initialized(tile_view_pattern), Initialized(buffer_pool), Initialized(raster_resources)) =
        (
            world.get_resource::<Eventually<TileViewPattern<wgpu::Queue, wgpu::Buffer>>>(),
            world.get_resource::<Eventually<VectorBufferPool>>(),
            world.get_resource::<Eventually<RasterResources>>(),
        ) else { return; };

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
