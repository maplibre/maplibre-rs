use geozero::mvt::tile::Layer;

use crate::{
    coords::WorldTileCoords,
    io::{geometry_index::TileIndex, tile_repository::StoredLayer},
    render::ShaderVertex,
    tessellation::{IndexDataType, OverAlignedVertexBuffer},
};

pub trait TileTessellated: Send {
    fn new(coords: WorldTileCoords) -> Self;

    fn coords(&self) -> &WorldTileCoords;
}

pub trait UnavailableLayer: Send {
    fn new(coords: WorldTileCoords, layer_name: String) -> Self;

    fn coords(&self) -> &WorldTileCoords;

    fn to_stored_layer(self) -> StoredLayer;
}

pub trait TessellatedLayer: Send {
    fn new(
        coords: WorldTileCoords,
        buffer: OverAlignedVertexBuffer<ShaderVertex, IndexDataType>,
        feature_indices: Vec<u32>,
        layer_data: Layer,
    ) -> Self;

    fn coords(&self) -> &WorldTileCoords;

    fn to_stored_layer(self) -> StoredLayer;
}

pub trait IndexedLayer: Send + From<(WorldTileCoords, TileIndex)> {
    fn coords(&self) -> &WorldTileCoords;

    fn to_tile_index(self) -> TileIndex;
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

pub struct DefaultLayerUnavailable {
    pub coords: WorldTileCoords,
    pub layer_name: String,
}

impl UnavailableLayer for DefaultLayerUnavailable {
    fn new(coords: WorldTileCoords, layer_name: String) -> Self {
        Self { coords, layer_name }
    }

    fn coords(&self) -> &WorldTileCoords {
        &self.coords
    }

    fn to_stored_layer(self) -> StoredLayer {
        StoredLayer::UnavailableLayer {
            coords: self.coords,
            layer_name: self.layer_name,
        }
    }
}

pub struct DefaultLayerTesselated {
    pub coords: WorldTileCoords,
    pub buffer: OverAlignedVertexBuffer<ShaderVertex, IndexDataType>,
    /// Holds for each feature the count of indices.
    pub feature_indices: Vec<u32>,
    pub layer_data: Layer, // FIXME (perf): Introduce a better structure for this
}

impl TessellatedLayer for DefaultLayerTesselated {
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

    fn coords(&self) -> &WorldTileCoords {
        &self.coords
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

pub struct DefaultLayerIndexed {
    coords: WorldTileCoords,
    index: TileIndex,
}

impl From<(WorldTileCoords, TileIndex)> for DefaultLayerIndexed {
    fn from((coords, index): (WorldTileCoords, TileIndex)) -> Self {
        Self { coords, index }
    }
}

impl IndexedLayer for DefaultLayerIndexed {
    fn coords(&self) -> &WorldTileCoords {
        &self.coords
    }

    fn to_tile_index(self) -> TileIndex {
        self.index
    }
}

pub trait Transferables: 'static {
    type TileTessellated: TileTessellated;
    type LayerUnavailable: UnavailableLayer;
    type LayerTessellated: TessellatedLayer;
    type LayerIndexed: IndexedLayer;
}

#[derive(Copy, Clone)]
pub struct DefaultTransferables;

impl Transferables for DefaultTransferables {
    type TileTessellated = DefaultTileTessellated;
    type LayerUnavailable = DefaultLayerUnavailable;
    type LayerTessellated = DefaultLayerTesselated;
    type LayerIndexed = DefaultLayerIndexed;
}
