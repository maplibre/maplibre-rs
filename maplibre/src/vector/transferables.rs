use std::fmt::{Debug, Formatter};

use geozero::mvt::tile::Layer;

use crate::{
    coords::WorldTileCoords,
    io::{
        apc::{IntoMessage, Message, MessageTag},
        geometry_index::TileIndex,
    },
    render::ShaderVertex,
    tessellation::{IndexDataType, OverAlignedVertexBuffer},
    vector::{AvailableVectorLayerData, MissingVectorLayerData},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum VectorMessageTag {
    TileTessellated = 1,
    LayerMissing = 2,
    LayerTessellated = 3,
    LayerIndexed = 4,
}

impl MessageTag for VectorMessageTag {
    fn dyn_clone(&self) -> Box<dyn MessageTag> {
        Box::new(*self)
    }
}

pub trait TileTessellated: IntoMessage + Debug + Send {
    fn message_tag() -> &'static dyn MessageTag;

    fn build_from(coords: WorldTileCoords) -> Self
    where
        Self: Sized;

    fn coords(&self) -> WorldTileCoords;
}

pub trait LayerMissing: IntoMessage + Debug + Send {
    fn message_tag() -> &'static dyn MessageTag;

    fn build_from(coords: WorldTileCoords, layer_name: String) -> Self
    where
        Self: Sized;

    fn coords(&self) -> WorldTileCoords;

    fn layer_name(&self) -> &str;

    fn to_layer(self) -> MissingVectorLayerData;
}

pub trait LayerTessellated: IntoMessage + Debug + Send {
    fn message_tag() -> &'static dyn MessageTag;

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
    fn message_tag() -> &'static dyn MessageTag;

    fn build_from(coords: WorldTileCoords, index: TileIndex) -> Self
    where
        Self: Sized;

    fn coords(&self) -> WorldTileCoords;

    fn to_tile_index(self) -> TileIndex;
}

pub struct DefaultTileTessellated {
    coords: WorldTileCoords,
}

impl Debug for DefaultTileTessellated {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "DefaultTileTessellated({})", self.coords)
    }
}

impl IntoMessage for DefaultTileTessellated {
    fn into(self) -> Message {
        Message::new(Self::message_tag(), Box::new(self))
    }
}

impl TileTessellated for DefaultTileTessellated {
    fn message_tag() -> &'static dyn MessageTag {
        &VectorMessageTag::TileTessellated
    }

    fn build_from(coords: WorldTileCoords) -> Self {
        Self { coords }
    }

    fn coords(&self) -> WorldTileCoords {
        self.coords
    }
}

pub struct DefaultLayerMissing {
    pub coords: WorldTileCoords,
    pub layer_name: String,
}

impl Debug for DefaultLayerMissing {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "DefaultLayerMissing({})", self.coords)
    }
}

impl IntoMessage for DefaultLayerMissing {
    fn into(self) -> Message {
        Message::new(Self::message_tag(), Box::new(self))
    }
}

impl LayerMissing for DefaultLayerMissing {
    fn message_tag() -> &'static dyn MessageTag {
        &VectorMessageTag::LayerMissing
    }

    fn build_from(coords: WorldTileCoords, layer_name: String) -> Self {
        Self { coords, layer_name }
    }

    fn coords(&self) -> WorldTileCoords {
        self.coords
    }

    fn layer_name(&self) -> &str {
        &self.layer_name
    }

    fn to_layer(self) -> MissingVectorLayerData {
        MissingVectorLayerData {
            coords: self.coords,
            source_layer: self.layer_name,
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

impl Debug for DefaultLayerTesselated {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "DefaultLayerTesselated({})", self.coords)
    }
}

impl IntoMessage for DefaultLayerTesselated {
    fn into(self) -> Message {
        Message::new(Self::message_tag(), Box::new(self))
    }
}

impl LayerTessellated for DefaultLayerTesselated {
    fn message_tag() -> &'static dyn MessageTag {
        &VectorMessageTag::LayerTessellated
    }

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
        Message::new(Self::message_tag(), Box::new(self))
    }
}

impl LayerIndexed for DefaultLayerIndexed {
    fn message_tag() -> &'static dyn MessageTag {
        &VectorMessageTag::LayerIndexed
    }

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

pub trait VectorTransferables: Copy + Clone + 'static {
    type TileTessellated: TileTessellated;
    type LayerMissing: LayerMissing;
    type LayerTessellated: LayerTessellated;
    type LayerIndexed: LayerIndexed;
}

#[derive(Copy, Clone)]
pub struct DefaultVectorTransferables;

impl VectorTransferables for DefaultVectorTransferables {
    type TileTessellated = DefaultTileTessellated;
    type LayerMissing = DefaultLayerMissing;
    type LayerTessellated = DefaultLayerTesselated;
    type LayerIndexed = DefaultLayerIndexed;
}
