use std::{borrow::BorrowMut, cell::RefCell, ops::Deref, rc::Rc, sync::Arc};

use crate::{
    context::MapContext,
    coords::{WorldCoords, WorldTileCoords, Zoom, ZoomLevel, TILE_SIZE},
    ecs::{system::SystemContainer, world::World},
    headless::{
        environment::HeadlessEnvironment, graph_node::CopySurfaceBufferNode,
        stage::WriteSurfaceBufferSystem,
    },
    io::{
        apc::{Context, IntoMessage, Message, SendError},
        source_client::SourceFetchError,
        source_type::{SourceType, TessellateSource},
        RawLayer,
    },
    kernel::Kernel,
    map::MapError,
    plugin::Plugin,
    raster::{DefaultRasterTransferables, RasterPlugin},
    render::{
        draw_graph, eventually::Eventually, initialize_default_render_graph,
        register_default_render_stages, stages::RenderStageLabel, Renderer, ShaderVertex,
    },
    schedule::{Schedule, Stage},
    style::Style,
    tessellation::{IndexDataType, OverAlignedVertexBuffer},
    vector::{
        process_vector_tile, AvailableVectorLayerData, DefaultLayerTesselated,
        DefaultVectorTransferables, LayerTessellated, ProcessVectorContext, VectorBufferPool,
        VectorLayerData, VectorLayersDataComponent, VectorLayersIndicesComponent, VectorPlugin,
        VectorTileRequest, VectorTransferables,
    },
};

pub struct HeadlessMap {
    kernel: Rc<Kernel<HeadlessEnvironment>>,
    schedule: Schedule,
    map_context: MapContext,
}

impl HeadlessMap {
    pub fn new(
        style: Style,
        mut renderer: Renderer,
        kernel: Kernel<HeadlessEnvironment>,
        write_to_disk: bool,
    ) -> Result<Self, MapError> {
        let window_size = renderer.state().surface().size();

        let mut world = World::new(
            window_size,
            WorldCoords::from((TILE_SIZE / 2., TILE_SIZE / 2.)),
            Zoom::default(),
            cgmath::Deg(0.0),
        );

        let graph = &mut renderer.render_graph;
        initialize_default_render_graph(graph).unwrap();
        let draw_graph = graph
            .get_sub_graph_mut(draw_graph::NAME)
            .expect("Subgraph does not exist");
        draw_graph.add_node(draw_graph::node::COPY, CopySurfaceBufferNode::default());
        draw_graph
            .add_node_edge(draw_graph::node::MAIN_PASS, draw_graph::node::COPY)
            .unwrap(); // TODO: remove unwrap

        let mut schedule = Schedule::default();
        register_default_render_stages(&mut schedule);

        let kernel = Rc::new(kernel);
        VectorPlugin::<DefaultVectorTransferables>::default().build(
            &mut schedule,
            kernel.clone(),
            &mut world,
        );
        RasterPlugin::<DefaultRasterTransferables>::default().build(
            &mut schedule,
            kernel.clone(),
            &mut world,
        );

        schedule.add_system_to_stage(
            RenderStageLabel::Cleanup,
            SystemContainer::new(WriteSurfaceBufferSystem::new(write_to_disk)),
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

    pub fn render_tile(&mut self, layers: Vec<Box<DefaultLayerTesselated>>) {
        let context = &mut self.map_context;

        context.world.tiles.clear();

        context
            .world
            .tiles
            .spawn_mut((0, 0, ZoomLevel::default()).into())
            .unwrap()
            .insert(VectorLayersDataComponent {
                done: true,
                layers: layers
                    .into_iter()
                    .map(|layer| {
                        VectorLayerData::Available(AvailableVectorLayerData {
                            coords: layer.coords,
                            source_layer: layer.layer_data.name,
                            buffer: layer.buffer,
                            feature_indices: layer.feature_indices,
                        })
                    })
                    .collect::<Vec<_>>(),
            })
            .insert(VectorLayersIndicesComponent::default());

        self.schedule.run(context);

        if let Some(Eventually::Initialized(pool)) = context
            .world
            .resources
            .query_mut::<&mut Eventually<VectorBufferPool>>()
        {
            pool.clear();
        } else {
            panic!("failed to clear");
        }
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
    ) -> Vec<Box<DefaultLayerTesselated>> {
        let context = SimpleContext::default();
        let mut processor =
            ProcessVectorContext::<DefaultVectorTransferables, SimpleContext>::new(context);

        let target_coords = WorldTileCoords::default(); // load to 0,0,0
        process_vector_tile(
            &tile_data,
            VectorTileRequest {
                coords: target_coords,
                layers: source_layers
                    .iter()
                    .map(|layer| layer.to_string())
                    .collect(),
            },
            &mut processor,
        )
        .expect("Failed to process!");

        let messages = processor.take_context().messages.deref().take();
        let layers = messages.into_iter()
            .filter(|message| message.tag() == <DefaultVectorTransferables as VectorTransferables>::LayerTessellated::message_tag())
            .map(|message| message.into_transferable::<<DefaultVectorTransferables as VectorTransferables>::LayerTessellated>())
            .collect::<Vec<_>>();

        layers
    }
}

#[derive(Default, Clone)]
pub struct SimpleContext {
    pub messages: Rc<RefCell<Vec<Message>>>,
}

impl Context for SimpleContext {
    fn send<T: IntoMessage>(&self, message: T) -> Result<(), SendError> {
        self.messages.deref().borrow_mut().push(message.into());
        Ok(())
    }
}
