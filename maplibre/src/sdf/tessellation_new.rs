//! Tessellation for lines and polygons is implemented here.

use std::collections::HashMap;

use geo_types::Geometry;
use geozero::{
    geo_types::GeoWriter, ColumnValue, FeatureProcessor, GeomProcessor, PropertyProcessor,
};
use lyon::{
    geom::euclid::{Box2D, Point2D},
    tessellation::{VertexBuffers},
};

use crate::{
    euclid::{Rect, Size2D},
    legacy::{
        bidi::Char16,
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
        CanonicalTileID, MapMode, OverscaledTileID, TileSpace,
    },
    render::shaders::ShaderSymbolVertexNew,
    sdf::Feature,
};
use crate::legacy::buckets::symbol_bucket::SymbolBucketBuffer;
use crate::legacy::tagged_string::SectionOptions;
use crate::sdf::tessellation::IndexDataType;
use crate::sdf::text::GlyphSet;

type GeoResult<T> = geozero::error::Result<T>;

/// Build tessellations with vectors.
pub struct TextTessellatorNew {
    geo_writer: GeoWriter,

    // output
    pub quad_buffer: VertexBuffers<ShaderSymbolVertexNew, IndexDataType>,
    pub features: Vec<Feature>,

    // iteration variables
    current_index: usize,
    current_text: Option<String>,
    current_origin: Option<Box2D<f32, TileSpace>>,
}

impl TextTessellatorNew {
    pub fn finish(&mut self) {
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

        self.quad_buffer = buffer;
    }
}

impl Default for TextTessellatorNew {
    fn default() -> Self {
        Self {
            geo_writer: Default::default(),
            quad_buffer: VertexBuffers::new(),
            features: vec![],
            current_index: 0,
            current_text: None,
            current_origin: None,
        }
    }
}

impl GeomProcessor for TextTessellatorNew {
    fn xy(&mut self, x: f64, y: f64, idx: usize) -> GeoResult<()> {
        self.geo_writer.xy(x, y, idx)
    }
    fn point_begin(&mut self, idx: usize) -> GeoResult<()> {
        self.geo_writer.point_begin(idx)
    }
    fn point_end(&mut self, idx: usize) -> GeoResult<()> {
        self.geo_writer.point_end(idx)
    }
    fn multipoint_begin(&mut self, size: usize, idx: usize) -> GeoResult<()> {
        self.geo_writer.multipoint_begin(size, idx)
    }
    fn linestring_begin(&mut self, tagged: bool, size: usize, idx: usize) -> GeoResult<()> {
        self.geo_writer.linestring_begin(tagged, size, idx)
    }
    fn linestring_end(&mut self, tagged: bool, idx: usize) -> GeoResult<()> {
        self.geo_writer.linestring_end(tagged, idx)
    }
    fn multilinestring_begin(&mut self, size: usize, idx: usize) -> GeoResult<()> {
        self.geo_writer.multilinestring_begin(size, idx)
    }
    fn multilinestring_end(&mut self, idx: usize) -> GeoResult<()> {
        self.geo_writer.multilinestring_end(idx)
    }
    fn polygon_begin(&mut self, tagged: bool, size: usize, idx: usize) -> GeoResult<()> {
        self.geo_writer.polygon_begin(tagged, size, idx)
    }
    fn polygon_end(&mut self, tagged: bool, idx: usize) -> GeoResult<()> {
        self.geo_writer.polygon_end(tagged, idx)
    }
    fn multipolygon_begin(&mut self, size: usize, idx: usize) -> GeoResult<()> {
        self.geo_writer.multipolygon_begin(size, idx)
    }
    fn multipolygon_end(&mut self, idx: usize) -> GeoResult<()> {
        self.geo_writer.multipolygon_end(idx)
    }
}

impl PropertyProcessor
    for TextTessellatorNew
{
    fn property(
        &mut self,
        _idx: usize,
        name: &str,
        value: &ColumnValue,
    ) -> geozero::error::Result<bool> {
        // TODO: Support different tags
        if name == "name" {
            match value {
                ColumnValue::String(str) => {
                    self.current_text = Some(str.to_string());
                }
                _ => {}
            }
        }
        Ok(true)
    }
}

impl FeatureProcessor
    for TextTessellatorNew
{
    fn feature_end(&mut self, _idx: u64) -> geozero::error::Result<()> {
        let geometry = self.geo_writer.take_geometry();

        match geometry {
            Some(Geometry::Point(_point)) => {}
            Some(Geometry::Polygon(_polygon)) => {}
            Some(Geometry::LineString(_linestring)) => {}
            Some(Geometry::Line(_))
            | Some(Geometry::MultiPoint(_))
            | Some(Geometry::MultiLineString(_))
            | Some(Geometry::MultiPolygon(_))
            | Some(Geometry::GeometryCollection(_))
            | Some(Geometry::Rect(_))
            | Some(Geometry::Triangle(_)) => {
                log::debug!("Unsupported geometry in text tesselation")
            }
            None => {
                log::debug!("No geometry in feature")
            }
        };

        Ok(())
    }
}
