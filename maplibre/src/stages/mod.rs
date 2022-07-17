//! [Stages](Stage) for requesting and preparing data

use std::sync::{mpsc, Arc, Mutex};

use geozero::{mvt::tile, GeozeroDatasource};
use request_stage::RequestStage;

use crate::{
    coords::{WorldCoords, WorldTileCoords, Zoom, ZoomLevel},
    error::Error,
    io::{
        geometry_index::{GeometryIndex, IndexedGeometry, TileIndex},
        pipeline::{PipelineContext, PipelineProcessor, Processable},
        source_client::HttpSourceClient,
        tile_pipelines::build_vector_tile_pipeline,
        tile_request_state::TileRequestState,
        TileRequest, TileRequestID,
    },
    render::ShaderVertex,
    schedule::Schedule,
    stages::{
        message::{
            LayerTessellateMessage, MessageReceiver, MessageSender, TessellateMessage,
            TileTessellateMessage,
        },
        populate_tile_store_stage::PopulateTileStore,
    },
    tessellation::{IndexDataType, OverAlignedVertexBuffer},
    HttpClient, ScheduleMethod, Scheduler,
};

mod message;
mod populate_tile_store_stage;
mod request_stage;

/// Register stages required for requesting and preparing new tiles.
pub fn register_stages<HC: HttpClient, SM: ScheduleMethod>(
    schedule: &mut Schedule,
    http_source_client: HttpSourceClient<HC>,
    scheduler: Box<Scheduler<SM>>,
) {
    let (message_sender, message_receiver): (MessageSender, MessageReceiver) = mpsc::channel();
    let shared_thread_state = SharedThreadState {
        tile_request_state: Arc::new(Mutex::new(TileRequestState::new())),
        message_sender,
        geometry_index: Arc::new(Mutex::new(GeometryIndex::new())),
    };

    schedule.add_stage(
        "request",
        RequestStage::new(shared_thread_state.clone(), http_source_client, *scheduler),
    );
    schedule.add_stage(
        "populate_tile_store",
        PopulateTileStore::new(shared_thread_state, message_receiver),
    );
}

pub struct HeadedPipelineProcessor {
    state: SharedThreadState,
}

impl PipelineProcessor for HeadedPipelineProcessor {
    fn tile_finished(&mut self, request_id: TileRequestID, coords: &WorldTileCoords) {
        self.state
            .message_sender
            .send(TessellateMessage::Tile(TileTessellateMessage {
                request_id,
                coords: *coords,
            }))
            .unwrap();
    }

    fn layer_unavailable(&mut self, coords: &WorldTileCoords, layer_name: &str) {
        self.state
            .message_sender
            .send(TessellateMessage::Layer(
                LayerTessellateMessage::UnavailableLayer {
                    coords: *coords,
                    layer_name: layer_name.to_owned(),
                },
            ))
            .unwrap();
    }

    fn layer_tesselation_finished(
        &mut self,
        coords: &WorldTileCoords,
        buffer: OverAlignedVertexBuffer<ShaderVertex, IndexDataType>,
        feature_indices: Vec<u32>,
        layer_data: tile::Layer,
    ) {
        self.state
            .message_sender
            .send(TessellateMessage::Layer(
                LayerTessellateMessage::TessellatedLayer {
                    coords: *coords,
                    buffer,
                    feature_indices,
                    layer_data,
                },
            ))
            .unwrap();
    }

    fn layer_indexing_finished(
        &mut self,
        coords: &WorldTileCoords,
        geometries: Vec<IndexedGeometry<f64>>,
    ) {
        if let Ok(mut geometry_index) = self.state.geometry_index.lock() {
            geometry_index.index_tile(coords, TileIndex::Linear { list: geometries })
        }
    }
}

/// Stores and provides access to the thread safe data shared between the schedulers.
#[derive(Clone)]
pub struct SharedThreadState {
    pub tile_request_state: Arc<Mutex<TileRequestState>>,
    pub message_sender: mpsc::Sender<TessellateMessage>,
    pub geometry_index: Arc<Mutex<GeometryIndex>>,
}

impl SharedThreadState {
    fn get_tile_request(&self, request_id: TileRequestID) -> Option<TileRequest> {
        self.tile_request_state
            .lock()
            .ok()
            .and_then(|tile_request_state| tile_request_state.get_tile_request(request_id).cloned())
    }

    #[tracing::instrument(skip_all)]
    pub fn process_tile(&self, request_id: TileRequestID, data: Box<[u8]>) -> Result<(), Error> {
        if let Some(tile_request) = self.get_tile_request(request_id) {
            let mut pipeline_context = PipelineContext::new(HeadedPipelineProcessor {
                state: self.clone(),
            });
            let pipeline = build_vector_tile_pipeline();
            pipeline.process((tile_request, request_id, data), &mut pipeline_context);
        }

        Ok(())
    }

    pub fn tile_unavailable(
        &self,
        coords: &WorldTileCoords,
        request_id: TileRequestID,
    ) -> Result<(), Error> {
        if let Some(tile_request) = self.get_tile_request(request_id) {
            for to_load in &tile_request.layers {
                tracing::warn!("layer {} at {} unavailable", to_load, coords);
                self.message_sender.send(TessellateMessage::Layer(
                    LayerTessellateMessage::UnavailableLayer {
                        coords: tile_request.coords,
                        layer_name: to_load.to_string(),
                    },
                ))?;
            }
        }

        Ok(())
    }

    #[tracing::instrument(skip_all)]
    pub fn query_point(
        &self,
        world_coords: &WorldCoords,
        z: ZoomLevel,
        zoom: Zoom,
    ) -> Option<Vec<IndexedGeometry<f64>>> {
        if let Ok(geometry_index) = self.geometry_index.lock() {
            geometry_index
                .query_point(world_coords, z, zoom)
                .map(|geometries| {
                    geometries
                        .iter()
                        .cloned()
                        .cloned()
                        .collect::<Vec<IndexedGeometry<f64>>>()
                })
        } else {
            unimplemented!()
        }
    }
}
