use std::collections::HashSet;

use geozero::GeozeroDatasource;
use image::RgbaImage;
use log::error;
use prost::Message;

use crate::{
    coords::WorldTileCoords,
    io::{
        geometry_index::IndexProcessor,
        pipeline::{DataPipeline, PipelineContext, PipelineEnd, PipelineError, Processable},
    },
    tessellation::{zero_tessellator::ZeroTessellator, IndexDataType},
};

/// A request for a tile at the given coordinates and in the given layers.
pub struct VectorTileRequest {
    pub coords: WorldTileCoords,
    pub layers: HashSet<String>,
}

#[derive(Clone)]
pub enum PipelineTile {
    Vector(geozero::mvt::Tile),
    Raster(RgbaImage),
}

#[derive(Default)]
pub struct ParseTile;

impl Processable for ParseTile {
    type Input = (VectorTileRequest, Box<[u8]>);
    type Output = (VectorTileRequest, geozero::mvt::Tile);

    #[tracing::instrument(skip_all)]
    fn process(
        &self,
        (tile_request, data): Self::Input,
        _context: &mut PipelineContext,
    ) -> Result<Self::Output, PipelineError> {
        let tile = geozero::mvt::Tile::decode(data.as_ref()).expect("failed to load tile");
        Ok((tile_request, tile))
    }
}

#[derive(Default)]
pub struct IndexLayer;

impl Processable for IndexLayer {
    type Input = (VectorTileRequest, geozero::mvt::Tile);
    type Output = WorldTileCoords;

    #[tracing::instrument(skip_all)]
    fn process(
        &self,
        (tile_request, mut tile): Self::Input,
        context: &mut PipelineContext,
    ) -> Result<Self::Output, PipelineError> {
        let mut index = IndexProcessor::new();

        for layer in &mut tile.layers {
            layer.process(&mut index).unwrap();
        }

        context
            .processor_mut()
            .layer_indexing_finished(&tile_request.coords, index.get_geometries())?;
        Ok(tile_request.coords)
    }
}

#[derive(Default)]
pub struct TessellateLayer;

impl Processable for TessellateLayer {
    type Input = (VectorTileRequest, geozero::mvt::Tile);
    type Output = (VectorTileRequest, geozero::mvt::Tile);

    #[tracing::instrument(skip_all)]
    fn process(
        &self,
        (tile_request, mut tile): Self::Input,
        context: &mut PipelineContext,
    ) -> Result<Self::Output, PipelineError> {
        let coords = &tile_request.coords;

        for layer in &mut tile.layers {
            let cloned_layer = layer.clone();
            let layer_name: &str = &cloned_layer.name;
            if !tile_request.layers.contains(layer_name) {
                continue;
            }

            tracing::info!("layer {} at {} ready", layer_name, coords);

            let mut tessellator = ZeroTessellator::<IndexDataType>::default();
            if let Err(e) = layer.process(&mut tessellator) {
                context
                    .processor_mut()
                    .layer_unavailable(coords, layer_name)?;

                tracing::error!(
                    "layer {} at {} tesselation failed {:?}",
                    layer_name,
                    &coords,
                    e
                );
            } else {
                context.processor_mut().layer_tesselation_finished(
                    coords,
                    tessellator.buffer.into(),
                    tessellator.feature_indices,
                    cloned_layer,
                )?;
            }
        }

        Ok((tile_request, tile))
    }
}

#[derive(Default)]
pub struct TessellateLayerUnavailable;

impl Processable for TessellateLayerUnavailable {
    type Input = (VectorTileRequest, geozero::mvt::Tile);
    type Output = (VectorTileRequest, geozero::mvt::Tile);

    // TODO (perf): Maybe force inline
    fn process(
        &self,
        (tile_request, tile): Self::Input,
        context: &mut PipelineContext,
    ) -> Result<Self::Output, PipelineError> {
        let coords = &tile_request.coords;

        let available_layers: HashSet<_> = tile
            .layers
            .iter()
            .map(|layer| layer.name.clone())
            .collect::<HashSet<_>>();

        for missing_layer in tile_request.layers.difference(&available_layers) {
            context
                .processor_mut()
                .layer_unavailable(coords, missing_layer)?;

            tracing::info!(
                "requested layer {} at {} not found in tile",
                missing_layer,
                &coords
            );
        }
        Ok((tile_request, tile))
    }
}

#[derive(Default)]
pub struct TileFinished;

impl Processable for TileFinished {
    type Input = WorldTileCoords;
    type Output = ();

    fn process(
        &self,
        coords: Self::Input,
        context: &mut PipelineContext,
    ) -> Result<Self::Output, PipelineError> {
        tracing::info!("tile tessellated at {} finished", &coords);

        context.processor_mut().tile_finished(&coords)?;

        Ok(())
    }
}

pub fn build_vector_tile_pipeline() -> impl Processable<Input = <ParseTile as Processable>::Input> {
    DataPipeline::new(
        ParseTile,
        DataPipeline::new(
            TessellateLayer,
            DataPipeline::new(
                TessellateLayerUnavailable,
                DataPipeline::new(
                    IndexLayer,
                    DataPipeline::new(TileFinished, PipelineEnd::default()),
                ),
            ),
        ),
    )
}

pub struct RasterTileRequest {
    pub coords: WorldTileCoords,
}

#[derive(Default)]
pub struct RasterLayer;

impl Processable for RasterLayer {
    type Input = (RasterTileRequest, Box<[u8]>);
    type Output = WorldTileCoords;

    fn process(
        &self,
        (tile_request, data): Self::Input,
        context: &mut PipelineContext,
    ) -> Result<Self::Output, PipelineError> {
        let coords = &tile_request.coords;
        let data = data.to_vec();
        let img = image::load_from_memory(&data).unwrap();
        let rgba = img.to_rgba8();

        error!("layer raster finished");
        context.processor_mut().layer_raster_finished(
            coords,
            "raster".to_string(),
            rgba.clone(),
        )?;

        Ok(tile_request.coords)
    }
}

pub fn build_raster_tile_pipeline() -> impl Processable<Input = <RasterLayer as Processable>::Input>
{
    DataPipeline::new(
        RasterLayer,
        DataPipeline::new(TileFinished, PipelineEnd::default()),
    )
}

#[cfg(test)]
mod tests {
    use super::build_vector_tile_pipeline;
    use crate::{
        coords::ZoomLevel,
        io::{
            pipeline::{PipelineContext, PipelineProcessor, Processable},
            TileRequest,
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
                TileRequest {
                    coords: (0, 0, ZoomLevel::default()).into(),
                    layers: Default::default(),
                },
                Box::new([0]),
            ),
            &mut context,
        );
    }
}
