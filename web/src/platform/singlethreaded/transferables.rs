use std::fmt::{Debug, Formatter};

use flatbuffers::FlatBufferBuilder;
use image::RgbaImage;
use js_sys::{ArrayBuffer, Uint8Array};
use maplibre::{
    benchmarking::tessellation::{IndexDataType, OverAlignedVertexBuffer},
    coords::WorldTileCoords,
    io::{
        apc::{IntoMessage, Message, MessageTag},
        geometry_index::TileIndex,
    },
    raster::{
        AvailableRasterLayerData, LayerRaster, LayerRasterMissing, MissingRasterLayerData,
        RasterTransferables,
    },
    render::ShaderVertex,
    tile::Layer,
    vector::{
        AvailableVectorLayerData, LayerIndexed, LayerMissing, LayerTessellated,
        MissingVectorLayerData, TileTessellated, VectorTransferables,
    },
};

use crate::platform::singlethreaded::{
    apc::WebMessageTag,
    transferables::{
        basic_generated::*, layer_indexed_generated::*, layer_missing_generated::*,
        layer_raster_generated::*, layer_tessellated_generated::*, tile_tessellated_generated::*,
    },
};

pub mod basic_generated {
    #![allow(unused, unused_imports, clippy::all)]

    use maplibre::coords::{WorldTileCoords, ZoomLevel};

    include!(concat!(env!("OUT_DIR"), "/basic_generated.rs"));

    impl Into<WorldTileCoords> for &FlatWorldTileCoords {
        fn into(self) -> WorldTileCoords {
            WorldTileCoords {
                x: self.x(),
                y: self.y(),
                z: ZoomLevel::new(self.z()),
            }
        }
    }
}
pub mod layer_indexed_generated {
    #![allow(unused, unused_imports, clippy::all)]
    include!(concat!(env!("OUT_DIR"), "/layer_indexed_generated.rs"));
}
pub mod layer_tessellated_generated {
    #![allow(unused, unused_imports, clippy::all)]
    include!(concat!(env!("OUT_DIR"), "/layer_tessellated_generated.rs"));
}
pub mod layer_missing_generated {
    #![allow(unused, unused_imports, clippy::all)]
    include!(concat!(env!("OUT_DIR"), "/layer_missing_generated.rs"));
}
pub mod tile_tessellated_generated {
    #![allow(unused, unused_imports, clippy::all)]
    include!(concat!(env!("OUT_DIR"), "/tile_tessellated_generated.rs"));
}
pub mod layer_raster_generated {
    #![allow(unused, unused_imports, clippy::all)]
    include!(concat!(env!("OUT_DIR"), "/layer_raster_generated.rs"));
}

pub struct FlatBufferTransferable {
    tag: WebMessageTag,
    data: Vec<u8>,
    start: usize,
}

impl FlatBufferTransferable {
    pub fn from_array_buffer(tag: WebMessageTag, buffer: ArrayBuffer) -> Self {
        let buffer = Uint8Array::new(&buffer);

        FlatBufferTransferable {
            tag,
            data: buffer.to_vec(),
            start: 0,
        }
    }

    pub fn data(&self) -> &[u8] {
        &self.data[self.start..]
    }
}

impl TileTessellated for FlatBufferTransferable {
    fn message_tag() -> &'static dyn MessageTag {
        &WebMessageTag::TileTessellated
    }

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
        FlatBufferTransferable {
            tag: WebMessageTag::TileTessellated,
            data,
            start,
        }
    }

    fn coords(&self) -> WorldTileCoords {
        let data = root_as_flat_tile_tessellated(&self.data[self.start..]).unwrap();
        data.coords().unwrap().into()
    }
}

impl LayerMissing for FlatBufferTransferable {
    fn message_tag() -> &'static dyn MessageTag {
        &WebMessageTag::LayerMissing
    }

    fn build_from(coords: WorldTileCoords, layer_name: String) -> Self {
        let mut inner_builder = FlatBufferBuilder::with_capacity(1024);
        let layer_name = inner_builder.create_string(&layer_name);

        let mut builder = FlatLayerMissingBuilder::new(&mut inner_builder);
        builder.add_coords(&FlatWorldTileCoords::new(
            coords.x,
            coords.y,
            coords.z.into(),
        ));
        builder.add_layer_name(layer_name);
        let root = builder.finish();

        inner_builder.finish(root, None);
        let (data, start) = inner_builder.collapse();
        FlatBufferTransferable {
            tag: WebMessageTag::LayerMissing,
            data,
            start,
        }
    }

    fn coords(&self) -> WorldTileCoords {
        let data = root_as_flat_layer_missing(&self.data[self.start..]).unwrap();
        data.coords().unwrap().into()
    }

    fn layer_name(&self) -> &str {
        let data = root_as_flat_layer_missing(&self.data[self.start..]).unwrap();
        data.layer_name().expect("property must be set")
    }

    fn to_layer(self) -> MissingVectorLayerData {
        MissingVectorLayerData {
            source_layer: self.layer_name().to_owned(),
            coords: LayerMissing::coords(&self),
        }
    }
}

impl Debug for FlatBufferTransferable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "FlatBufferTransferable<{:?}>(??)", self.tag)
    }
}

impl IntoMessage for FlatBufferTransferable {
    fn into(self) -> Message {
        Message::new(self.tag.to_static(), Box::new(self))
    }
}

