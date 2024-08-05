use std::{borrow::Cow, collections::HashSet, marker::PhantomData};

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
    vector::tessellation::{ZeroTessellator, IndexDataType, OverAlignedVertexBuffer},
    vector::transferables::{
        LayerIndexed, LayerMissing, LayerTessellated, TileTessellated, VectorTransferables,
    },
};
use crate::render::shaders::SymbolVertex;
use crate::sdf::tessellation::TextTessellator;
use crate::style::layer::{LayerPaint, StyleLayer};
use crate::vector::transferables::SymbolLayerTessellated;

#[derive(Error, Debug)]
pub enum ProcessVectorError {
    /// Sending of results failed
    #[error("sending data back through context failed")]
    SendError(SendError),
    /// Error when decoding e.g. the protobuf file
    #[error("decoding failed")]
    Decoding(Cow<'static, str>),
}

/// A request for a tile at the given coordinates and in the given layers.
pub struct VectorTileRequest {
    pub coords: WorldTileCoords,
    pub layers: HashSet<StyleLayer>,
}

pub fn process_vector_tile<T: VectorTransferables, C: Context>(
    data: &[u8],
    tile_request: VectorTileRequest,
    context: &mut ProcessVectorContext<T, C>,
) -> Result<(), ProcessVectorError> {
    let mut tile = geozero::mvt::Tile::decode(data)
        .map_err(|e| ProcessVectorError::Decoding(e.to_string().into()))?;

    // Report available layers
    let coords = &tile_request.coords;

    for style_layer in &tile_request.layers {
        let id = &style_layer.id;
        if let (Some(paint), Some(source_layer)) = (&style_layer.paint, &style_layer.source_layer) {

            if let Some(layer) = tile.layers.iter_mut().find(|layer| &layer.name == source_layer) {
                let original_layer = layer.clone();

                match paint {
                    LayerPaint::Line(_) | LayerPaint::Fill(_) => {
                        let mut tessellator = ZeroTessellator::<IndexDataType>::default();

                        if let Err(e) = layer.process(&mut tessellator) {
                            context.layer_missing(coords, &source_layer)?;

                            tracing::error!("tesselation for layer source {source_layer} at {coords} failed {e:?}");
                        } else {
                            context.layer_tesselation_finished(
                                coords,
                                tessellator.buffer.into(),
                                tessellator.feature_indices,
                                original_layer,
                            )?;
                        }
                    }
                    LayerPaint::Symbol(_) => {

                        let mut tessellator = TextTessellator::<IndexDataType>::default();

                        if let Err(e) = layer.process(&mut tessellator) {
                            context.layer_missing(coords, &source_layer)?;

                            tracing::error!("tesselation for layer source {source_layer} at {coords} failed {e:?}");
                        } else {
                            if tessellator.quad_buffer.indices.is_empty() {
                                log::error!("quad buffer empty");
                                continue
                            }
                            context.symbol_layer_tesselation_finished(
                                coords,
                                tessellator.quad_buffer.into(),
                                tessellator.feature_indices,
                                original_layer,
                            )?;
                        }

                    }
                    _ => {
                        log::warn!("unhandled style layer type in {id}");
                    }
                }
            } else {
                log::warn!("layer source {source_layer} not found in vector tile");
            }
        } else {
            log::error!("vector style layer {id} misses a required attribute");
        }
    }

    // Report missing layers
    let coords = &tile_request.coords;
    let available_layers: HashSet<_> = tile
        .layers
        .iter()
        .map(|layer| layer.name.clone())
        .collect::<HashSet<_>>();

    // todo for missing_layer in tile_request.layers.difference(&available_layers) {
    //    context.layer_missing(coords, &missing_layer.id)?;
    //    tracing::info!("requested layer {missing_layer} at {coords} not found in tile");
    //}

    // Report index for layer
    let mut index = IndexProcessor::new();

    for layer in &mut tile.layers {
        layer.process(&mut index).unwrap();
    }

    context.layer_indexing_finished(&tile_request.coords, index.get_geometries())?;

    // Report end
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
            .send_back(T::TileTessellated::build_from(*coords))
            .map_err(|e| ProcessVectorError::SendError(e))
    }

    fn layer_missing(
        &mut self,
        coords: &WorldTileCoords,
        layer_name: &str,
    ) -> Result<(), ProcessVectorError> {
        self.context
            .send_back(T::LayerMissing::build_from(*coords, layer_name.to_owned()))
            .map_err(|e| ProcessVectorError::SendError(e))
    }

    fn layer_tesselation_finished(
        &mut self,
        coords: &WorldTileCoords,
        buffer: OverAlignedVertexBuffer<ShaderVertex, IndexDataType>,
        feature_indices: Vec<u32>,
        layer_data: tile::Layer,
    ) -> Result<(), ProcessVectorError> {
        self.context
            .send_back(T::LayerTessellated::build_from(
                *coords,
                buffer,
                feature_indices,
                layer_data,
            ))
            .map_err(|e| ProcessVectorError::SendError(e))
    }

    fn symbol_layer_tesselation_finished(
        &mut self,
        coords: &WorldTileCoords,
        buffer: OverAlignedVertexBuffer<SymbolVertex, IndexDataType>,
        feature_indices: Vec<u32>,
        layer_data: tile::Layer,
    ) -> Result<(), ProcessVectorError> {
        self.context
            .send(T::SymbolLayerTessellated::build_from(
                *coords,
                buffer,
                feature_indices,
                layer_data,
            ))
            .map_err(|e| ProcessVectorError::SendError(e))
    }


    fn layer_indexing_finished(
        &mut self,
        coords: &WorldTileCoords,
        geometries: Vec<IndexedGeometry<f64>>,
    ) -> Result<(), ProcessVectorError> {
        self.context
            .send_back(T::LayerIndexed::build_from(
                *coords,
                TileIndex::Linear { list: geometries },
            ))
            .map_err(|e| ProcessVectorError::SendError(e))
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
