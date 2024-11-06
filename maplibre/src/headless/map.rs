use std::{cell::RefCell, ops::Deref, rc::Rc};

use crate::{
    context::MapContext,
    coords::{WorldCoords, WorldTileCoords, Zoom, ZoomLevel, TILE_SIZE},
    headless::environment::HeadlessEnvironment,
    io::{
        apc::{Context, IntoMessage, Message, SendError},
        source_client::SourceFetchError,
        source_type::{SourceType, TessellateSource},
    },
    kernel::Kernel,
    map::MapError,
    plugin::Plugin,
    render::{eventually::Eventually, view_state::ViewState, Renderer},
    schedule::{Schedule, Stage},
    style::Style,
    tcs::world::World,
    vector::{
        process_vector_tile, AvailableVectorLayerData, DefaultVectorTransferables,
        LayerTessellated, ProcessVectorContext, VectorBufferPool, VectorLayerData,
        VectorLayersDataComponent, VectorTileRequest, VectorTransferables,
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
        plugins: Vec<Box<dyn Plugin<HeadlessEnvironment>>>,
    ) -> Result<Self, MapError> {
        let window_size = renderer.state().surface().size();

        let view_state = ViewState::new(
            window_size,
            WorldCoords::from((TILE_SIZE / 2., TILE_SIZE / 2.)),
            Zoom::default(),
            cgmath::Deg(0.0),
            cgmath::Rad(0.6435011087932844),
        );

        let mut world = World::default();
        let mut schedule = Schedule::default();
        let kernel = Rc::new(kernel);

        for plugin in &plugins {
            plugin.build(
                &mut schedule,
                kernel.clone(),
                &mut world,
                &mut renderer.render_graph,
            );
        }

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

        let pool = resources
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
        let context = HeadlessContext::default();
        let mut processor =
            ProcessVectorContext::<DefaultVectorTransferables, HeadlessContext>::new(context);

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
pub struct HeadlessContext {
    pub messages: Rc<RefCell<Vec<Message>>>,
}

impl Context for HeadlessContext {
    fn send_back<T: IntoMessage>(&self, message: T) -> Result<(), SendError> {
        self.messages.deref().borrow_mut().push(message.into());
        Ok(())
    }
}
