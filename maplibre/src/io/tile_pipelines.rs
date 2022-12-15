use std::collections::HashSet;

use geozero::GeozeroDatasource;
use prost::Message;

use crate::{
    io::{
        geometry_index::IndexProcessor,
        pipeline::{DataPipeline, PipelineContext, PipelineEnd, PipelineError, Processable},
        TileRequest,
    },
    tessellation::{zero_tessellator::ZeroTessellator, IndexDataType},
};

#[derive(Default)]
pub struct ParseTile;

impl Processable for ParseTile {
    type Input = (TileRequest, Box<[u8]>);
    type Output = (TileRequest, geozero::mvt::Tile);

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
    type Input = (TileRequest, geozero::mvt::Tile);
    type Output = (TileRequest, geozero::mvt::Tile);

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

        for layer in &mut tile.layers {
            layer.process(&mut index).unwrap();
        }

        context
            .processor_mut()
            .layer_indexing_finished(&tile_request.coords, index.get_geometries())?;
        Ok((tile_request, tile))
    }
}

#[derive(Default)]
pub struct TessellateLayer;

impl Processable for TessellateLayer {
    type Input = (TileRequest, geozero::mvt::Tile);
    type Output = (TileRequest, geozero::mvt::Tile);

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

        tracing::info!("tile tessellated at {} finished", &tile_request.coords);

        context
            .processor_mut()
            .tile_finished(&tile_request.coords)?;

        Ok((tile_request, tile))
    }
}

pub fn build_vector_tile_pipeline() -> impl Processable<Input = <ParseTile as Processable>::Input> {
    DataPipeline::new(
        ParseTile,
        DataPipeline::new(
            TessellateLayer,
            DataPipeline::new(IndexLayer, PipelineEnd::default()),
        ),
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

    #[test]
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
                Box::new([0]), // TODO: Add proper tile byte array
            ),
            &mut context,
        );
    }
}
