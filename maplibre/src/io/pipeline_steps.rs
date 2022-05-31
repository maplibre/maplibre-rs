use crate::io::geometry_index::IndexProcessor;
use crate::io::pipeline::{EndStep, PipelineContext, PipelineStep, Processable};
use crate::io::{TileRequest, TileRequestID};
use crate::tessellation::zero_tessellator::ZeroTessellator;
use crate::tessellation::IndexDataType;
use geozero::GeozeroDatasource;
use prost::Message;
use std::collections::HashSet;

pub struct ParseTileStep;

impl Processable for ParseTileStep {
    type Input = (TileRequest, TileRequestID, Box<[u8]>);
    type Output = (TileRequest, TileRequestID, geozero::mvt::Tile);

    // TODO (perf): Maybe force inline
    fn process(
        &self,
        (tile_request, request_id, data): Self::Input,
        _context: &mut PipelineContext,
    ) -> Self::Output {
        let tile = geozero::mvt::Tile::decode(data.as_ref()).expect("failed to load tile");
        (tile_request, request_id, tile)
    }
}

pub struct IndexLayerStep;

impl Processable for IndexLayerStep {
    type Input = (TileRequest, TileRequestID, geozero::mvt::Tile);
    type Output = (TileRequest, TileRequestID, geozero::mvt::Tile);

    // TODO (perf): Maybe force inline
    fn process(
        &self,
        (tile_request, request_id, tile): Self::Input,
        context: &mut PipelineContext,
    ) -> Self::Output {
        let index = IndexProcessor::new();

        context
            .processor
            .finished_layer_indexing(&tile_request.coords, index.get_geometries());
        (tile_request, request_id, tile)
    }
}

pub struct TessellateLayerStep;

impl Processable for TessellateLayerStep {
    type Input = (TileRequest, TileRequestID, geozero::mvt::Tile);
    type Output = (TileRequest, TileRequestID, geozero::mvt::Tile);

    // TODO (perf): Maybe force inline
    fn process(
        &self,
        (tile_request, request_id, mut tile): Self::Input,
        context: &mut PipelineContext,
    ) -> Self::Output {
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
                context.processor.unavailable_layer(coords, layer_name);

                tracing::error!(
                    "layer {} at {} tesselation failed {:?}",
                    layer_name,
                    &coords,
                    e
                );
            } else {
                context.processor.finished_layer_tesselation(
                    coords,
                    tessellator.buffer.into(),
                    tessellator.feature_indices,
                    cloned_layer,
                )
            }
        }

        let available_layers: HashSet<_> = tile
            .layers
            .iter()
            .map(|layer| layer.name.clone())
            .collect::<HashSet<_>>();

        for missing_layer in tile_request.layers.difference(&available_layers) {
            context.processor.unavailable_layer(coords, missing_layer);

            tracing::info!(
                "requested layer {} at {} not found in tile",
                missing_layer,
                &coords
            );
        }

        tracing::info!("tile tessellated at {} finished", &tile_request.coords);

        context
            .processor
            .finished_tile_tesselation(request_id, &tile_request.coords);

        (tile_request, request_id, tile)
    }
}

pub fn build_vector_tile_pipeline(
) -> impl Processable<Input = <ParseTileStep as Processable>::Input> {
    PipelineStep::new(
        ParseTileStep,
        PipelineStep::new(TessellateLayerStep, EndStep::default()),
    )
}

#[cfg(test)]
mod tests {
    use super::build_vector_tile_pipeline;
    use crate::io::pipeline::{PipelineContext, PipelineProcessor, Processable};
    use crate::io::TileRequest;
    pub struct DummyPipelineProcessor;

    impl PipelineProcessor for DummyPipelineProcessor {}

    #[test]
    fn test() {
        let mut context = PipelineContext {
            processor: Box::new(DummyPipelineProcessor),
        };

        let pipeline = build_vector_tile_pipeline();
        let output = pipeline.process(
            (
                TileRequest {
                    coords: (0, 0, 0).into(),
                    layers: Default::default(),
                },
                0,
                Box::new([0]),
            ),
            &mut context,
        );
    }
}
