use crate::{
    context::MapContext,
    render::render_phase::{LayerItem, PhaseItem, RenderPhase, TileMaskItem},
};

pub fn cleanup_system(
    MapContext {
        world, renderer, ..
    }: &mut MapContext,
) {
    let layer_items = world.get_resource_mut::<RenderPhase<LayerItem>>();
    layer_items.clear();
    let mask_items = world.get_resource_mut::<RenderPhase<TileMaskItem>>();
    mask_items.clear();
}
