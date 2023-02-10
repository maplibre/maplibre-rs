use crate::{
    context::MapContext,
    coords::{WorldCoords, WorldTileCoords, Zoom, TILE_SIZE},
    ecs::world::World,
    headless::{
        environment::HeadlessEnvironment, graph_node::CopySurfaceBufferNode,
        stage::WriteSurfaceBufferStage,
    },
    io::{
        pipeline::{PipelineContext, PipelineError, PipelineProcessor, Processable},
        raster_pipeline::{build_vector_tile_pipeline, VectorTileRequest},
        source_client::SourceFetchError,
        source_type::{SourceType, TessellateSource},
        RawLayer,
    },
    kernel::Kernel,
    map::MapError,
    render::{
        draw_graph, eventually::Eventually, register_default_render_stages,
        stages::RenderStageLabel, Renderer, ShaderVertex,
    },
    schedule::{Schedule, Stage},
    style::Style,
    tessellation::{IndexDataType, OverAlignedVertexBuffer},
};

pub struct HeadlessMap {
    kernel: Kernel<HeadlessEnvironment>,
    schedule: Schedule,
    map_context: MapContext,
}

impl HeadlessMap {
    pub fn new(
        style: Style,
        renderer: Renderer,
        kernel: Kernel<HeadlessEnvironment>,
        write_to_disk: bool,
    ) -> Result<Self, MapError> {
        let window_size = renderer.state().surface().size();

        let world = World::new(
            window_size,
            WorldCoords::from((TILE_SIZE / 2., TILE_SIZE / 2.)),
            Zoom::default(),
            cgmath::Deg(0.0),
        );

        let mut graph = create_default_render_graph().map_err(MapError::RenderGraphInit)?;
        let draw_graph = graph
            .get_sub_graph_mut(draw_graph::NAME)
            .expect("Subgraph does not exist");
        draw_graph.add_node(draw_graph::node::COPY, CopySurfaceBufferNode::default());
        draw_graph
            .add_node_edge(draw_graph::node::MAIN_PASS, draw_graph::node::COPY)
            .unwrap(); // TODO: remove unwrap

        let mut schedule = Schedule::default();
        register_default_render_stages(&mut schedule);
        schedule.add_stage(
            RenderStageLabel::Cleanup,
            WriteSurfaceBufferStage::new(write_to_disk),
        );

        Ok(Self {
            kernel,
            map_context: MapContext {
                style,
                world,
                renderer,
            },
            schedule,
        })
    }

    pub fn render_tile(&mut self, tile: StoredTile) {
        let context = &mut self.map_context;

        if let Eventually::Initialized(pool) = context.renderer.state.buffer_pool_mut() {
            pool.clear();
        } else {
            // TODO return error
        }

        context.world.tile_repository.clear();

        context.world.tile_repository.put_tile(tile);

        self.schedule.run(&mut self.map_context);
    }

    pub async fn fetch_tile(&self, coords: WorldTileCoords) -> Result<Box<[u8]>, SourceFetchError> {
        let source_client = self.kernel.source_client();
        let data = source_client
            .fetch(
                &coords,
                &SourceType::Tessellate(TessellateSource::default()),
            )
            .await?
            .into_boxed_slice();
        Ok(data)
    }

    pub async fn process_tile(
        &self,
        tile_data: Box<[u8]>,
        source_layers: &[&str],
    ) -> Result<StoredTile, PipelineError> {
        let mut pipeline_context = PipelineContext::new(HeadlessPipelineProcessor::default());
        let pipeline = build_vector_tile_pipeline();

        let target_coords = WorldTileCoords::default(); // load to 0,0,0
        pipeline.process(
            (
                VectorTileRequest {
                    coords: target_coords,
                    layers: source_layers
                        .iter()
                        .map(|layer| layer.to_string())
                        .collect(),
                },
                tile_data,
            ),
            &mut pipeline_context,
        )?;

        let processor = pipeline_context
            .take_processor::<HeadlessPipelineProcessor>()
            .expect("Unable to get processor");

        Ok(StoredTile::success(target_coords, processor.layers))
    }
}

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
    ) -> Result<(), PipelineError> {
        self.layers.push(StoredLayer::TessellatedLayer {
            coords: *coords,
            layer_name: layer_data.name,
            buffer,
            feature_indices,
        });
        Ok(())
    }
}
