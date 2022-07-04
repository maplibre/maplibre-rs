use crate::context::{MapContext, ViewState};
use crate::coords::{LatLon, ViewRegion, Zoom};
use crate::error::Error;
use crate::io::tile_repository::TileRepository;
use crate::render::camera::ViewProjection;
use crate::render::graph::{Node, NodeRunError, RenderContext, RenderGraphContext, SlotInfo};
use crate::render::resource::{BufferDimensions, BufferedTextureHead, IndexEntry};
use crate::render::resource::{Head, TrackedRenderPass};
use crate::render::stages::RenderStageLabel;
use crate::render::{
    create_default_render_graph, draw_graph, register_default_render_stages, RenderState,
};
use crate::schedule::{Schedule, Stage};
use crate::{
    HttpClient, MapWindow, MapWindowConfig, Renderer, ScheduleMethod, Scheduler, Style, WindowSize,
};
use std::fs::File;
use std::future::Future;
use std::io::Write;
use std::iter;
use std::ops::{Deref, Range};
use std::sync::Arc;
use tokio::runtime::Handle;
use tokio::task;
use wgpu::{BufferAsyncError, BufferSlice};

pub struct HeadlessMapWindowConfig {
    pub size: WindowSize,
}

impl MapWindowConfig for HeadlessMapWindowConfig {
    type MapWindow = HeadlessMapWindow;

    fn create(&self) -> Self::MapWindow {
        Self::MapWindow { size: self.size }
    }
}

pub struct HeadlessMapWindow {
    size: WindowSize,
}

impl MapWindow for HeadlessMapWindow {
    fn size(&self) -> WindowSize {
        self.size
    }
}

pub struct HeadlessMap<MWC, SM, HC>
where
    MWC: MapWindowConfig,
    SM: ScheduleMethod,
    HC: HttpClient,
{
    pub map_schedule: HeadlessMapSchedule<MWC, SM, HC>,
    pub window: MWC::MapWindow,
}

impl<MWC, SM, HC> HeadlessMap<MWC, SM, HC>
where
    MWC: MapWindowConfig,
    SM: ScheduleMethod,
    HC: HttpClient,
{
    pub fn map_schedule_mut(&mut self) -> &mut HeadlessMapSchedule<MWC, SM, HC> {
        &mut self.map_schedule
    }
}

/// Stores the state of the map, dispatches tile fetching and caching, tessellation and drawing.
pub struct HeadlessMapSchedule<MWC, SM, HC>
where
    MWC: MapWindowConfig,
    SM: ScheduleMethod,
    HC: HttpClient,
{
    map_window_config: MWC,

    pub map_context: MapContext,

    schedule: Schedule,
    scheduler: Scheduler<SM>,
    http_client: HC,
}

impl<MWC, SM, HC> HeadlessMapSchedule<MWC, SM, HC>
where
    MWC: MapWindowConfig,
    SM: ScheduleMethod,
    HC: HttpClient,
{
    pub fn new(
        map_window_config: MWC,
        window_size: WindowSize,
        renderer: Renderer,
        scheduler: Scheduler<SM>,
        http_client: HC,
        style: Style,
    ) -> Self {
        let view_state = ViewState::new(
            &window_size,
            style.zoom.map(|zoom| Zoom::new(zoom)).unwrap_or_default(),
            style
                .center
                .map(|center| LatLon::new(center[0], center[1]))
                .unwrap_or_default(),
            style.pitch.unwrap_or_default(),
        );
        let tile_repository = TileRepository::new();
        let mut schedule = Schedule::default();

        let mut graph = create_default_render_graph().unwrap();
        let draw_graph = graph.get_sub_graph_mut(draw_graph::NAME).unwrap();
        draw_graph.add_node(draw_graph::node::COPY, CopySurfaceBufferNode::default());
        draw_graph
            .add_node_edge(draw_graph::node::MAIN_PASS, draw_graph::node::COPY)
            .unwrap();

        register_default_render_stages(graph, &mut schedule);

        schedule.add_stage(
            RenderStageLabel::Cleanup,
            WriteSurfaceBufferStage::default(),
        );

        Self {
            map_window_config,
            map_context: MapContext {
                view_state,
                style,
                tile_repository,
                renderer,
            },
            schedule,
            scheduler,
            http_client,
        }
    }

    #[tracing::instrument(name = "update_and_redraw", skip_all)]
    pub fn update_and_redraw(&mut self) -> Result<(), Error> {
        self.schedule.run(&mut self.map_context);

        Ok(())
    }

    pub fn schedule(&self) -> &Schedule {
        &self.schedule
    }
    pub fn scheduler(&self) -> &Scheduler<SM> {
        &self.scheduler
    }
    pub fn http_client(&self) -> &HC {
        &self.http_client
    }
}

