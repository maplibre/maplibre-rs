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
        let tile_phase = &mut state.tile_phase;
        tile_phase.sort();
        let symbol_tile_phase = &mut state.symbol_tile_phase;
        symbol_tile_phase.sort();
    }
}
