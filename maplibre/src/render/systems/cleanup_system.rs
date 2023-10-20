use crate::{
    context::MapContext,
    render::render_phase::{LayerItem, RenderPhase, TileMaskItem},
};

pub fn cleanup_system(MapContext { world, .. }: &mut MapContext) {
    let Some((layer_item_phase, tile_mask_phase)) = world
        .resources
        .query_mut::<(&mut RenderPhase<LayerItem>, &mut RenderPhase<TileMaskItem>)>()
    else {
        return;
    };

    layer_item_phase.clear();
    tile_mask_phase.clear();
}
