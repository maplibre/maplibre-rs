use crate::{context::MapContext, debug::TileDebugItem, render::render_phase::RenderPhase};

pub fn cleanup_system(MapContext { world, .. }: &mut MapContext) {
    let Some(debug_tile_phase) = world
        .resources
        .query_mut::<&mut RenderPhase<TileDebugItem>>()
    else {
        return;
    };

    debug_tile_phase.clear();
}
