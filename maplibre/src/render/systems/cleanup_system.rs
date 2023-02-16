use crate::{
    context::MapContext,
    render::render_phase::{LayerItem, RenderPhase, TileDebugItem, TileMaskItem},
};

pub fn cleanup_system(
    MapContext {
        world, renderer, ..
    }: &mut MapContext,
) {
    let Some((layer_item_phase, tile_mask_phase, debug_tile_phase)) = world
        .resources
        .query_mut::<(
            &mut RenderPhase<LayerItem>,
            &mut RenderPhase<TileMaskItem>,
            &mut RenderPhase<TileDebugItem>,
        )>() else { return; };

    layer_item_phase.clear();
    tile_mask_phase.clear();
    debug_tile_phase.clear();
}
