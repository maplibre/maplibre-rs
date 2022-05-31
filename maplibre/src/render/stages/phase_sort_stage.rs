//! Sorts items of the [RenderPhases](RenderPhase).

use crate::context::MapContext;
use crate::coords::{ViewRegion, Zoom};
use crate::io::tile_repository::TileRepository;
use crate::render::camera::ViewProjection;
use crate::render::render_phase::RenderPhase;
use crate::render::resource::IndexEntry;
use crate::render::shaders::{
    ShaderCamera, ShaderFeatureStyle, ShaderGlobals, ShaderLayerMetadata, Vec4f32,
};
use crate::render::tile_view_pattern::TileInView;
use crate::render::util::Eventually::Initialized;
use crate::schedule::Stage;
use crate::{RenderState, Renderer, Style};
use std::iter;

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
