//! Extracts data from the current state.

use crate::{
    context::MapContext,
    render::{
        eventually::{Eventually, Eventually::Initialized},
        tile_view_pattern::{ViewTileSources, WgpuTileViewPattern, DEFAULT_TILE_SIZE},
        view_state::ViewStatePadding,
    },
    tcs::system::{SystemError, SystemResult},
};

pub fn tile_view_pattern_system(
    MapContext {
        view_state, world, ..
    }: &mut MapContext,
) -> SystemResult {
    let Some((Initialized(tile_view_pattern), view_tile_sources)) = world
        .resources
        .query::<(&Eventually<WgpuTileViewPattern>, &ViewTileSources)>()
    else {
        return Err(SystemError::Dependencies);
    };

    // Create the tile view pattern only for tiles in view -> Tight
    let view_region = view_state.create_view_region(
        view_state.zoom().zoom_level(DEFAULT_TILE_SIZE),
        ViewStatePadding::Tight,
    );

    if let Some(view_region) = &view_region {
        let zoom = view_state.zoom();

        let view_tiles =
            tile_view_pattern.generate_pattern(view_region, view_tile_sources, zoom, world);

        // TODO: Can we &mut borrow initially somehow instead of here?
        let Some(Initialized(tile_view_pattern)) = world
            .resources
            .query_mut::<&mut Eventually<WgpuTileViewPattern>>()
        else {
            return Err(SystemError::Dependencies);
        };

        log::trace!("Tiles in view: {}", view_tiles.len());

        tile_view_pattern.update_pattern(view_tiles);
    }

    Ok(())
}
