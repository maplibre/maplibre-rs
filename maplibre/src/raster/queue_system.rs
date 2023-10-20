//! Queues [PhaseItems](crate::render::render_phase::PhaseItem) for rendering.

use crate::{
    context::MapContext,
    raster::render_commands::DrawRasterTiles,
    render::{
        eventually::{Eventually, Eventually::Initialized},
        render_commands::DrawMasks,
        render_phase::{DrawState, LayerItem, RenderPhase, TileMaskItem},
        tile_view_pattern::WgpuTileViewPattern,
    },
    tcs::tiles::Tile,
};

pub fn queue_system(MapContext { world, .. }: &mut MapContext) {
    let Some((Initialized(tile_view_pattern),)) = world
        .resources
        .query::<(&Eventually<WgpuTileViewPattern>,)>()
    else {
        return;
    };

    let mut items = Vec::new();

    for view_tile in tile_view_pattern.iter() {
        let coords = &view_tile.coords();
        tracing::trace!("Drawing tile at {coords}");

        // draw tile normal or the source e.g. parent or children
        view_tile.render(|source_shape| {
            // FIXME if raster_resources.has_tile(source_shape.coords(), world) {
            items.push((
                LayerItem {
                    draw_function: Box::new(DrawState::<LayerItem, DrawRasterTiles>::new()),
                    index: 0,
                    style_layer: "raster".to_string(),
                    tile: Tile {
                        coords: source_shape.coords(),
                    },
                    source_shape: source_shape.clone(),
                },
                // FIXME tsc: Tile masks are currently drawn twice by each plugin
                TileMaskItem {
                    draw_function: Box::new(DrawState::<TileMaskItem, DrawMasks>::new()),
                    source_shape: source_shape.clone(),
                },
            ));
        });
    }

    let Some((layer_item_phase, tile_mask_phase)) = world
        .resources
        .query_mut::<(&mut RenderPhase<LayerItem>, &mut RenderPhase<TileMaskItem>)>()
    else {
        return;
    };

    for (layer, mask) in items {
        layer_item_phase.add(layer);
        tile_mask_phase.add(mask);
    }
}
