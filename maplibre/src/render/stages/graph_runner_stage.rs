//! Executes the [`RenderGraph`] current render graph.

use log::error;

use crate::{
    context::MapContext,
    render::{
        eventually::Eventually::Initialized, graph::RenderGraph, graph_runner::RenderGraphRunner,
    },
    schedule::Stage,
    Renderer,
};

/// Updates the [`RenderGraph`] with all of its nodes and then runs it to render the entire frame.
pub struct GraphRunnerStage {
    graph: RenderGraph,
}

impl GraphRunnerStage {
    pub fn new(graph: RenderGraph) -> Self {
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
