use std::{ops::Deref, rc::Rc};

use crate::{
    debug::{
        cleanup_system::cleanup_system, debug_pass::DebugPassNode, queue_system::queue_system,
        resource_system::resource_system,
    },
    environment::Environment,
    kernel::Kernel,
    plugin::Plugin,
    render::{
        eventually::Eventually,
        graph::RenderGraph,
        render_phase::{Draw, PhaseItem, RenderPhase},
        tile_view_pattern::TileShape,
        RenderStageLabel,
    },
    schedule::Schedule,
    tcs::world::World,
};

mod cleanup_system;
mod debug_pass;
mod queue_system;
mod render_commands;
mod resource_system;

/// Labels for the "draw" graph
mod draw_graph {
    pub const NAME: &str = "draw";
    // Labels for input nodes
    pub mod input {}
    // Labels for non-input nodes
    pub mod node {
        pub const MAIN_PASS: &str = "main_pass";
        pub const DEBUG_PASS: &str = "debug_pass";
    }
}

struct DebugPipeline(wgpu::RenderPipeline);
impl Deref for DebugPipeline {
    type Target = wgpu::RenderPipeline;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

struct TileDebugItem {
    pub draw_function: Box<dyn Draw<TileDebugItem>>,
    pub source_shape: TileShape,
}

impl PhaseItem for TileDebugItem {
    type SortKey = u32;

    fn sort_key(&self) -> Self::SortKey {
        0
    }

    fn draw_function(&self) -> &dyn Draw<TileDebugItem> {
        self.draw_function.as_ref()
    }
}

#[derive(Default)]
pub struct DebugPlugin;

impl<E: Environment> Plugin<E> for DebugPlugin {
    fn build(
        &self,
        schedule: &mut Schedule,
        _kernel: Rc<Kernel<E>>,
        world: &mut World,
        graph: &mut RenderGraph,
    ) {
        let resources = &mut world.resources;

        let draw_graph = graph.get_sub_graph_mut(draw_graph::NAME).unwrap();
        draw_graph.add_node(draw_graph::node::DEBUG_PASS, DebugPassNode::new());

        draw_graph
            .add_node_edge(draw_graph::node::MAIN_PASS, draw_graph::node::DEBUG_PASS)
            .unwrap();

        resources.init::<RenderPhase<TileDebugItem>>();
        resources.insert(Eventually::<DebugPipeline>::Uninitialized);

        schedule.add_system_to_stage(RenderStageLabel::Prepare, resource_system);
        schedule.add_system_to_stage(RenderStageLabel::Queue, queue_system);
        schedule.add_system_to_stage(RenderStageLabel::Cleanup, cleanup_system);
    }
}
