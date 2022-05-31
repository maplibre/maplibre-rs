//! [Stages](Stage) for requesting and preparing data

use crate::coords::{WorldCoords, WorldTileCoords, Zoom};
use crate::error::Error;
use crate::io::geometry_index::GeometryIndex;
use crate::io::geometry_index::{IndexProcessor, IndexedGeometry, TileIndex};
use crate::io::pipeline::{PipelineContext, PipelineProcessor};
use crate::io::pipeline_steps::build_vector_tile_pipeline;
use crate::io::source_client::{HttpSourceClient, SourceClient};
use crate::io::tile_request_state::TileRequestState;
use crate::io::{TileRequest, TileRequestID};
use crate::render::ShaderVertex;
use crate::schedule::Schedule;
use crate::stages::populate_tile_store_stage::PopulateTileStore;
use crate::tessellation::zero_tessellator::ZeroTessellator;
use crate::tessellation::{IndexDataType, OverAlignedVertexBuffer};
use crate::{HttpClient, ScheduleMethod, Scheduler};
use geozero::mvt::tile;
use geozero::GeozeroDatasource;
use prost::Message;
use request_stage::RequestStage;
use std::collections::HashSet;
use std::fmt;
use std::sync::{mpsc, Arc, Mutex};

use crate::io::pipeline::Processable;

mod populate_tile_store_stage;
mod request_stage;

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

    let scheduler = Box::new(scheduler.take());

    schedule.add_stage(
        "request",
        RequestStage::new(shared_thread_state.clone(), http_source_client, scheduler),
    );
    schedule.add_stage(
        "populate_tile_store",
        PopulateTileStore::new(shared_thread_state, message_receiver),
    );
}

type MessageSender = mpsc::Sender<TessellateMessage>;
type MessageReceiver = mpsc::Receiver<TessellateMessage>;

/// [crate::io::TileTessellateMessage] or [crate::io::LayerTessellateMessage] tessellation message.
enum TessellateMessage {
    Tile(TileTessellateMessage),
    Layer(LayerTessellateMessage),
}

///  The result of the tessellation of a tile.
struct TileTessellateMessage {
    pub request_id: TileRequestID,
    pub coords: WorldTileCoords,
}

/// `TessellatedLayer` contains the result of the tessellation for a specific layer, otherwise
/// `UnavailableLayer` if the layer doesn't exist.
enum LayerTessellateMessage {
    UnavailableLayer {
        coords: WorldTileCoords,
        layer_name: String,
    },
    TessellatedLayer {
        coords: WorldTileCoords,
        buffer: OverAlignedVertexBuffer<ShaderVertex, IndexDataType>,
        /// Holds for each feature the count of indices.
        feature_indices: Vec<u32>,
        layer_data: tile::Layer,
    },
}

impl fmt::Debug for LayerTessellateMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "LayerTessellateMessage{}", self.get_coords())
    }
}

impl LayerTessellateMessage {
    pub fn get_coords(&self) -> WorldTileCoords {
        match self {
            LayerTessellateMessage::UnavailableLayer { coords, .. } => *coords,
            LayerTessellateMessage::TessellatedLayer { coords, .. } => *coords,
        }
    }

    pub fn layer_name(&self) -> &str {
        match self {
            LayerTessellateMessage::UnavailableLayer { layer_name, .. } => layer_name.as_str(),
            LayerTessellateMessage::TessellatedLayer { layer_data, .. } => &layer_data.name,
        }
    }
}

pub struct HeadedPipelineProcessor {
    state: SharedThreadState,
}

impl PipelineProcessor for HeadedPipelineProcessor {
    fn finished_tile_tesselation(&mut self, request_id: TileRequestID, coords: &WorldTileCoords) {
        self.state
            .message_sender
            .send(TessellateMessage::Tile(TileTessellateMessage {
                request_id,
                coords: *coords,
            }))
            .unwrap();
    }

    fn unavailable_layer(&mut self, coords: &WorldTileCoords, layer_name: &str) {
        self.state.message_sender.send(TessellateMessage::Layer(
            LayerTessellateMessage::UnavailableLayer {
                coords: *coords,
                layer_name: layer_name.to_owned(),
            },
        ));
    }

    fn finished_layer_tesselation(
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

    fn finished_layer_indexing(
        &mut self,
        coords: &WorldTileCoords,
        geometries: Vec<IndexedGeometry<f64>>,
    ) {
        if let Ok(mut geometry_index) = self.state.geometry_index.lock() {
            geometry_index.index_tile(&coords, TileIndex::Linear { list: geometries })
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
            let mut processor = HeadedPipelineProcessor {
                state: self.clone(),
            };
            let mut pipeline_context = PipelineContext {
                processor: Box::new(processor),
            };
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
        z: u8,
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
