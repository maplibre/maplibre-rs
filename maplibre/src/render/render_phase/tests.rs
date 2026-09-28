use super::{DrawState, LayerItem, PhaseItem, RenderPhase, TileMaskItem};
use crate::{
    background::render_commands::{DrawBackgroundQuad, SetBackgroundPipeline},
    render::{render_commands::DrawMasks, tile_view_pattern::TileShape},
    tcs::tiles::Tile,
};

fn mask(generate_borders: bool) -> TileMaskItem {
    TileMaskItem {
        draw_function: Box::new(DrawState::<TileMaskItem, DrawMasks>::new()),
        source_shape: Default::default(),
        generate_borders,
    }
}

fn raster(index: u32, generate_borders: bool) -> LayerItem {
    LayerItem {
        draw_function: Box::new(DrawState::<
            LayerItem,
            (SetBackgroundPipeline, DrawBackgroundQuad),
        >::new()),
        index,
        is_line: false,
        generate_borders,
        style_layer: "raster".to_string(),
        tile: Tile {
            coords: Default::default(),
        },
        source_shape: TileShape::default(),
    }
}

#[test]
fn bordered_stencil_pass_sorts_before_borderless_pass() {
    let mut phase = RenderPhase::default();
    phase.add(mask(false));
    phase.add(mask(true));

    phase.sort();

    assert!(phase.items[0].generate_borders);
    assert!(!phase.items[1].generate_borders);
}

#[test]
fn raster_borders_sort_before_interiors_without_crossing_style_layers() {
    let mut phase = RenderPhase::default();
    phase.add(raster(2, false));
    phase.add(raster(1, false));
    phase.add(raster(2, true));
    phase.add(raster(1, true));

    phase.sort();

    let keys = phase
        .items
        .iter()
        .map(PhaseItem::sort_key)
        .collect::<Vec<_>>();
    assert_eq!(keys, vec![(1, false), (1, true), (2, false), (2, true)]);
}
