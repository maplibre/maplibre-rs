//! Rendering specific [Stages](Stage)

use graph_runner_stage::GraphRunnerStage;
use resource_stage::ResourceStage;
use upload_stage::UploadStage;

use crate::{
    context::MapContext,
    multi_stage,
    render::{
        graph::RenderGraph,
        stages::{
            extract_stage::ExtractStage, phase_sort_stage::PhaseSortStage, queue_stage::QueueStage,
        },
    },
    schedule::{Schedule, Stage, StageLabel},
};

mod extract_stage;
mod graph_runner_stage;
mod phase_sort_stage;
mod queue_stage;
mod resource_stage;
mod upload_stage;

/// The labels of the default App rendering stages.
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum RenderStageLabel {
    /// Prepare render resources from the extracted data for the GPU.
    /// For example during this phase textures are created, buffers are allocated and written.
    Prepare,

    /// Queues [PhaseItems](crate::render::render_phase::draw::PhaseItem) that depend on
    /// [`Prepare`](RenderStageLabel::Prepare) data and queue up draw calls to run during the
    /// [`Render`](RenderStageLabel::Render) stage.
    Queue,

    /// Sort the [`RenderPhases`](crate::render_phase::RenderPhase) here.
    PhaseSort,

    /// Actual rendering happens here.
    /// In most cases, only the render backend should insert resources here.
    Render,

    /// Cleanup render resources here.
    Cleanup,
}

impl StageLabel for RenderStageLabel {
    fn dyn_clone(&self) -> Box<dyn StageLabel> {
        Box::new(self.clone())
    }
}

multi_stage!(
    PrepareStage,
    resource: ResourceStage,
    extract: ExtractStage,
    upload: UploadStage
);

pub fn register_default_render_stages(graph: RenderGraph, schedule: &mut Schedule) {
    schedule.add_stage(RenderStageLabel::Prepare, PrepareStage::default());
    schedule.add_stage(RenderStageLabel::Queue, QueueStage::default());
    schedule.add_stage(RenderStageLabel::PhaseSort, PhaseSortStage::default());
    schedule.add_stage(RenderStageLabel::Render, GraphRunnerStage::new(graph));
}
