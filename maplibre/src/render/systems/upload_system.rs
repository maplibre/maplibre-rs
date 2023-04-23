//! Uploads data to the GPU which is needed for rendering.
use crate::{
    context::MapContext,
    render::{
        eventually::{Eventually, Eventually::Initialized},
        tile_view_pattern::WgpuTileViewPattern,
        Renderer,
    },
};

pub fn upload_system(
    MapContext {
        world,
        view_state,
        renderer: Renderer { queue, .. },
        ..
    }: &mut MapContext,
) {
    let Some(view_proj) = view_state.view_projection() else {
        // skip every thing if there is no view.
        return;
    };

    let Some(
        Initialized(tile_view_pattern)
    ) = world.resources.query_mut::<
        &mut Eventually<WgpuTileViewPattern>
    >() else { return; };

    tile_view_pattern.upload_pattern(queue, &view_proj);
}
