use std::rc::Rc;

use thiserror::Error;

use crate::{
    environment::OffscreenKernelConfig,
    headless::{
        environment::HeadlessEnvironment,
        graph_node::CopySurfaceBufferNode,
        system::WriteSurfaceBufferSystem,
        window::{HeadlessMapWindow, HeadlessMapWindowConfig},
    },
    io::apc::SchedulerAsyncProcedureCall,
    kernel::{Kernel, KernelBuilder},
    platform::{http_client::ReqwestHttpClient, scheduler::TokioScheduler},
    plugin::Plugin,
    render::{
        builder::RendererBuilder, graph::RenderGraph, tile_view_pattern::ViewTileSources,
        RenderStageLabel, Renderer,
    },
    schedule::Schedule,
    tcs::{system::SystemContainer, world::World},
    window::{MapWindowConfig, PhysicalSize},
};

/// Failure while creating an offscreen renderer.
#[derive(Debug, Error)]
pub enum HeadlessRendererError {
    /// Requested dimensions cannot form a physical window size.
    #[error("invalid headless renderer size {width}x{height}")]
    InvalidSize {
        /// Requested pixel width.
        width: u32,
        /// Requested pixel height.
        height: u32,
    },
    /// Offscreen window creation failed.
    #[error("headless window creation failed")]
    Window {
        /// Underlying window error.
        #[source]
        source: crate::window::WindowCreateError,
    },
    /// Renderer initialization failed.
    #[error("headless renderer initialization failed")]
    Renderer {
        /// Underlying renderer error.
        #[source]
        source: crate::render::error::RenderError,
    },
}

mod graph_node;
mod system;

pub mod environment;
pub mod map;
pub mod window;

pub async fn create_headless_renderer(
    width: u32,
    height: u32,
    cache_path: Option<String>,
) -> Result<(Kernel<HeadlessEnvironment>, Renderer), HeadlessRendererError> {
    let size = PhysicalSize::new(width, height)
        .ok_or(HeadlessRendererError::InvalidSize { width, height })?;
    let client = ReqwestHttpClient::new(cache_path);
    let kernel = KernelBuilder::new()
        .with_map_window_config(HeadlessMapWindowConfig::new(size))
        .with_http_client(client.clone())
        .with_apc(SchedulerAsyncProcedureCall::new(
            TokioScheduler::new(),
            OffscreenKernelConfig {
                cache_directory: None,
            },
        ))
        .with_scheduler(TokioScheduler::new())
        .build();

    let mwc: &HeadlessMapWindowConfig = kernel.map_window_config();
    let window: HeadlessMapWindow = mwc
        .create()
        .map_err(|source| HeadlessRendererError::Window { source })?;

    let renderer = RendererBuilder::new()
        .build()
        .initialize_headless::<HeadlessMapWindowConfig>(&window)
        .await
        .map_err(|source| HeadlessRendererError::Renderer { source })?;

    Ok((kernel, renderer))
}

/// Labels for the "draw" graph
mod draw_graph {
    pub const NAME: &str = "draw";
    // Labels for input nodes
    pub mod input {}
    // Labels for non-input nodes
    pub mod node {
        pub const TRANSLUCENT_PASS: &str = "translucent_pass";
        pub const COPY: &str = "copy_pass";
    }
}

fn attach_surface_copy_node(
    draw_graph: &mut RenderGraph,
) -> Result<(), crate::render::graph::RenderGraphError> {
    draw_graph.add_node(draw_graph::node::COPY, CopySurfaceBufferNode);
    draw_graph.add_node_edge(draw_graph::node::TRANSLUCENT_PASS, draw_graph::node::COPY)
}

pub struct HeadlessPlugin {
    write_to_disk: bool,
    preserve_tile_sources: bool,
}

impl HeadlessPlugin {
    pub fn new(write_to_disk: bool) -> Self {
        Self {
            write_to_disk,
            preserve_tile_sources: false,
        }
    }

    /// Keeps source availability checks active for parent/child tile fallback.
    pub fn preserve_tile_sources(mut self) -> Self {
        self.preserve_tile_sources = true;
        self
    }
}

impl Plugin<HeadlessEnvironment> for HeadlessPlugin {
    fn build(
        &self,
        schedule: &mut Schedule,
        _kernel: Rc<Kernel<HeadlessEnvironment>>,
        world: &mut World,
        graph: &mut RenderGraph,
    ) {
        let resources = &mut world.resources;

        let Some(draw_graph) = graph.get_sub_graph_mut(draw_graph::NAME) else {
            tracing::error!("headless draw subgraph is unavailable");
            return;
        };
        if let Err(error) = attach_surface_copy_node(draw_graph) {
            tracing::error!(?error, "cannot attach headless surface copy node");
            return;
        }

        schedule.add_system_to_stage(
            RenderStageLabel::Cleanup,
            SystemContainer::new(WriteSurfaceBufferSystem::new(self.write_to_disk)),
        );

        // FIXME tcs: Is this good style?
        schedule.remove_stage(RenderStageLabel::Extract);
        if !self.preserve_tile_sources {
            resources.get_mut::<ViewTileSources>().unwrap().clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{attach_surface_copy_node, draw_graph};
    use crate::render::graph::{EmptyNode, RenderGraph};

    #[test]
    #[allow(clippy::expect_used)]
    fn surface_copy_waits_for_translucent_rendering() {
        let mut graph = RenderGraph::default();
        let translucent = graph.add_node(draw_graph::node::TRANSLUCENT_PASS, EmptyNode);

        attach_surface_copy_node(&mut graph).expect("copy node should attach");

        let predecessors = graph
            .iter_node_inputs(draw_graph::node::COPY)
            .expect("copy node should exist")
            .map(|(_, node)| node.id)
            .collect::<Vec<_>>();
        assert_eq!(predecessors, vec![translucent]);
    }
}
