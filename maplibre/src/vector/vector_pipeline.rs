use std::{collections::HashSet, marker::PhantomData};

use geozero::{
    mvt::{tile, Message},
    GeozeroDatasource,
};
use image::RgbaImage;

use crate::{
    coords::WorldTileCoords,
    io::{
        apc::Context,
        geometry_index::{IndexProcessor, IndexedGeometry, TileIndex},
        pipeline::{DataPipeline, PipelineContext, PipelineEnd, PipelineError, Processable},
        source_client::HttpClient,
    },
    render::ShaderVertex,
    tessellation::{zero_tessellator::ZeroTessellator, IndexDataType, OverAlignedVertexBuffer},
    vector::transferables::{
        LayerIndexed, LayerTessellated, LayerUnavailable, TileTessellated, Transferables,
    },
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

pub struct ParseTile<T, HC, C>(PhantomData<T>, PhantomData<HC>, PhantomData<C>);

impl<T, HC, C> Default for ParseTile<T, HC, C> {
    fn default() -> Self {
        Self(Default::default(), Default::default(), Default::default())
    }
}

impl<T: Transferables, HC: HttpClient, C: Context<HC>> Processable for ParseTile<T, HC, C> {
    type Input = (VectorTileRequest, Box<[u8]>);
    type Output = (VectorTileRequest, geozero::mvt::Tile);
    type Context = VectorPipelineProcessor<T, HC, C>;

    #[tracing::instrument(skip_all)]
    fn process(
        &self,
        (tile_request, data): Self::Input,
        _context: &mut Self::Context,
    ) -> Result<Self::Output, PipelineError> {
        let tile = geozero::mvt::Tile::decode(data.as_ref()).expect("failed to load tile");
        Ok((tile_request, tile))
    }
}

pub struct IndexLayer<T, HC, C>(PhantomData<T>, PhantomData<HC>, PhantomData<C>);

impl<T, HC, C> Default for IndexLayer<T, HC, C> {
    fn default() -> Self {
        Self(Default::default(), Default::default(), Default::default())
    }
}

impl<T: Transferables, HC: HttpClient, C: Context<HC>> Processable for IndexLayer<T, HC, C> {
    type Input = (VectorTileRequest, geozero::mvt::Tile);
    type Output = WorldTileCoords;
    type Context = VectorPipelineProcessor<T, HC, C>;

    #[tracing::instrument(skip_all)]
    fn process(
        &self,
        (tile_request, mut tile): Self::Input,
        context: &mut Self::Context,
    ) -> Result<Self::Output, PipelineError> {
        let mut index = IndexProcessor::new();

        for layer in &mut tile.layers {
            layer.process(&mut index).unwrap();
        }

        context.layer_indexing_finished(&tile_request.coords, index.get_geometries())?;
        Ok(tile_request.coords)
    }
}

pub struct TessellateLayer<T, HC, C>(PhantomData<T>, PhantomData<HC>, PhantomData<C>);

impl<T, HC, C> Default for TessellateLayer<T, HC, C> {
    fn default() -> Self {
        Self(Default::default(), Default::default(), Default::default())
    }
}

impl<T: Transferables, HC: HttpClient, C: Context<HC>> Processable for TessellateLayer<T, HC, C> {
    type Input = (VectorTileRequest, geozero::mvt::Tile);
    type Output = (VectorTileRequest, geozero::mvt::Tile);
    type Context = VectorPipelineProcessor<T, HC, C>;

    #[tracing::instrument(skip_all)]
    fn process(
        &self,
        (tile_request, mut tile): Self::Input,
        context: &mut Self::Context,
    ) -> Result<Self::Output, PipelineError> {
        let coords = &tile_request.coords;

        for layer in &mut tile.layers {
            let cloned_layer = layer.clone();
            let layer_name: &str = &cloned_layer.name;
            if !tile_request.layers.contains(layer_name) {
                continue;
            }

            let mut tessellator = ZeroTessellator::<IndexDataType>::default();
            if let Err(e) = layer.process(&mut tessellator) {
                context.layer_unavailable(coords, layer_name)?;

                tracing::error!(
                    "layer {} at {} tesselation failed {:?}",
                    layer_name,
                    &coords,
                    e
                );
            } else {
                context.layer_tesselation_finished(
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

pub struct TessellateLayerUnavailable<T, HC, C>(PhantomData<T>, PhantomData<HC>, PhantomData<C>);

impl<T, HC, C> Default for TessellateLayerUnavailable<T, HC, C> {
    fn default() -> Self {
        Self(Default::default(), Default::default(), Default::default())
    }
}

impl<T: Transferables, HC: HttpClient, C: Context<HC>> Processable
    for TessellateLayerUnavailable<T, HC, C>
{
    type Input = (VectorTileRequest, geozero::mvt::Tile);
    type Output = (VectorTileRequest, geozero::mvt::Tile);
    type Context = VectorPipelineProcessor<T, HC, C>;

    // TODO (perf): Maybe force inline
    fn process(
        &self,
        (tile_request, tile): Self::Input,
        context: &mut Self::Context,
    ) -> Result<Self::Output, PipelineError> {
        let coords = &tile_request.coords;

        let available_layers: HashSet<_> = tile
            .layers
            .iter()
            .map(|layer| layer.name.clone())
            .collect::<HashSet<_>>();

        for missing_layer in tile_request.layers.difference(&available_layers) {
            context.layer_unavailable(coords, missing_layer)?;

            tracing::info!(
                "requested layer {} at {} not found in tile",
                missing_layer,
                &coords
            );
        }
        Ok((tile_request, tile))
    }
}

pub struct TileFinished<T, HC, C>(PhantomData<T>, PhantomData<HC>, PhantomData<C>);

impl<T, HC, C> Default for TileFinished<T, HC, C> {
    fn default() -> Self {
        Self(Default::default(), Default::default(), Default::default())
    }
}

impl<T: Transferables, HC: HttpClient, C: Context<HC>> Processable for TileFinished<T, HC, C> {
    type Input = WorldTileCoords;
    type Output = ();
    type Context = VectorPipelineProcessor<T, HC, C>;

    fn process(
        &self,
        coords: Self::Input,
        context: &mut Self::Context,
    ) -> Result<Self::Output, PipelineError> {
        tracing::info!("tile tessellated at {} finished", &coords);

        context.tile_finished(&coords)?;

        Ok(())
    }
}

pub fn build_vector_tile_pipeline<T: Transferables, HC: HttpClient, C: Context<HC>>(
) -> impl Processable<
    Input = <ParseTile<T, HC, C> as Processable>::Input,
    Context = VectorPipelineProcessor<T, HC, C>,
> {
    DataPipeline::new(
        ParseTile::default(),
        DataPipeline::new(
            TessellateLayer::default(),
            DataPipeline::new(
                TessellateLayerUnavailable::default(),
                DataPipeline::new(
                    IndexLayer::default(),
                    DataPipeline::new(TileFinished::default(), PipelineEnd::default()),
                ),
            ),
        ),
    )
}

pub struct VectorPipelineProcessor<T: Transferables, HC: HttpClient, C: Context<HC>> {
    context: C,
    phantom_t: PhantomData<T>,
    phantom_hc: PhantomData<HC>,
}

impl<T: Transferables, HC: HttpClient, C: Context<HC>> VectorPipelineProcessor<T, HC, C> {
    pub fn new(context: C) -> Self {
        Self {
            context,
            phantom_t: Default::default(),
            phantom_hc: Default::default(),
        }
    }
}

impl<T: Transferables, HC: HttpClient, C: Context<HC>> VectorPipelineProcessor<T, HC, C> {
    fn tile_finished(&mut self, coords: &WorldTileCoords) -> Result<(), PipelineError> {
        self.context
            .send(T::TileTessellated::build_from(*coords))
            .map_err(|e| PipelineError::Processing(Box::new(e)))
    }

    fn layer_unavailable(
        &mut self,
        coords: &WorldTileCoords,
        layer_name: &str,
    ) -> Result<(), PipelineError> {
        self.context
            .send(T::LayerUnavailable::build_from(
                *coords,
                layer_name.to_owned(),
            ))
            .map_err(|e| PipelineError::Processing(Box::new(e)))
    }

    fn layer_tesselation_finished(
        &mut self,
        coords: &WorldTileCoords,
        buffer: OverAlignedVertexBuffer<ShaderVertex, IndexDataType>,
        feature_indices: Vec<u32>,
        layer_data: tile::Layer,
    ) -> Result<(), PipelineError> {
        self.context
            .send(T::LayerTessellated::build_from(
                *coords,
                buffer,
                feature_indices,
                layer_data,
            ))
            .map_err(|e| PipelineError::Processing(Box::new(e)))
    }

    fn layer_indexing_finished(
        &mut self,
        coords: &WorldTileCoords,
        geometries: Vec<IndexedGeometry<f64>>,
    ) -> Result<(), PipelineError> {
        self.context
            .send(T::LayerIndexed::build_from(
                *coords,
                TileIndex::Linear { list: geometries },
            ))
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
        vector::vector_pipeline::VectorTileRequest,
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
