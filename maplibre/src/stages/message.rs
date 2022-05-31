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
