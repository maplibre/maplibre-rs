use std::{cell::RefCell, ops::Deref, rc::Rc};

use crate::{
    context::MapContext,
    coords::{WorldCoords, WorldTileCoords, Zoom, ZoomLevel, TILE_SIZE},
    headless::{
        environment::HeadlessEnvironment, graph_node::CopySurfaceBufferNode,
        system::WriteSurfaceBufferSystem,
    },
    io::{
        apc::{Context, IntoMessage, Message, SendError},
        source_client::SourceFetchError,
        source_type::{SourceType, TessellateSource},
    },
    kernel::Kernel,
    map::MapError,
    plugin::Plugin,
    raster::{DefaultRasterTransferables, RasterPlugin},
    render::{
        draw_graph, eventually::Eventually, initialize_default_render_graph,
        register_default_render_stages, RenderStageLabel, Renderer,
    },
    schedule::{Schedule, Stage},
    style::Style,
    tcs::{system::SystemContainer, world::World},
    vector::{
        process_vector_tile, AvailableVectorLayerData, DefaultVectorTransferables,
        LayerTessellated, ProcessVectorContext, VectorBufferPool, VectorLayerData,
        VectorLayersDataComponent, VectorPlugin, VectorTileRequest, VectorTransferables,
    },
    view_state::ViewState,
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

        let view_state = ViewState::new(
            window_size,
            WorldCoords::from((TILE_SIZE / 2., TILE_SIZE / 2.)),
            Zoom::default(),
            cgmath::Deg(0.0),
            cgmath::Deg(110.0),
        );

        let mut world = World::default();

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

        // FIXME tcs: Is this good style?
        schedule.remove_stage(RenderStageLabel::Extract);

        schedule.add_system_to_stage(
            RenderStageLabel::Cleanup,
            SystemContainer::new(WriteSurfaceBufferSystem::new(write_to_disk)),
        );

        Ok(Self {
            kernel,
            map_context: MapContext {
                style,
                view_state,
                world,
                renderer,
            },
            schedule,
        })
    }

    pub fn render_tile(
        &mut self,
        layers: Vec<Box<<DefaultVectorTransferables as VectorTransferables>::LayerTessellated>>,
    ) {
        let context = &mut self.map_context;
        let tiles = &mut context.world.tiles;

        tiles
            .spawn_mut((0, 0, ZoomLevel::default()).into())
            .expect("unable to spawn tile")
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
            });

        self.schedule.run(context);

        let resources = &mut context.world.resources;
        let tiles = &mut context.world.tiles;

        tiles.clear();
        let mut pool = resources
            .query_mut::<&mut Eventually<VectorBufferPool>>() // FIXME tcs: we access internals of the vector plugin here
            .expect("VectorBufferPool not found")
            .expect_initialized_mut("VectorBufferPool not initialized");

        pool.clear();
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
    ) -> Vec<Box<<DefaultVectorTransferables as VectorTransferables>::LayerTessellated>> {
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
