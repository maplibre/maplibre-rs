use maplibre::{
    benchmarking::tessellation::{IndexDataType, OverAlignedVertexBuffer},
    coords::{WorldTileCoords, ZoomLevel},
    io::{
        geometry_index::TileIndex,
        tile_repository::StoredLayer,
        transferables::{
            IndexedLayer, TessellatedLayer, TileTessellated, Transferables, UnavailableLayer,
        },
    },
    render::ShaderVertex,
    tile::Layer,
};
use transferable_memory_derive::MemoryTransferable;

#[derive(MemoryTransferable, Clone, Copy)]
pub struct TransferableWorldTileCoords {
    pub x: i32,
    pub y: i32,
    pub z: u8,
    pub padding1: u8,
    pub padding2: u8,
    pub padding3: u8,
}

impl From<WorldTileCoords> for TransferableWorldTileCoords {
    fn from(coords: WorldTileCoords) -> Self {
        Self {
            x: coords.x,
            y: coords.y,
            z: coords.z.into(),
            padding1: 0,
            padding2: 0,
            padding3: 0,
        }
    }
}

impl Into<WorldTileCoords> for TransferableWorldTileCoords {
    fn into(self) -> WorldTileCoords {
        WorldTileCoords {
            x: self.x,
            y: self.y,
            z: ZoomLevel::new(self.z),
        }
    }
}

#[derive(MemoryTransferable, Copy, Clone)]
pub struct LinearTileTessellated {
    pub coords: TransferableWorldTileCoords,
}

impl TileTessellated for LinearTileTessellated {
    fn new(coords: WorldTileCoords) -> Self {
        Self {
            coords: coords.into(),
        }
    }

    fn coords(&self) -> WorldTileCoords {
        self.coords.into()
    }
}

#[derive(MemoryTransferable, Copy, Clone)]
pub struct LinearLayerUnavailable {
    pub coords: TransferableWorldTileCoords,
    pub layer_name: [u8; 32],
}

impl UnavailableLayer for LinearLayerUnavailable {
    fn new(coords: WorldTileCoords, layer_name: String) -> Self {
        let mut new_layer_name = [0; 32];
        new_layer_name[0..layer_name.len()].clone_from_slice(layer_name.as_bytes());
        Self {
            coords: coords.into(),
            layer_name: new_layer_name,
        }
    }

    fn coords(&self) -> WorldTileCoords {
        self.coords.into()
    }

    fn to_stored_layer(self) -> StoredLayer {
        StoredLayer::UnavailableLayer {
            coords: self.coords.into(),
            layer_name: String::from_utf8(Vec::from(self.layer_name)).unwrap(), // FIXME (wasm-executor): Remove unwrap
        }
    }
}

// TODO: Missing
#[derive(MemoryTransferable, Copy, Clone)]
pub struct TessellationData<const I: usize, const V: usize, const F: usize> {
    pub size: u8,
    pub coords: TransferableWorldTileCoords,
    pub layer_name: [u8; 32],
    pub layer_name_len: usize,
    pub vertices: [ShaderVertex; V],
    pub vertices_len: usize,
    pub indices: [IndexDataType; I],
    pub indices_len: usize,
    pub usable_indices: u32,
    /// Holds for each feature the count of indices.
    pub feature_indices: [u32; F],
    pub feature_indices_len: usize,
}

