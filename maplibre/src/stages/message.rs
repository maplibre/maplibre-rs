use std::{fmt, sync::mpsc};

use geozero::mvt::tile;

use crate::{
    coords::WorldTileCoords,
    io::{tile_repository::StoredLayer, TileRequestID},
    render::ShaderVertex,
    tessellation::{IndexDataType, OverAlignedVertexBuffer},
};

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
