//! Sorts items of the [RenderPhases](RenderPhase).

use crate::{
    context::MapContext,
    render::{render_phase::RenderPhase, Renderer},
    schedule::Stage,
};

#[derive(Default)]
pub struct PhaseSortStage;

impl Stage for PhaseSortStage {
    fn run(
        &mut self,
        MapContext {
            renderer: Renderer { state, .. },
            ..
        }: &mut MapContext,
    ) {
        let mask_phase: &mut RenderPhase<_> = &mut state.mask_phase;
        mask_phase.sort();
        let file_phase = &mut state.vector_tile_phase;
        file_phase.sort();
        let raster_phase = &mut state.raster_tile_phase;
        raster_phase.sort();
    }
}
