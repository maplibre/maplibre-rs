use crate::{
    context::MapContext,
    render::render_phase::{LayerItem, PhaseItem, RenderPhase, TileMaskItem},
};

/// This system sorts all [`RenderPhases`](RenderPhase) for the [`PhaseItem`] type.
pub fn sort_phase_system(
    MapContext {
        world, renderer, ..
    }: &mut MapContext,
) {
    world.get_resource_mut::<RenderPhase<LayerItem>>().sort();
    world.get_resource_mut::<RenderPhase<TileMaskItem>>().sort();
}
