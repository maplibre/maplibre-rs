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
    // TODO duplicate
    let collection = world.resources.collect_mut6::<
        Eventually<TileViewPattern<wgpu::Queue, wgpu::Buffer>>,
        Eventually<VectorBufferPool>,
        Eventually<RasterResources>,
        MaskRenderPhase,
        VectorTilePhase,
        RasterTilePhase
    >().unwrap();

    let (
        Initialized(tile_view_pattern),
        Initialized(buffer_pool),
        Initialized(raster_resources),
        mask_phase,
        vector_tile_phase,
        raster_tile_phase
    ) = collection else { return; };

    mask_phase.clear();
    vector_tile_phase.clear();
    raster_tile_phase.clear();

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
