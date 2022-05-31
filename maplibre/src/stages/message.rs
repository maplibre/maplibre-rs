use crate::coords::{WorldCoords, WorldTileCoords, Zoom};
use crate::error::Error;
use crate::io::geometry_index::{GeometryIndex, IndexedGeometry};
use crate::io::pipeline::PipelineContext;
use crate::io::pipeline::Processable;
use crate::io::pipeline_steps::build_vector_tile_pipeline;
use crate::io::tile_repository::StoredLayer;
use crate::io::tile_request_state::TileRequestState;
use crate::io::{TileRequest, TileRequestID};
use crate::render::ShaderVertex;
use crate::stages::HeadedPipelineProcessor;
use crate::tessellation::{IndexDataType, OverAlignedVertexBuffer};
use geozero::mvt::tile;
use std::fmt;
use std::sync::{mpsc, Arc, Mutex};

pub type MessageSender = mpsc::Sender<TessellateMessage>;
pub type MessageReceiver = mpsc::Receiver<TessellateMessage>;

/// [crate::io::TileTessellateMessage] or [crate::io::LayerTessellateMessage] tessellation message.
pub enum TessellateMessage {
    Tile(TileTessellateMessage),
    Layer(LayerTessellateMessage),
}

///  The result of the tessellation of a tile.
pub struct TileTessellateMessage {
    pub request_id: TileRequestID,
    pub coords: WorldTileCoords,
}

/// `TessellatedLayer` contains the result of the tessellation for a specific layer, otherwise
/// `UnavailableLayer` if the layer doesn't exist.
pub enum LayerTessellateMessage {
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

impl Into<StoredLayer> for LayerTessellateMessage {
    fn into(self) -> StoredLayer {
        match self {
            LayerTessellateMessage::UnavailableLayer { coords, layer_name } => {
                StoredLayer::UnavailableLayer { coords, layer_name }
            }
            LayerTessellateMessage::TessellatedLayer {
                coords,
                buffer,
                feature_indices,
                layer_data,
            } => StoredLayer::TessellatedLayer {
                coords,
                buffer,
                feature_indices,
                layer_data,
            },
        }
    }
}

impl fmt::Debug for LayerTessellateMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "LayerTessellateMessage{}",
            match self {
                LayerTessellateMessage::UnavailableLayer { coords, .. } => coords,
                LayerTessellateMessage::TessellatedLayer { coords, .. } => coords,
            }
        )
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
