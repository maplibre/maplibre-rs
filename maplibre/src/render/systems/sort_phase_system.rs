use crate::{
    context::MapContext,
    render::render_phase::{LayerItem, RenderPhase},
    tcs::system::SystemResult,
};

/// This system sorts all [`RenderPhases`](RenderPhase) for the [`PhaseItem`] type.
pub fn sort_phase_system(MapContext { world, .. }: &mut MapContext) -> SystemResult {
    // We are only sorting layers and not masks
    world
        .resources
        .get_mut::<RenderPhase<LayerItem>>()
        .unwrap()
        .sort();

    Ok(())
}
