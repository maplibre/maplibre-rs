use std::fmt::{Debug, Formatter};

use geozero::mvt::tile::Layer;

use crate::{
    coords::WorldTileCoords,
    io::{
        apc::{IntoMessage, Message},
        geometry_index::TileIndex,
    },
    render::ShaderVertex,
    tessellation::{IndexDataType, OverAlignedVertexBuffer},
    vector::{AvailableVectorLayerData, UnavailableVectorLayerData},
};

pub trait TileTessellated: IntoMessage + Debug + Send {
    fn build_from(coords: WorldTileCoords) -> Self
    where
        Self: Sized;

    fn coords(&self) -> WorldTileCoords;
}

pub trait LayerUnavailable: IntoMessage + Debug + Send {
    fn build_from(coords: WorldTileCoords, layer_name: String) -> Self
    where
        Self: Sized;

    fn coords(&self) -> WorldTileCoords;
    fn layer_name(&self) -> &str;

    fn to_layer(self) -> UnavailableVectorLayerData;
}

pub trait LayerTessellated: IntoMessage + Debug + Send {
    fn build_from(
        coords: WorldTileCoords,
        buffer: OverAlignedVertexBuffer<ShaderVertex, IndexDataType>,
        feature_indices: Vec<u32>,
        layer_data: Layer,
    ) -> Self
    where
        Self: Sized;

    fn coords(&self) -> WorldTileCoords;

    fn is_empty(&self) -> bool;

    fn to_layer(self) -> AvailableVectorLayerData;
}

pub trait LayerIndexed: IntoMessage + Debug + Send {
    fn build_from(coords: WorldTileCoords, index: TileIndex) -> Self
    where
        Self: Sized;

    fn coords(&self) -> WorldTileCoords;

    fn to_tile_index(self) -> TileIndex;
}

pub struct DefaultTileTessellated {
    pub coords: WorldTileCoords,
}

impl Debug for DefaultTileTessellated {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "DefaultTileTessellated({})", self.coords)
    }
}

impl IntoMessage for DefaultTileTessellated {
    fn into(self) -> Message {
        Message {
            transferable: Box::new(self),
        }
    }
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

impl Debug for DefaultLayerUnavailable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "DefaultLayerUnavailable({})", self.coords)
    }
}

impl IntoMessage for DefaultLayerUnavailable {
    fn into(self) -> Message {
        Message {
            transferable: Box::new(self),
        }
    }
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

    fn to_layer(self) -> UnavailableVectorLayerData {
        UnavailableVectorLayerData {
            coords: self.coords,
            source_layer: self.layer_name,
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

impl Debug for DefaultLayerTesselated {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "DefaultLayerTesselated({})", self.coords)
    }
}

impl IntoMessage for DefaultLayerTesselated {
    fn into(self) -> Message {
        Message {
            transferable: Box::new(self),
        }
    }
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

    fn is_empty(&self) -> bool {
        self.buffer.usable_indices == 0
    }

    fn to_layer(self) -> AvailableVectorLayerData {
        AvailableVectorLayerData {
            coords: self.coords,
            source_layer: self.layer_data.name,
            buffer: self.buffer,
            feature_indices: self.feature_indices,
        }
    }
}

pub struct DefaultLayerIndexed {
    coords: WorldTileCoords,
    index: TileIndex,
}

impl Debug for DefaultLayerIndexed {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "DefaultLayerIndexed({})", self.coords)
    }
}

impl IntoMessage for DefaultLayerIndexed {
    fn into(self) -> Message {
        Message {
            transferable: Box::new(self),
        }
    }
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

pub trait Transferables: Copy + Clone + 'static {
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
