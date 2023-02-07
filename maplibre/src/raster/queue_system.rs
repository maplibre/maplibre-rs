//! Queues [PhaseItems](crate::render::render_phase::PhaseItem) for rendering.

use crate::{
    context::MapContext,
    ecs::world::Tile,
    raster::render_commands::DrawRasterTiles,
    render::{
        eventually::{Eventually, Eventually::Initialized},
        render_phase::{DrawState, LayerItem, RenderPhase},
        resource::RasterResources,
        tile_view_pattern::{HasTile, TileViewPattern},
    },
};

pub fn queue_system(
    MapContext {
        world, renderer, ..
    }: &mut MapContext,
) {
    // TODO duplicate
    let collection = world.resources.collect_mut3::<
        Eventually<TileViewPattern<wgpu::Queue, wgpu::Buffer>>,
        Eventually<RasterResources>,
        RenderPhase<LayerItem>,
    >().unwrap();

    let (
        Initialized(tile_view_pattern),
        Initialized(raster_resources),
        raster_tile_phase,
    ) = collection else { return; };

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
