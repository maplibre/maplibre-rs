use log::warn;
use maplibre::{
    benchmarking::tessellation::{IndexDataType, OverAlignedVertexBuffer},
    coords::WorldTileCoords,
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

#[derive(Copy, Clone)]
pub struct LinearTileTessellated {
    pub coords: WorldTileCoords,
    pub _padding: u8,
}

impl TileTessellated for LinearTileTessellated {
    fn new(coords: WorldTileCoords) -> Self {
        Self {
            coords,
            _padding: 0,
        }
    }

    fn coords(&self) -> &WorldTileCoords {
        &self.coords
    }
}

#[derive(Copy, Clone)]
pub struct LinearLayerUnavailable {
    pub coords: WorldTileCoords,
    pub layer_name: [u8; 32],
}

impl UnavailableLayer for LinearLayerUnavailable {
    fn new(coords: WorldTileCoords, layer_name: String) -> Self {
        let mut new_layer_name = [0; 32];
        new_layer_name[0..layer_name.len()].clone_from_slice(layer_name.as_bytes());
        Self {
            coords,
            layer_name: new_layer_name,
        }
    }

    fn coords(&self) -> &WorldTileCoords {
        &self.coords
    }

    fn to_stored_layer(self) -> StoredLayer {
        StoredLayer::UnavailableLayer {
            coords: self.coords,
            layer_name: String::from_utf8(Vec::from(self.layer_name)).unwrap(), // FIXME (wasm-executor): Remove unwrap
        }
    }
}

#[derive(Copy, Clone)]
pub struct InnerData {
    pub coords: WorldTileCoords,
    pub layer_name: [u8; 32],
    pub layer_name_len: usize,
    pub vertices: [ShaderVertex; 15000],
    pub vertices_len: usize,
    pub indices: [IndexDataType; 40000],
    pub indices_len: usize,
    pub usable_indices: u32,
    /// Holds for each feature the count of indices.
    pub feature_indices: [u32; 2048],
    pub feature_indices_len: usize,
}

#[derive(Clone)]
pub struct LinearLayerTesselated {
    pub data: Box<InnerData>,
}

impl TessellatedLayer for LinearLayerTesselated {
    fn new(
        coords: WorldTileCoords,
        buffer: OverAlignedVertexBuffer<ShaderVertex, IndexDataType>,
        feature_indices: Vec<u32>,
        layer_data: Layer,
    ) -> Self {
        let mut data = Box::new(InnerData {
            coords,

            layer_name: [0; 32],
            layer_name_len: layer_data.name.len(),

            vertices: [ShaderVertex::new([0.0, 0.0], [0.0, 0.0]); 15000],
            vertices_len: buffer.buffer.vertices.len(),

            indices: [0; 40000],
            indices_len: buffer.buffer.indices.len(),

            usable_indices: buffer.usable_indices,

            feature_indices: [0u32; 2048],
            feature_indices_len: feature_indices.len(),
        });

        if buffer.buffer.vertices.len() > 15000 {
            warn!("vertices too large");
            return Self {
                data: Box::new(InnerData {
                    coords,

                    layer_name: [0; 32],
                    layer_name_len: 0,

                    vertices: [ShaderVertex::new([0.0, 0.0], [0.0, 0.0]); 15000],
                    vertices_len: 0,

                    indices: [0; 40000],
                    indices_len: 0,

                    usable_indices: 0,

                    feature_indices: [0u32; 2048],
                    feature_indices_len: 0,
                }),
            };
        }

        if buffer.buffer.indices.len() > 40000 {
            warn!("indices too large");
            return Self {
                data: Box::new(InnerData {
                    coords,

                    layer_name: [0; 32],
                    layer_name_len: 0,

                    vertices: [ShaderVertex::new([0.0, 0.0], [0.0, 0.0]); 15000],
                    vertices_len: 0,

                    indices: [0; 40000],
                    indices_len: 0,

                    usable_indices: 0,

                    feature_indices: [0u32; 2048],
                    feature_indices_len: 0,
                }),
            };
        }

        if feature_indices.len() > 2048 {
            warn!("feature_indices too large");
            return Self {
                data: Box::new(InnerData {
                    coords,

                    layer_name: [0; 32],
                    layer_name_len: 0,

                    vertices: [ShaderVertex::new([0.0, 0.0], [0.0, 0.0]); 15000],
                    vertices_len: 0,

                    indices: [0; 40000],
                    indices_len: 0,

                    usable_indices: 0,

                    feature_indices: [0u32; 2048],
                    feature_indices_len: 0,
                }),
            };
        }

        data.vertices[0..buffer.buffer.vertices.len()].clone_from_slice(&buffer.buffer.vertices);
        data.indices[0..buffer.buffer.indices.len()].clone_from_slice(&buffer.buffer.indices);
        data.feature_indices[0..feature_indices.len()].clone_from_slice(&feature_indices);
        data.layer_name[0..layer_data.name.len()].clone_from_slice(layer_data.name.as_bytes());

        Self { data }
    }

    fn coords(&self) -> &WorldTileCoords {
        &self.data.coords
    }

    fn to_stored_layer(self) -> StoredLayer {
        // TODO: Avoid copies here
        StoredLayer::TessellatedLayer {
            coords: self.data.coords,
            layer_name: String::from_utf8(Vec::from(
                &self.data.layer_name[..self.data.layer_name_len],
            ))
            .unwrap(), // FIXME (wasm-executor): Remove unwrap
            buffer: OverAlignedVertexBuffer::from_slices(
                &self.data.vertices[..self.data.vertices_len],
                &self.data.indices[..self.data.indices_len],
                self.data.usable_indices,
            ),
            feature_indices: Vec::from(&self.data.feature_indices[..self.data.feature_indices_len]),
        }
    }
}

#[derive(Copy, Clone)]
pub struct LinearLayerIndexed {
    pub coords: WorldTileCoords,
}

impl From<(WorldTileCoords, TileIndex)> for LinearLayerIndexed {
    fn from((coords, _index): (WorldTileCoords, TileIndex)) -> Self {
        Self { coords }
    }
}

impl IndexedLayer for LinearLayerIndexed {
    fn coords(&self) -> &WorldTileCoords {
        &self.coords
    }

    fn to_tile_index(self) -> TileIndex {
        // FIXME replace this stub implementation
        TileIndex::Linear { list: vec![] }
    }
}

pub struct LinearTransferables;

impl Transferables for LinearTransferables {
    type TileTessellated = LinearTileTessellated;
    type LayerUnavailable = LinearLayerUnavailable;
    type LayerTessellated = LinearLayerTesselated;
    type LayerIndexed = LinearLayerIndexed;
}
