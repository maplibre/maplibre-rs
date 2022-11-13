use std::collections::HashSet;

use crate::{
    context::MapContext,
    coords::{WorldCoords, WorldTileCoords, Zoom, TILE_SIZE},
    error::Error,
    headless::{
        environment::HeadlessEnvironment, graph_node::CopySurfaceBufferNode,
        stage::WriteSurfaceBufferStage,
    },
    io::{
        pipeline::{PipelineContext, PipelineProcessor, Processable},
        tile_pipelines::build_vector_tile_pipeline,
        tile_repository::{StoredLayer, StoredTile},
        RawLayer, TileRequest,
    },
    kernel::Kernel,
    render::{
        create_default_render_graph, draw_graph, eventually::Eventually,
        register_default_render_stages, stages::RenderStageLabel, Renderer, ShaderVertex,
    },
    schedule::{Schedule, Stage},
    style::Style,
    tessellation::{IndexDataType, OverAlignedVertexBuffer},
    world::World,
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
    ) -> Result<Self, Error> {
        let window_size = renderer.state().surface().size();

        let world = World::new(
            window_size,
            WorldCoords::from((TILE_SIZE / 2., TILE_SIZE / 2.)),
            Zoom::default(),
            cgmath::Deg(0.0),
        );

        let mut graph = create_default_render_graph()?;
        let draw_graph = graph
            .get_sub_graph_mut(draw_graph::NAME)
            .expect("Subgraph does not exist");
        draw_graph.add_node(draw_graph::node::COPY, CopySurfaceBufferNode::default());
        draw_graph
            .add_node_edge(draw_graph::node::MAIN_PASS, draw_graph::node::COPY)
            .unwrap(); // TODO: remove unwrap

        let mut schedule = Schedule::default();
        register_default_render_stages(graph, &mut schedule);
        schedule.add_stage(
            RenderStageLabel::Cleanup,
            WriteSurfaceBufferStage::default(),
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

    pub fn render_tile(&mut self, tile: StoredTile) -> Result<(), Error> {
        let context = &mut self.map_context;

        if let Eventually::Initialized(pool) = context.renderer.state.buffer_pool_mut() {
            pool.clear();
        } else {
            // TODO return error
        }

        context.world.tile_repository.clear();

        context.world.tile_repository.put_tile(tile);

        self.schedule.run(&mut self.map_context);
        Ok(())
    }

    pub async fn fetch_tile(
        &self,
        coords: WorldTileCoords,
        source_layers: &[&str],
    ) -> Result<StoredTile, Error> {
        let source_client = self.kernel.source_client();

        let data = source_client.fetch(&coords).await?.into_boxed_slice();

        let mut pipeline_context = PipelineContext::new(HeadlessPipelineProcessor::default());
        let pipeline = build_vector_tile_pipeline();

        let target_coords = WorldTileCoords::default(); // load to 0,0,0
        pipeline.process(
            (
                TileRequest {
                    coords: target_coords,
                    layers: source_layers
                        .iter()
                        .map(|layer| layer.to_string())
                        .collect::<HashSet<String>>(),
                },
                data,
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
    ) -> Result<(), Error> {
        self.layers.push(StoredLayer::TessellatedLayer {
            coords: *coords,
            layer_name: layer_data.name,
            buffer,
            feature_indices,
        });
        Ok(())
    }
}
