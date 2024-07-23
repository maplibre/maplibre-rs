use std::rc::Rc;

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

mod graph_node;
mod system;

pub mod environment;
pub mod map;
pub mod window;

pub async fn create_headless_renderer(
    tile_size: u32,
    cache_path: Option<String>,
) -> (Kernel<HeadlessEnvironment>, Renderer) {
    let client = ReqwestHttpClient::new(cache_path);
    let mut kernel = KernelBuilder::new()
        .with_map_window_config(HeadlessMapWindowConfig::new(
            PhysicalSize::new(tile_size, tile_size).unwrap(),
        ))
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
    let window: HeadlessMapWindow = mwc.create().expect("failed to create headless window");

    let renderer = RendererBuilder::new()
        .build()
        .initialize_headless::<HeadlessMapWindowConfig>(&window)
        .await
        .expect("Failed to initialize renderer");

    (kernel, renderer)
}

/// Labels for the "draw" graph
mod draw_graph {
    pub const NAME: &str = "draw";
    // Labels for input nodes
    pub mod input {}
    // Labels for non-input nodes
    pub mod node {
        pub const MAIN_PASS: &str = "main_pass";
        pub const COPY: &str = "copy_pass";
    }
}

pub struct HeadlessPlugin {
    write_to_disk: bool,
}

impl HeadlessPlugin {
    pub fn new(write_to_disk: bool) -> Self {
        Self { write_to_disk }
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

        let draw_graph = graph
            .get_sub_graph_mut(draw_graph::NAME)
            .expect("Subgraph does not exist");
        draw_graph.add_node(draw_graph::node::COPY, CopySurfaceBufferNode::default());
        draw_graph
            .add_node_edge(draw_graph::node::MAIN_PASS, draw_graph::node::COPY)
            .unwrap(); // TODO: remove unwrap

        schedule.add_system_to_stage(
            RenderStageLabel::Cleanup,
            SystemContainer::new(WriteSurfaceBufferSystem::new(self.write_to_disk)),
        );

        // FIXME tcs: Is this good style?
        schedule.remove_stage(RenderStageLabel::Extract);
        resources.get_mut::<ViewTileSources>().unwrap().clear();
    }
}
