use flatbuffers::FlatBufferBuilder;
use maplibre::{
    benchmarking::tessellation::{IndexDataType, OverAlignedVertexBuffer},
    coords::{WorldTileCoords, ZoomLevel},
    io::{
        geometry_index::TileIndex,
        tile_repository::StoredLayer,
        transferables::{
            LayerIndexed, LayerTessellated, LayerUnavailable, TileTessellated, Transferables,
        },
    },
    render::ShaderVertex,
    tile::Layer,
};

use crate::platform::singlethreaded::transferables::{
    basic_generated::*, layer_indexed_generated::*, layer_tessellated_generated::*,
    layer_unavailable_generated::*, tile_tessellated_generated::*,
};

pub mod basic_generated {
    #![allow(unused_imports)]
    include!(concat!(env!("OUT_DIR"), "/basic_generated.rs"));
}
pub mod layer_indexed_generated {
    #![allow(unused_imports)]
    include!(concat!(env!("OUT_DIR"), "/layer_indexed_generated.rs"));
}
pub mod layer_tessellated_generated {
    #![allow(unused_imports)]
    include!(concat!(env!("OUT_DIR"), "/layer_tessellated_generated.rs"));
}
pub mod layer_unavailable_generated {
    #![allow(unused_imports)]
    include!(concat!(env!("OUT_DIR"), "/layer_unavailable_generated.rs"));
}
pub mod tile_tessellated_generated {
    #![allow(unused_imports)]
    include!(concat!(env!("OUT_DIR"), "/tile_tessellated_generated.rs"));
}

pub struct FlatBufferTransferable {
    pub data: Vec<u8>,
    pub start: usize,
}

impl<'a, 'b> TileTessellated for FlatBufferTransferable {
    fn build_from(coords: WorldTileCoords) -> Self {
        let mut inner_builder = FlatBufferBuilder::with_capacity(1024);
        let mut builder = FlatTileTessellatedBuilder::new(&mut inner_builder);

        builder.add_coords(&FlatWorldTileCoords::new(
            coords.x,
            coords.y,
            coords.z.into(),
        ));
        let root = builder.finish();
        inner_builder.finish(root, None);
        let (data, start) = inner_builder.collapse();
        FlatBufferTransferable { data, start }
    }

    fn coords(&self) -> WorldTileCoords {
        let data = unsafe { root_as_flat_tile_tessellated_unchecked(&self.data[self.start..]) };
        WorldTileCoords {
            x: data.coords().unwrap().x(),
            y: data.coords().unwrap().y(),
            z: ZoomLevel::new(data.coords().unwrap().z()),
        }
    }
}

impl LayerUnavailable for FlatBufferTransferable {
    fn build_from(coords: WorldTileCoords, layer_name: String) -> Self {
        let mut inner_builder = FlatBufferBuilder::with_capacity(1024);
        let layer_name = inner_builder.create_string(&layer_name);

        let mut builder = FlatLayerUnavailableBuilder::new(&mut inner_builder);
        builder.add_coords(&FlatWorldTileCoords::new(
            coords.x,
            coords.y,
            coords.z.into(),
        ));
        builder.add_layer_name(layer_name);
        let root = builder.finish();

        inner_builder.finish(root, None);
        let (data, start) = inner_builder.collapse();
        FlatBufferTransferable { data, start }
    }

    fn coords(&self) -> WorldTileCoords {
        let data = unsafe { root_as_flat_layer_unavailable_unchecked(&self.data[self.start..]) };
        WorldTileCoords {
            x: data.coords().unwrap().x(),
            y: data.coords().unwrap().y(),
            z: ZoomLevel::new(data.coords().unwrap().z()),
        }
    }

    fn layer_name(&self) -> &str {
        let data = unsafe { root_as_flat_layer_unavailable_unchecked(&self.data[self.start..]) };
        data.layer_name().expect("property must be set")
    }

    fn to_stored_layer(self) -> StoredLayer {
        StoredLayer::UnavailableLayer {
            layer_name: self.layer_name().to_owned(),
            coords: LayerUnavailable::coords(&self),
        }
    }
}

