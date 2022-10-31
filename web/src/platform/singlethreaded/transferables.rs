use bytemuck::{TransparentWrapper, Zeroable};
use bytemuck_derive::{Pod, Zeroable};
use log::warn;
use maplibre::{
    benchmarking::tessellation::{IndexDataType, OverAlignedVertexBuffer},
    coords::WorldTileCoords,
    io::{
        tile_repository::StoredLayer,
        transferables::{TessellatedLayer, TileTessellated, Transferables, UnavailableLayer},
    },
    render::ShaderVertex,
    tile::Layer,
};

// FIXME (wasm-executor): properly do this!, fix this whole file
#[derive(Copy, Clone, Debug)]
#[repr(transparent)]
pub struct WrapperWorldTileCoords(WorldTileCoords);
unsafe impl TransparentWrapper<WorldTileCoords> for WrapperWorldTileCoords {}
unsafe impl bytemuck::Zeroable for WrapperWorldTileCoords {}
unsafe impl bytemuck::Pod for WrapperWorldTileCoords {}

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct LongVertexShader([ShaderVertex; 15000]);
unsafe impl TransparentWrapper<[ShaderVertex; 15000]> for LongVertexShader {}
unsafe impl bytemuck::Zeroable for LongVertexShader {}
unsafe impl bytemuck::Pod for LongVertexShader {}

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct LongIndices([IndexDataType; 40000]);
unsafe impl TransparentWrapper<[IndexDataType; 40000]> for LongIndices {}
unsafe impl bytemuck::Zeroable for LongIndices {}
unsafe impl bytemuck::Pod for LongIndices {}

#[derive(Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct LinearTileTessellated {
    pub coords: WrapperWorldTileCoords,
}

impl TileTessellated for LinearTileTessellated {
    fn new(coords: WorldTileCoords) -> Self {
        Self {
            coords: WrapperWorldTileCoords::wrap(coords),
        }
    }

    fn coords(&self) -> &WorldTileCoords {
        WrapperWorldTileCoords::peel_ref(&self.coords)
    }
}

#[derive(Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct LinearUnavailableLayer {
    pub coords: WrapperWorldTileCoords,
    pub layer_name: [u8; 32],
}

impl UnavailableLayer for LinearUnavailableLayer {
    fn new(coords: WorldTileCoords, layer_name: String) -> Self {
        let mut new_layer_name = [0; 32];
        new_layer_name[0..layer_name.len()].clone_from_slice(layer_name.as_bytes());
        Self {
            coords: WrapperWorldTileCoords::wrap(coords),
            layer_name: new_layer_name,
        }
    }

    fn to_stored_layer(self) -> StoredLayer {
        StoredLayer::UnavailableLayer {
            coords: WrapperWorldTileCoords::peel(self.coords),
            layer_name: String::from_utf8(Vec::from(self.layer_name)).unwrap(), // FIXME (wasm-executor): Remove unwrap
        }
    }
}

#[derive(Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct InnerData {
    pub coords: WrapperWorldTileCoords,
    pub layer_name: [u8; 32],
    pub layer_name_len: usize,
    pub vertices: LongVertexShader,
    pub vertices_len: usize,
    pub indices: LongIndices,
    pub indices_len: usize,
    pub usable_indices: u32,
    /// Holds for each feature the count of indices.
    pub feature_indices: [u32; 2048],
    pub feature_indices_len: usize,
}

#[derive(Clone)]
pub struct LinearTessellatedLayer {
    pub data: Box<InnerData>,
}

impl TessellatedLayer for LinearTessellatedLayer {
    fn new(
        coords: WorldTileCoords,
        buffer: OverAlignedVertexBuffer<ShaderVertex, IndexDataType>,
        feature_indices: Vec<u32>,
        layer_data: Layer,
    ) -> Self {
        let mut data = Box::new(InnerData {
            coords: WrapperWorldTileCoords::wrap(coords),

            layer_name: [0; 32],
            layer_name_len: layer_data.name.len(),

            vertices: LongVertexShader::wrap([ShaderVertex::zeroed(); 15000]),
            vertices_len: buffer.buffer.vertices.len(),

            indices: LongIndices::wrap([IndexDataType::zeroed(); 40000]),
            indices_len: buffer.buffer.indices.len(),

            usable_indices: buffer.usable_indices,

            feature_indices: [0u32; 2048],
            feature_indices_len: feature_indices.len(),
        });

        if buffer.buffer.vertices.len() > 15000 {
            warn!("vertices too large");
            return Self {
                data: Box::new(InnerData {
                    coords: WrapperWorldTileCoords::wrap(coords),

                    layer_name: [0; 32],
                    layer_name_len: 0,

                    vertices: LongVertexShader::wrap([ShaderVertex::zeroed(); 15000]),
                    vertices_len: 0,

                    indices: LongIndices::wrap([IndexDataType::zeroed(); 40000]),
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
                    coords: WrapperWorldTileCoords::wrap(coords),

                    layer_name: [0; 32],
                    layer_name_len: 0,

                    vertices: LongVertexShader::wrap([ShaderVertex::zeroed(); 15000]),
                    vertices_len: 0,

                    indices: LongIndices::wrap([IndexDataType::zeroed(); 40000]),
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
                    coords: WrapperWorldTileCoords::wrap(coords),

                    layer_name: [0; 32],
                    layer_name_len: 0,

                    vertices: LongVertexShader::wrap([ShaderVertex::zeroed(); 15000]),
                    vertices_len: 0,

                    indices: LongIndices::wrap([IndexDataType::zeroed(); 40000]),
                    indices_len: 0,

                    usable_indices: 0,

                    feature_indices: [0u32; 2048],
                    feature_indices_len: 0,
                }),
            };
        }

        data.vertices.0[0..buffer.buffer.vertices.len()].clone_from_slice(&buffer.buffer.vertices);
        data.indices.0[0..buffer.buffer.indices.len()].clone_from_slice(&buffer.buffer.indices);
        data.feature_indices[0..feature_indices.len()].clone_from_slice(&feature_indices);
        data.layer_name[0..layer_data.name.len()].clone_from_slice(layer_data.name.as_bytes());

        Self { data }
    }

    fn to_stored_layer(self) -> StoredLayer {
        // TODO: Avoid copies here
        StoredLayer::TessellatedLayer {
            coords: WrapperWorldTileCoords::peel(self.data.coords),
            layer_name: String::from_utf8(Vec::from(
                &self.data.layer_name[..self.data.layer_name_len],
            ))
            .unwrap(), // FIXME (wasm-executor): Remove unwrap
            buffer: OverAlignedVertexBuffer::from_slices(
                &self.data.vertices.0[..self.data.vertices_len],
                &self.data.indices.0[..self.data.indices_len],
                self.data.usable_indices,
            ),
            feature_indices: Vec::from(&self.data.feature_indices[..self.data.feature_indices_len]),
        }
    }
}

pub struct LinearTransferables;

impl Transferables for LinearTransferables {
    type TileTessellated = LinearTileTessellated;
    type UnavailableLayer = LinearUnavailableLayer;
    type TessellatedLayer = LinearTessellatedLayer;
}
