use crate::{
    context::MapContext,
    debug::TileDebugItem,
    render::render_phase::RenderPhase,
    tcs::system::{SystemError, SystemResult},
};

pub fn cleanup_system(MapContext { world, .. }: &mut MapContext) -> SystemResult {
    let Some(debug_tile_phase) = world
        .resources
        .query_mut::<&mut RenderPhase<TileDebugItem>>()
    else {
        return Err(SystemError::Dependencies);
    };

    debug_tile_phase.clear();

    Ok(())
}