/// Node which copies the contents of the GPU-side texture in [`BufferedTextureHead`] to an
/// unmapped GPU-side buffer. This buffer will be mapped in
/// [`crate::render::stages::write_surface_buffer_stage::WriteSurfaceBufferStage`].
#[derive(Default)]
pub struct CopySurfaceBufferNode {}

impl CopySurfaceBufferNode {
    pub fn new() -> Self {
        Self {}
    }
}

impl Node for CopySurfaceBufferNode {
    fn input(&self) -> Vec<SlotInfo> {
        vec![]
    }

    fn update(&mut self, _state: &mut RenderState) {}

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        RenderContext {
            command_encoder, ..
        }: &mut RenderContext,
        state: &RenderState,
    ) -> Result<(), NodeRunError> {
        match state.surface.head() {
            Head::Headed(_) => {}
            Head::Headless(buffered_texture) => {
                let size = state.surface.size();
                command_encoder.copy_texture_to_buffer(
                    buffered_texture.texture.as_image_copy(),
                    wgpu::ImageCopyBuffer {
                        buffer: &buffered_texture.output_buffer,
                        layout: wgpu::ImageDataLayout {
                            offset: 0,
                            bytes_per_row: Some(
                                std::num::NonZeroU32::new(
                                    buffered_texture.buffer_dimensions.padded_bytes_per_row as u32,
                                )
                                .unwrap(),
                            ),
                            rows_per_image: None,
                        },
                    },
                    wgpu::Extent3d {
                        width: size.width() as u32,
                        height: size.height() as u32,
                        depth_or_array_layers: 1,
                    },
                );
            }
        }

        Ok(())
    }
}

/// Stage which writes the current contents of the GPU/CPU buffer in [`BufferedTextureHead`]
/// to disk as PNG.
#[derive(Default)]
pub struct WriteSurfaceBufferStage {
    frame: u64,
}

impl Stage for WriteSurfaceBufferStage {
    fn run(
        &mut self,
        MapContext {
            renderer: Renderer { state, device, .. },
            ..
        }: &mut MapContext,
    ) {
        match state.surface.head() {
            Head::Headed(_) => {}
            Head::Headless(buffered_texture) => {
                let buffered_texture: Arc<BufferedTextureHead> = buffered_texture.clone();

                let device = device.clone();
                let current_frame = self.frame;

                task::block_in_place(|| {
                    Handle::current().block_on(async {
                        buffered_texture
                            .create_png(&device, format!("frame_{}.png", current_frame).as_str())
                            .await;
                    })
                });

                self.frame += 1;
            }
        }
    }
}

pub mod utils {
    use crate::coords::WorldTileCoords;
    use crate::io::pipeline::PipelineProcessor;
    use crate::io::tile_repository::StoredLayer;
    use crate::io::RawLayer;
    use crate::render::ShaderVertex;
    use crate::tessellation::{IndexDataType, OverAlignedVertexBuffer};

    #[derive(Default)]
    pub struct HeadlessPipelineProcessor {
        pub layers: Vec<StoredLayer>,
    }

    impl PipelineProcessor for HeadlessPipelineProcessor {
        fn layer_tesselation_finished(
            &mut self,
            coords: &WorldTileCoords,
            buffer: OverAlignedVertexBuffer<ShaderVertex, IndexDataType>,
            feature_indices: Vec<u32>,
            layer_data: RawLayer,
        ) {
            self.layers.push(StoredLayer::TessellatedLayer {
                coords: *coords,
                buffer,
                feature_indices,
                layer_data,
            })
        }
    }
}
