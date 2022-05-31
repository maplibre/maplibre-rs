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
use crate::io::tile_repository::StoredLayer;
use crate::stages::message::{
    LayerTessellateMessage, MessageReceiver, MessageSender, SharedThreadState, TessellateMessage,
    TileTessellateMessage,
};

mod message;
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
