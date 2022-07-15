//! Sorts items of the [RenderPhases](RenderPhase).

use crate::context::MapContext;

use crate::render::render_phase::RenderPhase;

use crate::schedule::Stage;
use crate::Renderer;

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
        let file_phase = &mut state.tile_phase;
        file_phase.sort();
    }
}
