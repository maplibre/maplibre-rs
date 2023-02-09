use crate::{
    context::MapContext,
    render::render_phase::{LayerItem, RenderPhase, TileMaskItem},
};

pub fn cleanup_system(
    MapContext {
        world, renderer, ..
    }: &mut MapContext,
) {
    let layer_items = world.resources.get_mut::<RenderPhase<LayerItem>>().unwrap();
    layer_items.clear();
    let mask_items = world
        .resources
        .get_mut::<RenderPhase<TileMaskItem>>()
        .unwrap();
    mask_items.clear();
}