impl<const I: usize, const V: usize, const F: usize> TessellationData<I, V, F> {
    pub fn new(
        coords: WorldTileCoords,
        buffer: OverAlignedVertexBuffer<ShaderVertex, IndexDataType>,
        feature_indices: Vec<u32>,
        layer_data: Layer,
    ) -> Box<TessellationData<I, V, F>> {
        let mut uninit = Box::<Self>::new_zeroed();
        let mut data: Box<TessellationData<I, V, F>> = unsafe { uninit.assume_init() };

        let vertices_len = buffer.buffer.vertices.len();
        let indices_len = buffer.buffer.indices.len();
        let features_len = feature_indices.len();
        if vertices_len >= V || indices_len >= I || features_len >= F {
            panic!(
                "Unsupported tessellated layer size: I: {} V: {} F: {}",
                indices_len, vertices_len, features_len
            )
        }

        data.coords = coords.into();

        data.usable_indices = buffer.usable_indices;

        data.vertices_len = vertices_len;
        data.vertices[0..data.vertices_len].clone_from_slice(&buffer.buffer.vertices);

        data.indices_len = indices_len;
        data.indices[0..data.indices_len].clone_from_slice(&buffer.buffer.indices);

        data.feature_indices_len = features_len;
        data.feature_indices[0..data.feature_indices_len].clone_from_slice(&feature_indices);

        data.layer_name_len = layer_data.name.len();
        data.layer_name[0..data.layer_name_len].clone_from_slice(layer_data.name.as_bytes());

        data
    }

    fn to_stored_layer(self) -> StoredLayer {
        // TODO: Avoid copies here
        StoredLayer::TessellatedLayer {
            coords: self.coords.into(),
            layer_name: String::from_utf8(Vec::from(&self.layer_name[..self.layer_name_len]))
                .unwrap(), // FIXME (wasm-executor): Remove unwrap
            buffer: OverAlignedVertexBuffer::from_slices(
                &self.vertices[..self.vertices_len],
                &self.indices[..self.indices_len],
                self.usable_indices,
            ),
            feature_indices: Vec::from(&self.feature_indices[..self.feature_indices_len]),
        }
    }

    fn coords(&self) -> WorldTileCoords {
        self.coords.into()
    }
}

pub type LargeTesselationData = TessellationData<50000, 40000, 2048>;

#[derive(Clone)]
pub enum VariableTessellationData {
    Large(Box<LargeTesselationData>),
}

impl VariableTessellationData {
    fn coords(&self) -> WorldTileCoords {
        match self {
            VariableTessellationData::Large(data) => data.coords(),
        }
    }

    fn to_stored_layer(self) -> StoredLayer {
        match self {
            VariableTessellationData::Large(data) => data.to_stored_layer(),
        }
    }
}

#[derive(Clone)]
pub struct LinearLayerTessellated {
    pub data: VariableTessellationData,
}

impl TessellatedLayer for LinearLayerTessellated {
    fn new(
        coords: WorldTileCoords,
        buffer: OverAlignedVertexBuffer<ShaderVertex, IndexDataType>,
        feature_indices: Vec<u32>,
        layer_data: Layer,
    ) -> Self {
        Self {
            data: match buffer.buffer.vertices.len() {
                n if n <= 30000 => {
                    let mut data =
                        TessellationData::new(coords, buffer, feature_indices, layer_data);
                    data.size = 3;
                    VariableTessellationData::Large(data)
                }
                _ => {
                    panic!("Unsupported tesselated layer size")
                }
            },
        }
    }

    fn coords(&self) -> WorldTileCoords {
        self.data.coords()
    }

    fn to_stored_layer(self) -> StoredLayer {
        self.data.to_stored_layer()
    }
}

#[derive(MemoryTransferable, Copy, Clone)]
pub struct LinearLayerIndexed {
    pub coords: TransferableWorldTileCoords,
}

impl From<(WorldTileCoords, TileIndex)> for LinearLayerIndexed {
    fn from((coords, _index): (WorldTileCoords, TileIndex)) -> Self {
        Self {
            coords: coords.into(),
        }
    }
}

impl IndexedLayer for LinearLayerIndexed {
    fn coords(&self) -> WorldTileCoords {
        self.coords.into()
    }

    fn to_tile_index(self) -> TileIndex {
        // FIXME replace this stub implementation
        TileIndex::Linear { list: vec![] }
    }
}

#[derive(MemoryTransferable, Copy, Clone)]
pub struct LinearTransferables;

impl Transferables for LinearTransferables {
    type TileTessellated = LinearTileTessellated;
    type LayerUnavailable = LinearLayerUnavailable;
    type LayerTessellated = LinearLayerTessellated;
    type LayerIndexed = LinearLayerIndexed;
}
