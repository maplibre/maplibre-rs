use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    marker::PhantomData,
};

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
    render::{
        shaders::{ShaderSymbolVertex, ShaderSymbolVertexNew},
        ShaderVertex,
    },
    sdf::{tessellation::TextTessellator, tessellation_new::TextTessellatorNew, Feature},
    style::layer::{LayerPaint, StyleLayer},
    vector::{
        tessellation::{IndexDataType, OverAlignedVertexBuffer, ZeroTessellator},
        transferables::{
            LayerIndexed, LayerMissing, LayerTessellated, SymbolLayerTessellated, TileTessellated,
            VectorTransferables,
        },
    },
};

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

/// Resolve the properties of an MVT feature into a HashMap of string key-value pairs,
/// using the layer's keys/values dictionaries.
fn resolve_feature_properties(
    layer: &tile::Layer,
    feature: &tile::Feature,
) -> HashMap<String, String> {
    let mut props = HashMap::new();
    for pair in feature.tags.chunks(2) {
        let [key_idx, value_idx] = [pair[0], pair[1]];
        let Some(key) = layer.keys.get(key_idx as usize) else {
            continue;
        };
        let Some(value) = layer.values.get(value_idx as usize) else {
            continue;
        };
        let val_str = if let Some(ref v) = value.string_value {
            v.clone()
        } else if let Some(v) = value.float_value {
            v.to_string()
        } else if let Some(v) = value.double_value {
            v.to_string()
        } else if let Some(v) = value.int_value {
            v.to_string()
        } else if let Some(v) = value.uint_value {
            v.to_string()
        } else if let Some(v) = value.sint_value {
            v.to_string()
        } else if let Some(v) = value.bool_value {
            v.to_string()
        } else {
            continue;
        };
        props.insert(key.clone(), val_str);
    }
    props
}

/// Evaluate a MapLibre GL JS legacy filter expression against feature properties.
/// Supports: ["all", ...], ["any", ...], ["==", key, val], ["!=", key, val],
/// ["has", key], ["!has", key], ["in", key, v1, v2, ...], ["!in", key, v1, v2, ...]
fn evaluate_filter(filter: &serde_json::Value, props: &HashMap<String, String>) -> bool {
    let Some(arr) = filter.as_array() else {
        return true; // non-array filter passes everything
    };
    let Some(op) = arr.first().and_then(|v| v.as_str()) else {
        return true;
    };
    match op {
        "all" => arr[1..].iter().all(|f| evaluate_filter(f, props)),
        "any" => arr[1..].iter().any(|f| evaluate_filter(f, props)),
        "none" => !arr[1..].iter().any(|f| evaluate_filter(f, props)),
        "==" if arr.len() >= 3 => {
            let key = arr[1].as_str().unwrap_or("");
            let expected = arr[2].as_str().map(|s| s.to_string()).unwrap_or_else(|| {
                // Handle numeric comparisons
                arr[2].as_f64().map(|n| n.to_string()).unwrap_or_default()
            });
            props.get(key).map(|v| v == &expected).unwrap_or(false)
        }
        "!=" if arr.len() >= 3 => {
            let key = arr[1].as_str().unwrap_or("");
            let expected = arr[2]
                .as_str()
                .map(|s| s.to_string())
                .unwrap_or_else(|| arr[2].as_f64().map(|n| n.to_string()).unwrap_or_default());
            props.get(key).map(|v| v != &expected).unwrap_or(true)
        }
        "has" if arr.len() >= 2 => {
            let key = arr[1].as_str().unwrap_or("");
            props.contains_key(key)
        }
        "!has" if arr.len() >= 2 => {
            let key = arr[1].as_str().unwrap_or("");
            !props.contains_key(key)
        }
        "in" if arr.len() >= 3 => {
            let key = arr[1].as_str().unwrap_or("");
            let Some(val) = props.get(key) else {
                return false;
            };
            arr[2..]
                .iter()
                .any(|v| v.as_str().map(|s| s == val).unwrap_or(false))
        }
        "!in" if arr.len() >= 3 => {
            let key = arr[1].as_str().unwrap_or("");
            let Some(val) = props.get(key) else {
                return true;
            };
            !arr[2..]
                .iter()
                .any(|v| v.as_str().map(|s| s == val).unwrap_or(false))
        }
        _ => {
            log::warn!("unsupported filter operator: {op}");
            true
        }
    }
}

