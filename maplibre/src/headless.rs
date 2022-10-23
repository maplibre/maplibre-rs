use std::{
    collections::HashSet,
    fs::File,
    future::Future,
    io::Write,
    iter,
    marker::PhantomData,
    ops::{Deref, Range},
    sync::Arc,
};

use tokio::{runtime::Handle, task};
use wgpu::{BufferAsyncError, BufferSlice};

use crate::{
    context::{MapContext, ViewState},
    coords::{LatLon, ViewRegion, WorldCoords, WorldTileCoords, Zoom, TILE_SIZE},
    error::Error,
    headless::utils::HeadlessPipelineProcessor,
    io::{
        apc::{AsyncProcedureCall, SchedulerAsyncProcedureCall},
        pipeline::{PipelineContext, Processable},
        source_client::HttpSourceClient,
        tile_pipelines::build_vector_tile_pipeline,
        tile_repository::{StoredLayer, TileRepository},
        transferables::{DefaultTransferables, Transferables},
        TileRequest,
    },
    render::{
        camera::ViewProjection,
        create_default_render_graph, draw_graph,
        eventually::Eventually,
        graph::{Node, NodeRunError, RenderContext, RenderGraphContext, SlotInfo},
        register_default_render_stages,
        resource::{BufferDimensions, BufferedTextureHead, Head, IndexEntry, TrackedRenderPass},
        stages::RenderStageLabel,
        RenderState,
    },
    schedule::{Schedule, Stage},
    Environment, HttpClient, MapWindow, MapWindowConfig, Renderer, Scheduler, Style, WindowSize,
};

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

pub struct HeadlessEnvironment<
    S: Scheduler,
    HC: HttpClient,
    T: Transferables,
    APC: AsyncProcedureCall<T, HC>,
> {
    phantom_s: PhantomData<S>,
    phantom_hc: PhantomData<HC>,
    phantom_t: PhantomData<T>,
    phantom_apc: PhantomData<APC>,
}

impl<S: Scheduler, HC: HttpClient, T: Transferables, APC: AsyncProcedureCall<T, HC>> Environment
    for HeadlessEnvironment<S, HC, T, APC>
{
    type MapWindowConfig = HeadlessMapWindowConfig;
    type AsyncProcedureCall = APC;
    type Scheduler = S;
    type HttpClient = HC;
    type Transferables = T;
}

pub struct HeadlessMap<E: Environment> {
    pub map_schedule: HeadlessMapSchedule<E>,
    pub window: <E::MapWindowConfig as MapWindowConfig>::MapWindow,
}

impl<E: Environment> HeadlessMap<E> {
    pub fn map_schedule_mut(&mut self) -> &mut HeadlessMapSchedule<E> {
        &mut self.map_schedule
    }
}

/// Stores the state of the map, dispatches tile fetching and caching, tessellation and drawing.
pub struct HeadlessMapSchedule<E: Environment> {
    map_window_config: E::MapWindowConfig,

    pub map_context: MapContext,

    schedule: Schedule,
    scheduler: E::Scheduler,
    http_client: E::HttpClient,
}

impl<E: Environment> HeadlessMapSchedule<E> {
    pub fn new(
        map_window_config: E::MapWindowConfig,
        window_size: WindowSize,
        renderer: Renderer,
        scheduler: E::Scheduler,
        http_client: E::HttpClient,
        style: Style,
    ) -> Self {
        let view_state = ViewState::new(
            &window_size,
            WorldCoords::from((TILE_SIZE / 2., TILE_SIZE / 2.)),
            Zoom::default(),
            0.0,
            cgmath::Deg(110.0),
        );
        let tile_repository = TileRepository::new();
        let mut schedule = Schedule::default();

        let mut graph = create_default_render_graph().unwrap(); // TODO: remove unwrap
        let draw_graph = graph.get_sub_graph_mut(draw_graph::NAME).unwrap(); // TODO: remove unwrap
        draw_graph.add_node(draw_graph::node::COPY, CopySurfaceBufferNode::default());
        draw_graph
            .add_node_edge(draw_graph::node::MAIN_PASS, draw_graph::node::COPY)
            .unwrap(); // TODO: remove unwrap

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
    pub fn scheduler(&self) -> &E::Scheduler {
        &self.scheduler
    }
    pub fn http_client(&self) -> &E::HttpClient {
        &self.http_client
    }

    pub async fn fetch_process(&mut self, coords: &WorldTileCoords) -> Option<()> {
        let source_layers: HashSet<String> = self
            .map_context
            .style
            .layers
            .iter()
            .filter_map(|layer| layer.source_layer.clone())
            .collect();

        let http_source_client: HttpSourceClient<E::HttpClient> =
            HttpSourceClient::new(self.http_client.clone());

        let data = http_source_client
            .fetch(&coords)
            .await
            .unwrap() // TODO: remove unwrap
            .into_boxed_slice();

        let mut pipeline_context = PipelineContext::new(HeadlessPipelineProcessor::default());
        let pipeline = build_vector_tile_pipeline();

        let request = TileRequest {
            coords: WorldTileCoords::default(),
            layers: source_layers,
        };

        pipeline.process((request, data), &mut pipeline_context);

        let mut processor = pipeline_context
            .take_processor::<HeadlessPipelineProcessor>()
            .unwrap(); // TODO: remove unwrap

        if let Eventually::Initialized(pool) = self.map_context.renderer.state.buffer_pool_mut() {
            pool.clear();
        }

        self.map_context.tile_repository.clear();

        while let Some(layer) = processor.layers.pop() {
            self.map_context
                .tile_repository
                .create_tile(&layer.get_coords());
            self.map_context
                .tile_repository
                .put_tessellated_layer(layer);
        }

        Some(())
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
        let surface = state.surface();
        match surface.head() {
            Head::Headed(_) => {}
            Head::Headless(buffered_texture) => {
                let size = surface.size();
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
                                .unwrap(), // TODO: remove unwrap
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
        let surface = state.surface();
        match surface.head() {
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
    use crate::{
        coords::WorldTileCoords,
        io::{pipeline::PipelineProcessor, tile_repository::StoredLayer, RawLayer},
        render::ShaderVertex,
        tessellation::{IndexDataType, OverAlignedVertexBuffer},
    };

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
                layer_name: layer_data.name,
                buffer,
                feature_indices,
            })
        }
    }
}
