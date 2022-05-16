use crate::schedule::{Schedule, Stage, StageLabel};
use graph_runner_stage::GraphRunnerStage;
use resource_stage::ResourceStage;
use upload_stage::UploadStage;

mod graph_runner_stage;
mod resource_stage;
mod upload_stage;

use crate::context::MapContext;
use crate::{Renderer, ScheduleMethod};
pub use graph_runner_stage::{draw_graph, node};

/// The labels of the default App rendering stages.
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum RenderStageLabel {
    /// FIXME Extract data from the "app world" and insert it into the "render world".
    /// This step should be kept as short as possible to increase the "pipelining potential" for
    /// running the next frame while rendering the current frame.
    /// FIXME: Unclear what to do here
    Extract,

    /// Prepare render resources from the extracted data for the GPU.
    /// FIXME: Create textures, buffers, write_to buffers
    Prepare,

    /// Create [`BindGroups`](crate::render_resource::BindGroup) that depend on
    /// [`Prepare`](RenderStageLabel::Prepare) data and queue up draw calls to run during the
    /// [`Render`](RenderStageLabel::Render) stage.
    /// FIXME: Add items to render phase
    Queue,

    // FIXME: TODO: This could probably be moved in favor of a system ordering abstraction in Render or Queue
    /// Sort the [`RenderPhases`](crate::render_phase::RenderPhase) here.
    PhaseSort,

    /// Actual rendering happens here.
    /// In most cases, only the render backend should insert resources here.
    Render,

    /// Cleanup render resources here.
    /// FIXME: Cleanup cached data
    Cleanup,
}

impl StageLabel for RenderStageLabel {
    fn dyn_clone(&self) -> Box<dyn StageLabel> {
        Box::new(self.clone())
    }
}

#[derive(Default)]
struct PrepareStage {
    resource_stage: ResourceStage,
    upload_stage: UploadStage,
}

impl Stage for PrepareStage {
    fn run(&mut self, context: &mut MapContext) {
        self.resource_stage.run(context);
        self.upload_stage.run(context);
    }
}

pub fn register_render_stages(schedule: &mut Schedule) {
    schedule.add_stage(RenderStageLabel::Prepare, PrepareStage::default());
    schedule.add_stage(RenderStageLabel::Render, GraphRunnerStage::default());
}
