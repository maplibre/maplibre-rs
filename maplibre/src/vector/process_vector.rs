use std::{collections::HashSet, marker::PhantomData};

use geozero::{
    mvt::{tile, Message},
    GeozeroDatasource,
};
use thiserror::Error;

use crate::{
    coords::WorldTileCoords,
    io::{
        apc::{Context, SendError},
        geometry_index::{IndexProcessor, IndexedGeometry, TileIndex},
    },
    render::ShaderVertex,
    tessellation::{zero_tessellator::ZeroTessellator, IndexDataType, OverAlignedVertexBuffer},
    vector::transferables::{
        LayerIndexed, LayerMissing, LayerTessellated, TileTessellated, VectorTransferables,
    },
};

#[derive(Error, Debug)]
pub enum ProcessVectorError {
    /// Sending of results failed
    #[error("sending data back through context failed")]
    SendError(SendError),
    /// Error during processing of the pipeline
    #[error("processing data in pipeline failed")]
    Processing(Box<dyn std::error::Error>),
}

/// A request for a tile at the given coordinates and in the given layers.
pub struct VectorTileRequest {
    pub coords: WorldTileCoords,
    pub layers: HashSet<String>,
}

pub fn process_vector_tile<T: VectorTransferables, C: Context>(
    data: &[u8],
    tile_request: VectorTileRequest,
    context: &mut ProcessVectorContext<T, C>,
) -> Result<(), ProcessVectorError> {
    // Decode

    let mut tile = geozero::mvt::Tile::decode(data).expect("failed to load tile");

    // Available

    let coords = &tile_request.coords;

    for layer in &mut tile.layers {
        let cloned_layer = layer.clone();
        let layer_name: &str = &cloned_layer.name;
        if !tile_request.layers.contains(layer_name) {
            continue;
        }

        let mut tessellator = ZeroTessellator::<IndexDataType>::default();
        if let Err(e) = layer.process(&mut tessellator) {
            context.layer_missing(coords, layer_name)?;

            tracing::error!("layer {layer_name} at {coords} tesselation failed {e:?}");
        } else {
            context.layer_tesselation_finished(
                coords,
                tessellator.buffer.into(),
                tessellator.feature_indices,
                cloned_layer,
            )?;
        }
    }

    // Missing

    let coords = &tile_request.coords;

    let available_layers: HashSet<_> = tile
        .layers
        .iter()
        .map(|layer| layer.name.clone())
        .collect::<HashSet<_>>();

    for missing_layer in tile_request.layers.difference(&available_layers) {
        context.layer_missing(coords, missing_layer)?;
        tracing::info!("requested layer {missing_layer} at {coords} not found in tile");
    }

    // Indexing

    let mut index = IndexProcessor::new();

    for layer in &mut tile.layers {
        layer.process(&mut index).unwrap();
    }

    context.layer_indexing_finished(&tile_request.coords, index.get_geometries())?;

    // End

    tracing::info!("tile tessellated at {coords} finished");
    context.tile_finished(coords)?;

    Ok(())
}

pub struct ProcessVectorContext<T: VectorTransferables, C: Context> {
    context: C,
    phantom_t: PhantomData<T>,
}

impl<T: VectorTransferables, C: Context> ProcessVectorContext<T, C> {
    pub fn new(context: C) -> Self {
        Self {
            context,
            phantom_t: Default::default(),
        }
    }
}

impl<T: VectorTransferables, C: Context> ProcessVectorContext<T, C> {
    pub fn take_context(self) -> C {
        self.context
    }

    fn tile_finished(&mut self, coords: &WorldTileCoords) -> Result<(), ProcessVectorError> {
        self.context
            .send(T::TileTessellated::build_from(*coords))
            .map_err(|e| ProcessVectorError::Processing(Box::new(e)))
    }

    fn layer_missing(
        &mut self,
        coords: &WorldTileCoords,
        layer_name: &str,
    ) -> Result<(), ProcessVectorError> {
        self.context
            .send(T::LayerMissing::build_from(*coords, layer_name.to_owned()))
            .map_err(|e| ProcessVectorError::Processing(Box::new(e)))
    }

    fn layer_tesselation_finished(
        &mut self,
        coords: &WorldTileCoords,
        buffer: OverAlignedVertexBuffer<ShaderVertex, IndexDataType>,
        feature_indices: Vec<u32>,
        layer_data: tile::Layer,
    ) -> Result<(), ProcessVectorError> {
        self.context
            .send(T::LayerTessellated::build_from(
                *coords,
                buffer,
                feature_indices,
                layer_data,
            ))
            .map_err(|e| ProcessVectorError::Processing(Box::new(e)))
    }

    fn layer_indexing_finished(
        &mut self,
        coords: &WorldTileCoords,
        geometries: Vec<IndexedGeometry<f64>>,
    ) -> Result<(), ProcessVectorError> {
        self.context
            .send(T::LayerIndexed::build_from(
                *coords,
                TileIndex::Linear { list: geometries },
            ))
            .map_err(|e| ProcessVectorError::Processing(Box::new(e)))
    }
}

#[cfg(test)]
mod tests {
    use super::ProcessVectorContext;
    use crate::{
        coords::ZoomLevel,
        io::apc::tests::DummyContext,
        vector::{
            process_vector::{process_vector_tile, VectorTileRequest},
            DefaultVectorTransferables,
        },
    };

    #[test] // TODO: Add proper tile byte array
    #[ignore]
    fn test() {
        let _output = process_vector_tile(
            &[0],
            VectorTileRequest {
                coords: (0, 0, ZoomLevel::default()).into(),
                layers: Default::default(),
            },
            &mut ProcessVectorContext::<DefaultVectorTransferables, _>::new(DummyContext),
        );
    }
}
