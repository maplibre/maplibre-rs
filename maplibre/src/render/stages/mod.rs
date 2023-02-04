//! Rendering specific [Stages](Stage)

use graph_runner_stage::GraphRunnerSystem;

use crate::{
    ecs::system::stage::SystemStage,
    multi_stage,
    render::stages::resource_stage::ResourceSystem,
    schedule::{Schedule, Stage, StageLabel},
};

mod graph_runner_stage;
mod resource_stage;

/// The labels of the default App rendering stages.
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum RenderStageLabel {
    /// Extract data from the world.
    Extract,

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

pub fn register_default_render_stages(schedule: &mut Schedule) {
    schedule.add_stage(RenderStageLabel::Extract, SystemStage::default());
    schedule.add_stage(
        RenderStageLabel::Prepare,
        SystemStage::default().with_system_direct(ResourceSystem),
    );
    schedule.add_stage(RenderStageLabel::Queue, SystemStage::default());
    schedule.add_stage(RenderStageLabel::PhaseSort, SystemStage::default());
    schedule.add_stage(
        RenderStageLabel::Render,
        SystemStage::default().with_system_direct(GraphRunnerSystem),
    );
    schedule.add_stage(RenderStageLabel::Cleanup, SystemStage::default());
}
