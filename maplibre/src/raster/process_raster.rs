use std::marker::PhantomData;

use image::RgbaImage;
use thiserror::Error;

use crate::{
    coords::WorldTileCoords,
    io::apc::Context,
    raster::transferables::{LayerRaster, RasterTransferables},
};

#[derive(Error, Debug)]
pub enum ProcessRasterError {
    /// Error during processing of the pipeline
    #[error("processing data in pipeline failed")]
    Processing(Box<dyn std::error::Error>),
}

pub struct RasterTileRequest {
    pub coords: WorldTileCoords,
}

pub fn process_raster_tile<T: RasterTransferables, C: Context>(
    data: &[u8],
    tile_request: RasterTileRequest,
    context: &mut ProcessRasterContext<T, C>,
) -> Result<(), ProcessRasterError> {
    let coords = &tile_request.coords;
    let img = image::load_from_memory(data).unwrap();
    let rgba = img.to_rgba8();

    context.layer_raster_finished(coords, "raster".to_string(), rgba)?;

    Ok(())
}
pub struct ProcessRasterContext<T: RasterTransferables, C: Context> {
    context: C,
    phantom_t: PhantomData<T>,
}

impl<T: RasterTransferables, C: Context> ProcessRasterContext<T, C> {
    pub fn new(context: C) -> Self {
        Self {
            context,
            phantom_t: Default::default(),
        }
    }
}

impl<T: RasterTransferables, C: Context> ProcessRasterContext<T, C> {
    fn layer_raster_finished(
        &mut self,
        coords: &WorldTileCoords,
        layer_name: String,
        image_data: RgbaImage,
    ) -> Result<(), ProcessRasterError> {
        self.context
            .send_back(T::LayerRaster::build_from(*coords, layer_name, image_data))
            .map_err(|e| ProcessRasterError::Processing(Box::new(e)))
    }
}

#[cfg(test)]
mod tests {
    use super::process_raster_tile;
    use crate::{
        coords::ZoomLevel,
        io::apc::tests::DummyContext,
        raster::{
            process_raster::{ProcessRasterContext, RasterTileRequest},
            DefaultRasterTransferables,
        },
    };

    #[test] // TODO: Add proper tile byte array
    #[ignore]
    fn test() {
        let _output = process_raster_tile(
            &[0],
            RasterTileRequest {
                coords: (0, 0, ZoomLevel::default()).into(),
            },
            &mut ProcessRasterContext::<DefaultRasterTransferables, _>::new(DummyContext),
        );
    }
}
