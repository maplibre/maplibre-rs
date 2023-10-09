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
    let Some(Initialized(tile_view_pattern)) = world
        .resources
        .query_mut::<&mut Eventually<WgpuTileViewPattern>>()
    else {
        return;
    };

    let view_proj = view_state.view_projection();
    tile_view_pattern.upload_pattern(queue, &view_proj);
}
