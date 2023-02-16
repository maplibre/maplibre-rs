//! Extracts data from the current state.

use crate::{
    context::MapContext,
    render::{
        eventually::{Eventually, Eventually::Initialized},
        tile_view_pattern::{TilePhase, WgpuTileViewPattern},
    },
};

pub fn tile_view_pattern_system(
    MapContext {
        world, renderer, ..
    }: &mut MapContext,
) {
    let Some((
        Initialized(tile_view_pattern),
        tile_phase,
    )) = world.resources.query::<(
        &Eventually<WgpuTileViewPattern>,
        &TilePhase
    )>() else { return; };

    let view_state = &world.view_state;
    let view_region = view_state.create_view_region();

    if let Some(view_region) = &view_region {
        let zoom = view_state.zoom();

        let tile_views = tile_view_pattern.generate_pattern(view_region, tile_phase, zoom, world);

        let Some(Initialized(tile_view_pattern)) = world
            .resources
            .query_mut::<&mut Eventually<WgpuTileViewPattern>>() else { return; };
        tile_view_pattern.view_tiles = tile_views;
    }
}