impl LayerTessellated for FlatBufferTransferable {
    fn message_tag() -> &'static dyn MessageTag {
        &WebMessageTag::LayerTessellated
    }

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
        FlatBufferTransferable {
            tag: WebMessageTag::LayerTessellated,
            data,
            start,
        }
    }

    fn coords(&self) -> WorldTileCoords {
        let data = root_as_flat_layer_tessellated(&self.data[self.start..]).unwrap();
        data.coords().unwrap().into()
    }

    fn is_empty(&self) -> bool {
        let data = root_as_flat_layer_tessellated(&self.data[self.start..]).unwrap();
        data.usable_indices() == 0
    }

    fn to_layer(self) -> AvailableVectorLayerData {
        let data = root_as_flat_layer_tessellated(&self.data[self.start..]).unwrap();
        let vertices = data
            .vertices()
            .unwrap()
            .iter()
            .map(|vertex| ShaderVertex::new(vertex.position().into(), vertex.normal().into()));

        let indices = data.indices().unwrap();
        let feature_indices: Vec<u32> = data.feature_indices().unwrap().iter().collect();
        let usable_indices = data.usable_indices();
        AvailableVectorLayerData {
            coords: LayerTessellated::coords(&self),
            source_layer: data.layer_name().unwrap().to_owned(),
            buffer: OverAlignedVertexBuffer::from_iters(vertices, indices, usable_indices),
            feature_indices,
        }
    }
}

impl LayerIndexed for FlatBufferTransferable {
    fn message_tag() -> &'static dyn MessageTag {
        &WebMessageTag::LayerIndexed
    }

    fn build_from(coords: WorldTileCoords, _index: TileIndex) -> Self {
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
        FlatBufferTransferable {
            tag: WebMessageTag::LayerIndexed,
            data,
            start,
        }
    }

    fn coords(&self) -> WorldTileCoords {
        let data = root_as_flat_layer_indexed(&self.data[self.start..]).unwrap();
        data.coords().unwrap().into()
    }

    fn to_tile_index(self) -> TileIndex {
        TileIndex::Linear { list: vec![] } // TODO index
    }
}

impl LayerRaster for FlatBufferTransferable {
    fn message_tag() -> &'static dyn MessageTag {
        &WebMessageTag::LayerRaster
    }

    fn build_from(coords: WorldTileCoords, layer_name: String, image: RgbaImage) -> Self {
        let mut inner_builder = FlatBufferBuilder::with_capacity(1024);

        let width = image.width();
        let height = image.height();

        let layer_name = inner_builder.create_string(&layer_name);
        let image_data = inner_builder.create_vector(&image.into_vec());

        let mut builder = FlatLayerRasterBuilder::new(&mut inner_builder);

        builder.add_coords(&FlatWorldTileCoords::new(
            coords.x,
            coords.y,
            coords.z.into(),
        ));
        builder.add_layer_name(layer_name);
        builder.add_image_data(image_data);
        builder.add_width(width);
        builder.add_height(height);

        let root = builder.finish();
        inner_builder.finish(root, None);
        let (data, start) = inner_builder.collapse();
        FlatBufferTransferable {
            tag: WebMessageTag::LayerRaster,
            data,
            start,
        }
    }

    fn coords(&self) -> WorldTileCoords {
        let data = root_as_flat_layer_raster(&self.data[self.start..]).unwrap();
        data.coords().unwrap().into()
    }

    fn to_layer(self) -> AvailableRasterLayerData {
        let data = root_as_flat_layer_raster(&self.data[self.start..]).unwrap();
        let image_data = data.image_data().unwrap().iter().collect();
        AvailableRasterLayerData {
            coords: LayerRaster::coords(&self),
            source_layer: "raster".to_owned(),
            image: RgbaImage::from_vec(data.width(), data.height(), image_data).unwrap(),
        }
    }
}

impl LayerRasterMissing for FlatBufferTransferable {
    fn message_tag() -> &'static dyn MessageTag {
        &WebMessageTag::LayerRasterMissing
    }

    fn build_from(coords: WorldTileCoords) -> Self {
        let mut inner_builder = FlatBufferBuilder::with_capacity(1024);
        let mut builder = FlatLayerIndexedBuilder::new(&mut inner_builder);

        builder.add_coords(&FlatWorldTileCoords::new(
            coords.x,
            coords.y,
            coords.z.into(),
        ));
        let root = builder.finish();
        inner_builder.finish(root, None);
        let (data, start) = inner_builder.collapse();
        FlatBufferTransferable {
            tag: WebMessageTag::LayerRasterMissing,
            data,
            start,
        }
    }

    fn coords(&self) -> WorldTileCoords {
        let data = root_as_flat_layer_missing(&self.data[self.start..]).unwrap();
        data.coords().unwrap().into()
    }

    fn to_layer(self) -> MissingRasterLayerData {
        let _data = root_as_flat_layer_raster(&self.data[self.start..]).unwrap();
        MissingRasterLayerData {
            coords: LayerRaster::coords(&self),
            source_layer: "raster".to_string(),
        }
    }
}

#[derive(Copy, Clone)]
pub struct FlatTransferables;

impl VectorTransferables for FlatTransferables {
    type TileTessellated = FlatBufferTransferable;
    type LayerMissing = FlatBufferTransferable;
    type LayerTessellated = FlatBufferTransferable;
    type LayerIndexed = FlatBufferTransferable;
}

impl RasterTransferables for FlatTransferables {
    type LayerRaster = FlatBufferTransferable;
    type LayerRasterMissing = FlatBufferTransferable;
}
