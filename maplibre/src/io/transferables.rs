use geozero::mvt::{tile, tile::Layer};

use crate::{
    coords::WorldTileCoords,
    io::tile_repository::StoredLayer,
    render::ShaderVertex,
    tessellation::{IndexDataType, OverAlignedVertexBuffer},
};

pub trait TileTessellated: Send {
    fn new(coords: WorldTileCoords) -> Self;

    fn coords(&self) -> &WorldTileCoords;
}

pub trait UnavailableLayer: Send {
    fn new(coords: WorldTileCoords, layer_name: String) -> Self;

    fn to_stored_layer(self) -> StoredLayer;
}

pub trait TessellatedLayer: Send {
    fn new(
        coords: WorldTileCoords,
        buffer: OverAlignedVertexBuffer<ShaderVertex, IndexDataType>,
        feature_indices: Vec<u32>,
        layer_data: tile::Layer,
    ) -> Self;
    fn to_stored_layer(self) -> StoredLayer;
}

pub struct DefaultTileTessellated {
    pub coords: WorldTileCoords,
}

impl TileTessellated for DefaultTileTessellated {
    fn new(coords: WorldTileCoords) -> Self {
        Self { coords }
    }

    fn coords(&self) -> &WorldTileCoords {
        &self.coords
    }
}

pub struct DefaultUnavailableLayer {
    pub coords: WorldTileCoords,
    pub layer_name: String,
}

impl UnavailableLayer for DefaultUnavailableLayer {
    fn new(coords: WorldTileCoords, layer_name: String) -> Self {
        Self { coords, layer_name }
    }

    fn to_stored_layer(self) -> StoredLayer {
        StoredLayer::UnavailableLayer {
            coords: self.coords,
            layer_name: self.layer_name,
        }
    }
}

pub struct DefaultTessellatedLayer {
    pub coords: WorldTileCoords,
    pub buffer: OverAlignedVertexBuffer<ShaderVertex, IndexDataType>,
    /// Holds for each feature the count of indices.
    pub feature_indices: Vec<u32>,
    pub layer_data: Layer, // FIXME (perf): Introduce a better structure for this
}

impl TessellatedLayer for DefaultTessellatedLayer {
    fn new(
        coords: WorldTileCoords,
        buffer: OverAlignedVertexBuffer<ShaderVertex, IndexDataType>,
        feature_indices: Vec<u32>,
        layer_data: Layer,
    ) -> Self {
        Self {
            coords,
            buffer,
            feature_indices,
            layer_data,
        }
    }

    fn to_stored_layer(self) -> StoredLayer {
        StoredLayer::TessellatedLayer {
            coords: self.coords,
            layer_name: self.layer_data.name,
            buffer: self.buffer,
            feature_indices: self.feature_indices,
        }
    }
}

pub trait Transferables: 'static {
    type TileTessellated: TileTessellated;
    type UnavailableLayer: UnavailableLayer;
    type TessellatedLayer: TessellatedLayer;
}

#[derive(Copy, Clone)]
pub struct DefaultTransferables;

impl Transferables for DefaultTransferables {
    type TileTessellated = DefaultTileTessellated;
    type UnavailableLayer = DefaultUnavailableLayer;
    type TessellatedLayer = DefaultTessellatedLayer;
}
