use std::fmt::{Debug, Formatter};

use image::RgbaImage;

use crate::{
    coords::WorldTileCoords,
    io::apc::{IntoMessage, Message},
    raster::RasterLayerData,
};

pub trait LayerRaster: IntoMessage + Debug + Send {
    fn message_tag() -> u32 {
        6
    }

    fn build_from(coords: WorldTileCoords, layer_name: String, image: RgbaImage) -> Self;

    fn coords(&self) -> WorldTileCoords;

    fn to_layer(self) -> RasterLayerData;
}

pub struct DefaultRasterLayer {
    pub coords: WorldTileCoords,
    pub layer_name: String,
    pub image: RgbaImage,
}

impl Debug for DefaultRasterLayer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "DefaultRasterLayer({})", self.coords)
    }
}

impl IntoMessage for DefaultRasterLayer {
    fn into(self) -> Message {
        Message {
            tag: DefaultRasterLayer::message_tag(), // FIXME tcs: Avoid duplicates!
            transferable: Box::new(self),
        }
    }
}

impl LayerRaster for DefaultRasterLayer {
    fn build_from(coords: WorldTileCoords, layer_name: String, image: RgbaImage) -> Self {
        Self {
            coords,
            layer_name,
            image,
        }
    }

    fn coords(&self) -> WorldTileCoords {
        self.coords
    }

    fn to_layer(self) -> RasterLayerData {
        RasterLayerData {
            coords: self.coords,
            source_layer: "raster".to_string(),
            image: self.image,
        }
    }
}

pub trait RasterTransferables: Copy + Clone + 'static {
    type LayerRaster: LayerRaster;
}

#[derive(Copy, Clone)]
pub struct DefaultRasterTransferables;

impl RasterTransferables for DefaultRasterTransferables {
    type LayerRaster = DefaultRasterLayer;
}
