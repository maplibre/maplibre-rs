use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    marker::PhantomData,
};

use geozero::{
    mvt::{tile, Message},
    GeozeroDatasource,
};
use lyon::tessellation::VertexBuffers;
use thiserror::Error;

use crate::{
    coords::WorldTileCoords,
    euclid::{Point2D, Rect, Size2D},
    io::{
        apc::{Context, SendError},
        geometry_index::{IndexProcessor, IndexedGeometry, TileIndex},
    },
    legacy::{
        bidi::Char16,
        buckets::symbol_bucket::SymbolBucketBuffer,
        font_stack::FontStackHasher,
        geometry_tile_data::{GeometryCoordinates, SymbolGeometryTileLayer},
        glyph::{Glyph, GlyphDependencies, GlyphMap, GlyphMetrics, Glyphs},
        glyph_atlas::{GlyphPosition, GlyphPositionMap, GlyphPositions},
        image::ImageMap,
        image_atlas::ImagePositions,
        layout::{
            layout::{BucketParameters, LayerTypeInfo, LayoutParameters},
            symbol_feature::{SymbolGeometryTileFeature, VectorGeometryTileFeature},
            symbol_layout::{FeatureIndex, LayerProperties, SymbolLayer, SymbolLayout},
        },
        style_types::SymbolLayoutProperties_Unevaluated,
        tagged_string::SectionOptions,
        CanonicalTileID, MapMode, OverscaledTileID,
    },
    render::{
        shaders::{ShaderSymbolVertex, ShaderSymbolVertexNew},
        ShaderVertex,
    },
    sdf::{tessellation::TextTessellator, text::GlyphSet, Feature},
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
                        let data = include_bytes!("../../../data/0-255.pbf");
                        let glyphs = GlyphSet::try_from(data.as_slice()).unwrap();

                        let font_stack = vec![
                            "Open Sans Regular".to_string(),
                            "Arial Unicode MS Regular".to_string(),
                        ];

                        let layer_name = "layer".to_string();

                        let section_options = SectionOptions::new(1.0, font_stack.clone(), None);

                        let mut glyph_dependencies = GlyphDependencies::new();

                        let tile_id = OverscaledTileID {
                            canonical: CanonicalTileID { x: 0, y: 0, z: 0 },
                            overscaled_z: 0,
                        };
                        let mut parameters = BucketParameters {
                            tile_id: tile_id,
                            mode: MapMode::Continuous,
                            pixel_ratio: 1.0,
                            layer_type: LayerTypeInfo,
                        };
                        let layer_data = SymbolGeometryTileLayer {
                            name: layer_name.clone(),
                            features: vec![SymbolGeometryTileFeature::new(Box::new(
                                VectorGeometryTileFeature {
                                    geometry: vec![GeometryCoordinates(vec![Point2D::new(
                                        512, 512,
                                    )])],
                                },
                            ))],
                        };
                        let layer_properties = vec![LayerProperties {
                            id: layer_name.clone(),
                            layer: SymbolLayer {
                                layout: SymbolLayoutProperties_Unevaluated,
                            },
                        }];

                        let image_positions = ImagePositions::new();

                        let glyph_map = GlyphPositionMap::from_iter(glyphs.glyphs.iter().map(
                            |(unicode_point, glyph)| {
                                (
                                    *unicode_point as Char16,
                                    GlyphPosition {
                                        rect: Rect::new(
                                            Point2D::new(
                                                glyph.tex_origin_x as u16 + 3,
                                                glyph.tex_origin_y as u16 + 3,
                                            ),
                                            Size2D::new(
                                                glyph.buffered_dimensions().0 as u16,
                                                glyph.buffered_dimensions().1 as u16,
                                            ),
                                        ), // FIXME: verify if this mapping is correct
                                        metrics: GlyphMetrics {
                                            width: glyph.width,
                                            height: glyph.height,
                                            left: glyph.left_bearing,
                                            top: glyph.top_bearing,
                                            advance: glyph.h_advance,
                                        },
                                    },
                                )
                            },
                        ));

                        let glyph_positions: GlyphPositions =
                            GlyphPositions::from([(FontStackHasher::new(&font_stack), glyph_map)]);

                        let glyphs: GlyphMap = GlyphMap::from([(
                            FontStackHasher::new(&font_stack),
                            Glyphs::from_iter(glyphs.glyphs.iter().map(
                                |(unicode_point, glyph)| {
                                    (
                                        *unicode_point as Char16,
                                        Some(Glyph {
                                            id: *unicode_point as Char16,
                                            bitmap: Default::default(),
                                            metrics: GlyphMetrics {
                                                width: glyph.width,
                                                height: glyph.height,
                                                left: glyph.left_bearing,
                                                top: glyph.top_bearing,
                                                advance: glyph.h_advance,
                                            },
                                        }),
                                    )
                                },
                            )),
                        )]);

                        let mut layout = SymbolLayout::new(
                            &parameters,
                            &layer_properties,
                            Box::new(layer_data),
                            &mut LayoutParameters {
                                bucket_parameters: &mut parameters.clone(),
                                glyph_dependencies: &mut glyph_dependencies,
                                image_dependencies: &mut Default::default(),
                                available_images: &mut Default::default(),
                            },
                        )
                        .unwrap();

                        assert_eq!(glyph_dependencies.len(), 1);

                        let empty_image_map = ImageMap::new();
                        layout.prepare_symbols(
                            &glyphs,
                            &glyph_positions,
                            &empty_image_map,
                            &image_positions,
                        );

                        let mut output = HashMap::new();
                        layout.create_bucket(
                            image_positions,
                            Box::new(FeatureIndex),
                            &mut output,
                            false,
                            false,
                            &tile_id.canonical,
                        );

                        let new_buffer = output.remove(&layer_name).unwrap();

                        let mut buffer = VertexBuffers::new();
                        let text_buffer = new_buffer.bucket.text;
                        let SymbolBucketBuffer {
                            shared_vertices,
                            triangles,
                            ..
                        } = text_buffer;
                        buffer.vertices = shared_vertices
                            .iter()
                            .map(|v| ShaderSymbolVertexNew::new(v))
                            .collect();
                        buffer.indices = triangles.indices.iter().map(|i| *i as u32).collect();

                        // TODO
                        let mut tessellator = TextTessellator::<IndexDataType>::default();

                        //if let Err(e) = layer.process(&mut tessellator) {
                        if let Err(e) = Ok::<(), ProcessVectorError>(()) {
                            context.layer_missing(coords, &source_layer)?;

                            tracing::error!("tesselation for layer source {source_layer} at {coords} failed {e:?}");
                        } else {
                            context.symbol_layer_tesselation_finished(
                                coords,
                                tessellator.quad_buffer.into(),
                                buffer.into(),
                                tessellator.features,
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
        buffer: OverAlignedVertexBuffer<ShaderSymbolVertex, IndexDataType>,
        new_buffer: OverAlignedVertexBuffer<ShaderSymbolVertexNew, IndexDataType>,
        features: Vec<Feature>,
        layer_data: tile::Layer,
    ) -> Result<(), ProcessVectorError> {
        self.context
            .send(T::SymbolLayerTessellated::build_from(
                *coords, buffer, new_buffer, features, layer_data,
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
