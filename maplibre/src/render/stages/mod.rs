//! Rendering specific [Stages](Stage)

use graph_runner_system::GraphRunnerSystem;

use crate::{
    ecs::system::{stage::SystemStage, SystemContainer},
    render::stages::{
        cleanup_system::cleanup_system, resource_system::ResourceSystem,
        sort_phase_system::sort_phase_system,
    },
    schedule::{Schedule, StageLabel},
};

mod cleanup_system;
mod graph_runner_system;
mod resource_system;
mod sort_phase_system;

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
        SystemStage::default().with_system(SystemContainer::new(ResourceSystem)),
    );
    schedule.add_stage(RenderStageLabel::Queue, SystemStage::default());
    schedule.add_stage(
        RenderStageLabel::PhaseSort,
        SystemStage::default().with_system(sort_phase_system),
    );

    schedule.add_stage(
        RenderStageLabel::Render,
        SystemStage::default().with_system(SystemContainer::new(GraphRunnerSystem)),
    );
    schedule.add_stage(
        RenderStageLabel::Cleanup,
        SystemStage::default().with_system(cleanup_system),
    );
}