/// Filter an MVT layer's features in-place according to a style filter expression.
fn apply_filter_to_layer(layer: &mut tile::Layer, filter: &serde_json::Value) {
    // Collect which features pass the filter (can't borrow layer immutably
    // inside retain because retain borrows features mutably).
    let keep: Vec<bool> = layer
        .features
        .iter()
        .map(|feature| {
            let props = resolve_feature_properties(layer, feature);
            evaluate_filter(filter, &props)
        })
        .collect();
    let mut idx = 0;
    layer.features.retain(|_| {
        let pass = keep[idx];
        idx += 1;
        pass
    });
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
            if let Some(layer) = tile
                .layers
                .iter_mut()
                .find(|layer| &layer.name == source_layer)
            {
                // Clone the layer so filtering doesn't affect other style layers
                // that reference the same source layer.
                let mut filtered_layer = layer.clone();

                // Apply style filter to exclude non-matching features
                if let Some(filter) = &style_layer.filter {
                    apply_filter_to_layer(&mut filtered_layer, filter);
                }

                let original_layer = filtered_layer.clone();
                let layer = &mut filtered_layer;

                match paint {
                    LayerPaint::Line(_) | LayerPaint::Fill(_) => {
                        let mut tessellator = ZeroTessellator::<IndexDataType>::default();
                        match paint {
                            LayerPaint::Fill(p) => {
                                tessellator.style_property = p.fill_color.clone()
                            }
                            LayerPaint::Line(p) => {
                                tessellator.style_property = p.line_color.clone();
                                tessellator.is_line_layer = true;
                            }
                            LayerPaint::Background(p) => {
                                tessellator.style_property = p.background_color.clone()
                            }
                            _ => {}
                        }

                        if let Err(e) = layer.process(&mut tessellator) {
                            context.layer_missing(coords, &source_layer)?;

                            tracing::error!("tesselation for layer source {source_layer} at {coords} failed {e:?}");
                        } else {
                            context.layer_tesselation_finished(
                                coords,
                                tessellator.buffer.into(),
                                tessellator.feature_indices,
                                tessellator.feature_colors,
                                original_layer,
                                id.clone(),
                            )?;
                        }
                    }
                    LayerPaint::Symbol(symbol_paint) => {
                        let mut tessellator = TextTessellator::<IndexDataType>::default();
                        let text_field = symbol_paint
                            .text_field
                            .clone()
                            .unwrap_or_else(|| "name".to_string());
                        let mut tessellator_new = TextTessellatorNew::new(text_field);

                        if let Err(e) = layer.process(&mut tessellator_new) {
                            context.layer_missing(coords, &source_layer)?;

                            tracing::error!("tesselation for layer source {source_layer} at {coords} failed {e:?}");
                        } else {
                            tessellator_new.finish();
                            context.symbol_layer_tesselation_finished(
                                coords,
                                tessellator.quad_buffer.into(),
                                tessellator_new.quad_buffer.into(),
                                tessellator_new.features,
                                original_layer,
                                id.clone(),
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

    for layer in tile_request.layers {
        if let Some(source_layer) = layer.source_layer {
            if !available_layers.contains(&source_layer) {
                context.layer_missing(coords, &source_layer)?;
                tracing::info!(
                    "requested source layer {source_layer} at {coords} not found in tile"
                );
            }
        }
    }

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
        feature_colors: Vec<[f32; 4]>,
        layer_data: tile::Layer,
        style_layer_id: String,
    ) -> Result<(), ProcessVectorError> {
        self.context
            .send_back(T::LayerTessellated::build_from(
                *coords,
                buffer,
                feature_indices,
                feature_colors,
                layer_data,
                style_layer_id,
            ))
            .map_err(|e| ProcessVectorError::SendError(e))
    }

    fn symbol_layer_tesselation_finished(
        &mut self,
        coords: &WorldTileCoords,
        buffer: OverAlignedVertexBuffer<ShaderSymbolVertex, IndexDataType>,
        new_buffer: OverAlignedVertexBuffer<ShaderSymbolVertexNew, IndexDataType>,
        features: Vec<Feature>,
        layer_data: tile::Layer,
        style_layer_id: String,
    ) -> Result<(), ProcessVectorError> {
        self.context
            .send_back(T::SymbolLayerTessellated::build_from(
                *coords,
                buffer,
                new_buffer,
                features,
                layer_data,
                style_layer_id,
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
