//! Queues [PhaseItems](crate::render::render_phase::PhaseItem) for rendering.

use std::ops::Deref;

use crate::{
    context::MapContext,
    ecs::tiles::Tile,
    raster::render_commands::DrawRasterTiles,
    render::{
        eventually::{Eventually, Eventually::Initialized},
        render_phase::{DrawState, LayerItem, RenderPhase},
        resource::RasterResources,
        tile_view_pattern::HasTile,
    },
    vector::WgpuTileViewPattern,
};

pub fn queue_system(
    MapContext {
        world, renderer, ..
    }: &mut MapContext,
) {
    let (
        Initialized(tile_view_pattern),
        Initialized(ref raster_resources),
        mut raster_tile_phase,
    ) = world.resources.query_mut::<(&mut Eventually<WgpuTileViewPattern>, &mut Eventually<RasterResources>, &mut RenderPhase<LayerItem>)>().unwrap() else { return; }; // FIXME tcs: Unwrap

    for view_tile in tile_view_pattern.iter() {
        let coords = &view_tile.coords();
        tracing::trace!("Drawing tile at {coords}");

        // draw tile normal or the source e.g. parent or children
        view_tile.render(|source_shape| {
            if raster_resources.has_tile(&source_shape.coords()) {
                raster_tile_phase.add(LayerItem {
                    draw_function: Box::new(DrawState::<LayerItem, DrawRasterTiles>::new()),
                    index: 0,
                    style_layer: "raster".to_string(),
                    tile: Tile {
                        coords: source_shape.coords(),
                    },
                    source_shape: source_shape.clone(),
                })
            }
        });
    }
}
