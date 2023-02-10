use std::{collections::HashSet, marker::PhantomData};

use geozero::{mvt::Message, GeozeroDatasource};
use image::RgbaImage;

use crate::{
    coords::WorldTileCoords,
    io::{
        apc::Context,
        geometry_index::IndexProcessor,
        pipeline::{DataPipeline, PipelineContext, PipelineEnd, PipelineError, Processable},
        source_client::HttpClient,
    },
    raster::transferables::{LayerRaster, Transferables},
    tessellation::{zero_tessellator::ZeroTessellator, IndexDataType},
};

pub struct RasterTileRequest {
    pub coords: WorldTileCoords,
}

pub struct RasterLayer<T, HC, C>(PhantomData<T>, PhantomData<HC>, PhantomData<C>);

impl<T, HC, C> Default for RasterLayer<T, HC, C> {
    fn default() -> Self {
        Self {
            0: Default::default(),
            1: Default::default(),
            2: Default::default(),
        }
    }
}

impl<T: Transferables, HC: HttpClient, C: Context<HC>> Processable for RasterLayer<T, HC, C> {
    type Input = (RasterTileRequest, Box<[u8]>);
    type Output = WorldTileCoords;
    type Context = RasterPipelineProcessor<T, HC, C>;

    fn process(
        &self,
        (tile_request, data): Self::Input,
        context: &mut Self::Context,
    ) -> Result<Self::Output, PipelineError> {
        let coords = &tile_request.coords;
        let img = image::load_from_memory(&data).unwrap();
        let rgba = img.to_rgba8();

        context.layer_raster_finished(coords, "raster".to_string(), rgba.clone())?;

        Ok(tile_request.coords)
    }
}

pub fn build_raster_tile_pipeline<T: Transferables, HC: HttpClient, C: Context<HC>>(
) -> impl Processable<
    Input = <RasterLayer<T, HC, C> as Processable>::Input,
    Context = RasterPipelineProcessor<T, HC, C>,
> {
    DataPipeline::new(
        RasterLayer::default(),
        /* FIXME: Raster end */
        PipelineEnd::default(),
    )
}

pub struct RasterPipelineProcessor<T: Transferables, HC: HttpClient, C: Context<HC>> {
    context: C,
    phantom_t: PhantomData<T>,
    phantom_hc: PhantomData<HC>,
}

impl<T: Transferables, HC: HttpClient, C: Context<HC>> RasterPipelineProcessor<T, HC, C> {
    pub fn new(context: C) -> Self {
        Self {
            context,
            phantom_t: Default::default(),
            phantom_hc: Default::default(),
        }
    }
}

impl<T: Transferables, HC: HttpClient, C: Context<HC>> RasterPipelineProcessor<T, HC, C> {
    fn layer_raster_finished(
        &mut self,
        coords: &WorldTileCoords,
        layer_name: String,
        image_data: RgbaImage,
    ) -> Result<(), PipelineError> {
        self.context
            .send(T::LayerRaster::build_from(*coords, layer_name, image_data))
            .map_err(|e| PipelineError::Processing(Box::new(e)))
    }
}

#[cfg(test)]
mod tests {
    use super::build_vector_tile_pipeline;
    use crate::{
        coords::ZoomLevel,
        io::{
            pipeline::{PipelineContext, PipelineProcessor, Processable},
            raster_pipeline::VectorTileRequest,
        },
    };

    pub struct DummyPipelineProcessor;

    impl PipelineProcessor for DummyPipelineProcessor {}

    #[test] // TODO: Add proper tile byte array
    #[ignore]
    fn test() {
        let mut context = PipelineContext::new(DummyPipelineProcessor);

        let pipeline = build_vector_tile_pipeline();
        let _output = pipeline.process(
            (
                VectorTileRequest {
                    coords: (0, 0, ZoomLevel::default()).into(),
                    layers: Default::default(),
                },
                Box::new([0]),
            ),
            &mut context,
        );
    }
}
