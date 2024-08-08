//! Executes the [`RenderGraph`] current render graph.

use std::{borrow::Cow, error::Error};

use log::error;

use crate::{
    context::MapContext,
    render::{eventually::Eventually::Initialized, graph_runner::RenderGraphRunner, Renderer},
    tcs::system::{System, SystemError, SystemResult},
};

/// Updates the [`RenderGraph`] with all of its nodes and then runs it to render the entire frame.
#[derive(Default)]
pub struct GraphRunnerSystem;

impl System for GraphRunnerSystem {
    fn name(&self) -> Cow<'static, str> {
        "graph_runner".into()
    }

    fn run(
        &mut self,
        MapContext {
            world,
            renderer:
                Renderer {
                    device,
                    queue,
                    resources: state,
                    render_graph,
                    ..
                },
            ..
        }: &mut MapContext,
    ) -> SystemResult {
        render_graph.update(state);

        if let Err(e) = RenderGraphRunner::run(render_graph, device, queue, state, world) {
            error!("Error running render graph:");
            {
                let mut src: &dyn Error = &e;
                loop {
                    error!("> {src}");
                    match src.source() {
                        Some(s) => src = s,
                        None => break,
                    }
                }
            }

            // TODO: Replace panic with a graceful exit in the event loop
            // if e.should_exit() { *control_flow = ControlFlow::Exit; }
            panic!("Error running render graph: {e:?}");
        }

        {
            let _span = tracing::info_span!("present_frames").entered();

            let Initialized(render_target) = state.render_target.take() else {
                return Err(SystemError::Dependencies);
            };

            if let Some(surface_texture) = render_target.take_surface_texture() {
                surface_texture.present();
            }

            #[cfg(feature = "tracing-tracy")]
            tracing::event!(
                tracing::Level::INFO,
                message = "finished frame",
                tracy.frame_mark = true
            );

            Ok(())
        }
    }
}
