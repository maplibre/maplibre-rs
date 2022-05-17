//! Executes the [`RenderGraph`] current render graph.

// Plugins that contribute to the RenderGraph should use the following label conventions:
// 1. Graph modules should have a NAME, input module, and node module (where relevant)
// 2. The "top level" graph is the plugin module root. Just add things like `pub mod node` directly under the plugin module
// 3. "sub graph" modules should be nested beneath their parent graph module

use crate::context::MapContext;
use crate::render::graph::{EmptyNode, RenderGraph};
use crate::render::graph_runner::RenderGraphRunner;
use crate::render::main_pass::{MainPassDriverNode, MainPassNode};
use crate::render::util::Eventually::Initialized;
use crate::schedule::Stage;
use crate::Renderer;
use log::error;

pub mod node {
    pub const MAIN_PASS_DEPENDENCIES: &str = "main_pass_dependencies";
    pub const MAIN_PASS_DRIVER: &str = "main_pass_driver";
}

pub mod draw_graph {
    pub const NAME: &str = "draw";
    pub mod input {}
    pub mod node {
        pub const MAIN_PASS: &str = "main_pass";
    }
}

/// Updates the [`RenderGraph`] with all of its nodes and then runs it to render the entire frame.
pub struct GraphRunnerStage {
    graph: RenderGraph,
}

impl Default for GraphRunnerStage {
    fn default() -> Self {
        let pass_node = MainPassNode::new();
        let mut graph = RenderGraph::default();

        let mut draw_graph = RenderGraph::default();
        draw_graph.add_node(draw_graph::node::MAIN_PASS, pass_node);
        let input_node_id = draw_graph.set_input(vec![]);
        draw_graph
            .add_node_edge(input_node_id, draw_graph::node::MAIN_PASS)
            .unwrap();
        graph.add_sub_graph(draw_graph::NAME, draw_graph);

        graph.add_node(node::MAIN_PASS_DEPENDENCIES, EmptyNode);
        graph.add_node(node::MAIN_PASS_DRIVER, MainPassDriverNode);
        graph
            .add_node_edge(node::MAIN_PASS_DEPENDENCIES, node::MAIN_PASS_DRIVER)
            .unwrap();
        Self { graph }
    }
}

impl Stage for GraphRunnerStage {
    fn run(
        &mut self,
        MapContext {
            renderer:
                Renderer {
                    device,
                    queue,
                    state,
                    ..
                },
            ..
        }: &mut MapContext,
    ) {
        self.graph.update(state);

        if let Err(e) = RenderGraphRunner::run(&self.graph, device, queue, state) {
            error!("Error running render graph:");
            {
                let mut src: &dyn std::error::Error = &e;
                loop {
                    error!("> {}", src);
                    match src.source() {
                        Some(s) => src = s,
                        None => break,
                    }
                }
            }

            panic!("Error running render graph: {:?}", e);
        }

        {
            let _span = tracing::info_span!("present_frames").entered();

            if let Initialized(render_target) = state.render_target.take() {
                if let Some(surface_texture) = render_target.take_surface_texture() {
                    surface_texture.present();
                }

                #[cfg(feature = "tracing-tracy")]
                tracing::event!(
                    tracing::Level::INFO,
                    message = "finished frame",
                    tracy.frame_mark = true
                );
            }
        }
    }
}
