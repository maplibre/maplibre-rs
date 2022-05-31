//! Rendering specific [Stages](Stage)

use crate::context::MapContext;
use crate::multi_stage;
use crate::render::graph::{EmptyNode, RenderGraph, RenderGraphError};
use crate::render::main_pass::{MainPassDriverNode, MainPassNode};
use crate::render::stages::extract_stage::ExtractStage;
use crate::render::stages::phase_sort_stage::PhaseSortStage;
use crate::render::stages::queue_stage::QueueStage;
use crate::render::{draw_graph, main_graph};
use crate::schedule::{MultiStage, Schedule, Stage, StageLabel};
use graph_runner_stage::GraphRunnerStage;
use resource_stage::ResourceStage;
use upload_stage::UploadStage;

mod extract_stage;
mod graph_runner_stage;
mod phase_sort_stage;
mod queue_stage;
mod resource_stage;
mod upload_stage;
#[cfg(not(target_arch = "wasm32"))]
mod write_surface_buffer_stage;

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

pub fn register_render_stages(
    schedule: &mut Schedule,
    headless: bool,
) -> Result<(), RenderGraphError> {
    let mut graph = RenderGraph::default();

    let mut draw_graph = RenderGraph::default();
    draw_graph.add_node(draw_graph::node::MAIN_PASS, MainPassNode::new());
    let input_node_id = draw_graph.set_input(vec![]);
    draw_graph.add_node_edge(input_node_id, draw_graph::node::MAIN_PASS)?;

    #[cfg(not(target_arch = "wasm32"))]
    if headless {
        use crate::render::copy_surface_to_buffer_node::CopySurfaceBufferNode;
        draw_graph.add_node(draw_graph::node::COPY, CopySurfaceBufferNode::default());
        draw_graph.add_node_edge(draw_graph::node::MAIN_PASS, draw_graph::node::COPY)?;
    }

    graph.add_sub_graph(draw_graph::NAME, draw_graph);
    graph.add_node(main_graph::node::MAIN_PASS_DEPENDENCIES, EmptyNode);
    graph.add_node(main_graph::node::MAIN_PASS_DRIVER, MainPassDriverNode);
    graph.add_node_edge(
        main_graph::node::MAIN_PASS_DEPENDENCIES,
        main_graph::node::MAIN_PASS_DRIVER,
    )?;

    schedule.add_stage(RenderStageLabel::Prepare, PrepareStage::default());
    schedule.add_stage(RenderStageLabel::Queue, QueueStage::default());
    schedule.add_stage(RenderStageLabel::PhaseSort, PhaseSortStage::default());
    schedule.add_stage(RenderStageLabel::Render, GraphRunnerStage::new(graph));

    #[cfg(not(target_arch = "wasm32"))]
    if headless {
        use crate::render::stages::write_surface_buffer_stage::WriteSurfaceBufferStage;
        schedule.add_stage(
            RenderStageLabel::Cleanup,
            WriteSurfaceBufferStage::default(),
        );
    }

    Ok(())
}
