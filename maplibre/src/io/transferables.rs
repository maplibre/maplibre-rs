use geozero::mvt::tile::Layer;

use crate::{
    coords::WorldTileCoords,
    io::{geometry_index::TileIndex, tile_repository::StoredLayer},
    render::ShaderVertex,
    tessellation::{IndexDataType, OverAlignedVertexBuffer},
};

pub trait TileTessellated: Send {
    fn build_from(coords: WorldTileCoords) -> Self
    where
        Self: Sized;

    fn coords(&self) -> WorldTileCoords;
}

pub trait LayerUnavailable: Send {
    fn build_from(coords: WorldTileCoords, layer_name: String) -> Self
    where
        Self: Sized;

    fn coords(&self) -> WorldTileCoords;
    fn layer_name(&self) -> &str;

    fn to_stored_layer(self) -> StoredLayer;
}

pub trait LayerTessellated: Send {
    fn build_from(
        coords: WorldTileCoords,
        buffer: OverAlignedVertexBuffer<ShaderVertex, IndexDataType>,
        feature_indices: Vec<u32>,
        layer_data: Layer,
    ) -> Self
    where
        Self: Sized;

    fn coords(&self) -> WorldTileCoords;

    fn to_stored_layer(self) -> StoredLayer;
}

pub trait LayerIndexed: Send {
    fn build_from(coords: WorldTileCoords, index: TileIndex) -> Self
    where
        Self: Sized;

    fn coords(&self) -> WorldTileCoords;

    fn to_tile_index(self) -> TileIndex;
}

pub struct DefaultTileTessellated {
    pub coords: WorldTileCoords,
}

impl TileTessellated for DefaultTileTessellated {
    fn build_from(coords: WorldTileCoords) -> Self {
        Self { coords }
    }

    fn coords(&self) -> WorldTileCoords {
        self.coords
    }
}

pub struct DefaultLayerUnavailable {
    pub coords: WorldTileCoords,
    pub layer_name: String,
}

impl LayerUnavailable for DefaultLayerUnavailable {
    fn build_from(coords: WorldTileCoords, layer_name: String) -> Self {
        Self { coords, layer_name }
    }

    fn coords(&self) -> WorldTileCoords {
        self.coords
    }

    fn layer_name(&self) -> &str {
        &self.layer_name
    }

    fn to_stored_layer(self) -> StoredLayer {
        StoredLayer::UnavailableLayer {
            coords: self.coords,
            layer_name: self.layer_name,
        }
    }
}

#[derive(Clone)]
pub struct DefaultLayerTesselated {
    pub coords: WorldTileCoords,
    pub buffer: OverAlignedVertexBuffer<ShaderVertex, IndexDataType>,
    /// Holds for each feature the count of indices.
    pub feature_indices: Vec<u32>,
    pub layer_data: Layer, // FIXME (perf): Introduce a better structure for this
}

impl LayerTessellated for DefaultLayerTesselated {
    fn build_from(
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

    fn coords(&self) -> WorldTileCoords {
        self.coords
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

impl LayerIndexed for DefaultLayerIndexed {
    fn build_from(coords: WorldTileCoords, index: TileIndex) -> Self {
        Self { coords, index }
    }

    fn coords(&self) -> WorldTileCoords {
        self.coords
    }

    fn to_tile_index(self) -> TileIndex {
        self.index
    }
}

pub trait Transferables: 'static {
    type TileTessellated: TileTessellated;
    type LayerUnavailable: LayerUnavailable;
    type LayerTessellated: LayerTessellated;
    type LayerIndexed: LayerIndexed;
}

#[derive(Copy, Clone)]
pub struct DefaultTransferables;

impl Transferables for DefaultTransferables {
    type TileTessellated = DefaultTileTessellated;
    type LayerUnavailable = DefaultLayerUnavailable;
    type LayerTessellated = DefaultLayerTesselated;
    type LayerIndexed = DefaultLayerIndexed;
}
