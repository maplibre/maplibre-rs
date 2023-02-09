//! Queues [PhaseItems](crate::render::render_phase::PhaseItem) for rendering.

use crate::{
    context::MapContext,
    ecs::tiles::Tile,
    raster::{render_commands::DrawRasterTiles, resource::RasterResources},
    render::{
        eventually::{Eventually, Eventually::Initialized},
        render_phase::{DrawState, LayerItem, RenderPhase},
        tile_view_pattern::HasTile,
    },
    vector::WgpuTileViewPattern,
};

pub fn queue_system(
    MapContext {
        world, renderer, ..
    }: &mut MapContext,
) {
    let Some((
        Initialized(tile_view_pattern),
        Initialized(raster_resources),
        layer_item_phase,
    )) = world.resources.query_mut::<(
        &mut Eventually<WgpuTileViewPattern>,
        &mut Eventually<RasterResources>,
        &mut RenderPhase<LayerItem>,
    )>() else { return; };

    for view_tile in tile_view_pattern.iter() {
        let coords = &view_tile.coords();
        tracing::trace!("Drawing tile at {coords}");

        // draw tile normal or the source e.g. parent or children
        view_tile.render(|source_shape| {
            if raster_resources.has_tile(&source_shape.coords()) {
                layer_item_phase.add(LayerItem {
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