impl LayerTessellated for FlatBufferTransferable {
    fn build_from(
        coords: WorldTileCoords,
        buffer: OverAlignedVertexBuffer<ShaderVertex, IndexDataType>,
        feature_indices: Vec<u32>,
        layer_data: Layer,
    ) -> Self {
        let mut inner_builder = FlatBufferBuilder::with_capacity(1024);

        let vertices = inner_builder.create_vector(
            &buffer
                .buffer
                .vertices
                .iter()
                .map(|vertex| FlatShaderVertex::new(&vertex.position, &vertex.normal))
                .collect::<Vec<_>>(),
        );
        let indices = inner_builder.create_vector(&buffer.buffer.indices);
        let feature_indices = inner_builder.create_vector(&feature_indices);
        let layer_name = inner_builder.create_string(&layer_data.name);

        let mut builder = FlatLayerTessellatedBuilder::new(&mut inner_builder);

        builder.add_coords(&FlatWorldTileCoords::new(
            coords.x,
            coords.y,
            coords.z.into(),
        ));
        builder.add_layer_name(layer_name);
        builder.add_vertices(vertices);
        builder.add_indices(indices);
        builder.add_feature_indices(feature_indices);
        builder.add_usable_indices(buffer.usable_indices);
        let root = builder.finish();

        inner_builder.finish(root, None);
        let (data, start) = inner_builder.collapse();
        FlatBufferTransferable { data, start }
    }

    fn coords(&self) -> WorldTileCoords {
        let data = unsafe { root_as_flat_layer_tessellated_unchecked(&self.data[self.start..]) };
        WorldTileCoords {
            x: data.coords().unwrap().x(),
            y: data.coords().unwrap().y(),
            z: ZoomLevel::new(data.coords().unwrap().z()),
        }
    }

    fn to_stored_layer(self) -> StoredLayer {
        let data = unsafe { root_as_flat_layer_tessellated_unchecked(&self.data[self.start..]) };
        let vertices = data
            .vertices()
            .unwrap()
            .iter()
            .map(|vertex| ShaderVertex::new(vertex.position().into(), vertex.normal().into()));

        let indices = data.indices().unwrap();
        let feature_indices: Vec<u32> = data.feature_indices().unwrap().iter().collect();
        let usable_indices = data.usable_indices();
        StoredLayer::TessellatedLayer {
            coords: LayerTessellated::coords(&self),
            layer_name: data.layer_name().unwrap().to_owned(),
            buffer: OverAlignedVertexBuffer::from_iters(vertices, indices, usable_indices),
            feature_indices,
        }
    }
}

impl LayerIndexed for FlatBufferTransferable {
    fn build_from(coords: WorldTileCoords, index: TileIndex) -> Self {
        let mut inner_builder = FlatBufferBuilder::with_capacity(1024);
        let mut builder = FlatLayerIndexedBuilder::new(&mut inner_builder);

        // TODO index

        builder.add_coords(&FlatWorldTileCoords::new(
            coords.x,
            coords.y,
            coords.z.into(),
        ));
        let root = builder.finish();
        inner_builder.finish(root, None);
        let (data, start) = inner_builder.collapse();
        FlatBufferTransferable { data, start }
    }

    fn coords(&self) -> WorldTileCoords {
        let data = unsafe { root_as_flat_layer_indexed_unchecked(&self.data[self.start..]) };
        WorldTileCoords {
            x: data.coords().unwrap().x(),
            y: data.coords().unwrap().y(),
            z: ZoomLevel::new(data.coords().unwrap().z()),
        }
    }

    fn to_tile_index(self) -> TileIndex {
        TileIndex::Linear { list: vec![] } // TODO
    }
}

#[derive(Copy, Clone)]
pub struct FlatTransferables;

impl Transferables for FlatTransferables {
    type TileTessellated = FlatBufferTransferable;
    type LayerUnavailable = FlatBufferTransferable;
    type LayerTessellated = FlatBufferTransferable;
    type LayerIndexed = FlatBufferTransferable;
}
