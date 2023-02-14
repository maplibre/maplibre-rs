//! Extracts data from the current state.

use std::ops::Deref;

use crate::{
    context::MapContext,
    coords::WorldTileCoords,
    ecs::tiles::Tiles,
    raster::RasterResources,
    render::{
        eventually::{Eventually, Eventually::Initialized},
        tile_view_pattern::HasTile,
    },
    vector::{VectorBufferPool, VectorLayersDataComponent, WgpuTileViewPattern},
};

// FIXME: Is this the correct way to do this? Ideally we want to wait until all layers are uploaded to the gpu?
struct VectorTilesDone<'t> {
    tiles: &'t Tiles,
}

impl<'t> HasTile for VectorTilesDone<'t> {
    fn has_tile(&self, coords: &WorldTileCoords) -> bool {
        let Some(vector_layers_indices) = self
            .tiles
            .query::<&VectorLayersDataComponent>(*coords) else { return false; };

        vector_layers_indices.done
    }
}

pub fn tile_view_pattern_system(
    MapContext {
        world, renderer, ..
    }: &mut MapContext,
) {
    let Some((
        Initialized(tile_view_pattern),
        Initialized(buffer_pool),
        Initialized(raster_resources)
    )) = world.resources.query_mut::<(
        &mut Eventually<WgpuTileViewPattern>,
         &mut Eventually<VectorBufferPool>,
         &mut Eventually<RasterResources> // FIXME tcs: Make this independent of raster
    )>() else { return; };

    let view_state = &world.view_state;
    let view_region = view_state.create_view_region();

    if let Some(view_region) = &view_region {
        let zoom = view_state.zoom();
        //tile_view_pattern.update_pattern(view_region, buffer_pool.index(), zoom);
        tile_view_pattern.update_pattern(
            view_region,
            &(
                raster_resources.deref(), // FIXME tcs: Remove for headless?
                buffer_pool.index(),      // FIXME tcs: Remove for headless?
                &VectorTilesDone {
                    tiles: &world.tiles,
                },
            ),
            zoom,
        );
    }
}
