//! Sorts items of the [RenderPhases](RenderPhase).

use crate::{
    context::MapContext,
    ecs::world::World,
    render::render_phase::RenderPhase,
    vector::{MaskRenderPhase, RasterTilePhase, VectorTilePhase},
};

pub fn render_phase_sort_system(
    MapContext {
        world, renderer, ..
    }: &mut MapContext,
) {
    world.get_resource_mut::<MaskRenderPhase>().sort();
    world.get_resource_mut::<VectorTilePhase>().sort();
    world.get_resource_mut::<RasterTilePhase>().sort();
}
