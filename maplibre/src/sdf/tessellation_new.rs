//! Tessellation for lines and polygons is implemented here.

use std::collections::HashMap;
use csscolorparser::Color;
use geo_types::Geometry;
use geozero::{ColumnValue, FeatureProcessor, GeomProcessor, PropertyProcessor};
use geozero::geo_types::GeoWriter;
use lyon::{
    geom::euclid::{Box2D, Point2D},
    tessellation::{
        geometry_builder::MaxIndex, VertexBuffers,
    },
};

use crate::{
    sdf::{
        text::{Anchor, Glyph, GlyphSet, SymbolVertexBuilder},
        Feature, TileSpace,
    },
};
use crate::euclid::{Rect, Size2D};
use crate::io::geometry_index::IndexedGeometry;
use crate::render::shaders::ShaderSymbolVertexNew;
use crate::sdf::bidi::Char16;
use crate::sdf::{CanonicalTileID, MapMode, OverscaledTileID};
use crate::sdf::font_stack::FontStackHasher;
use crate::sdf::geometry_tile_data::{GeometryCoordinates, SymbolGeometryTileLayer};
use crate::sdf::glyph::{GlyphDependencies, GlyphMap, GlyphMetrics};
use crate::sdf::glyph_atlas::{GlyphAtlas, GlyphPosition, GlyphPositionMap, GlyphPositions};
use crate::sdf::image::ImageMap;
use crate::sdf::image_atlas::ImagePositions;
use crate::sdf::layout::layout::{BucketParameters, LayerTypeInfo, LayoutParameters};
use crate::sdf::layout::symbol_feature::{SymbolGeometryTileFeature, VectorGeometryTileFeature};
use crate::sdf::layout::symbol_layout::{FeatureIndex, LayerProperties, SymbolLayer, SymbolLayout};
use crate::sdf::style_types::SymbolLayoutProperties_Unevaluated;


/// Vertex buffers index data type.
pub type IndexDataType = u32; // Must match INDEX_FORMAT

type GeoResult<T> = geozero::error::Result<T>;

/// Build tessellations with vectors.
pub struct TextTessellatorNew<I> {
    geo_writer: GeoWriter,

    // output
    pub quad_buffer: VertexBuffers<ShaderSymbolVertexNew, I>,
    pub features: Vec<Feature>,

    // iteration variables
    current_index: usize,
    current_text: Option<String>,
    current_origin: Option<Box2D<f32, TileSpace>>,
}

impl<I> TextTessellatorNew<I> {
    pub fn finish() {

        let fontStack = vec![
            "Open Sans Regular".to_string(),
            "Arial Unicode MS Regular".to_string(),
        ];

        // load glyph/image data

        let image_positions = ImagePositions::new();

        let mut glyphPosition = GlyphPosition {
            rect: Rect::new(Point2D::new(0, 0), Size2D::new(10, 10)),
            metrics: GlyphMetrics {
                width: 18,
                height: 18,
                left: 2,
                top: -8,
                advance: 21,
            },
        };
        let glyphPositions: GlyphPositions = GlyphPositions::from([(
            FontStackHasher::new(&fontStack),
            GlyphPositionMap::from([('中' as Char16, glyphPosition)]),
        )]);

        let mut glyph = Glyph::default();
        glyph.id = '中' as Char16;
        glyph.metrics = glyphPosition.metrics;

        let glyphs: GlyphMap = GlyphMap::from([(
            FontStackHasher::new(&fontStack),
            Glyphs::from([('中' as Char16, Some(glyph))]),
        )]);

        let empty_image_map = ImageMap::new();

        // layouting


        let mut glyphDependencies = GlyphDependencies::new();

        let tile_id = OverscaledTileID {
            canonical: CanonicalTileID { x: 0, y: 0, z: 0 },
            overscaledZ: 0,
        };
        let mut parameters = BucketParameters {
            tileID: tile_id,
            mode: MapMode::Continuous,
            pixelRatio: 1.0,
            layerType: LayerTypeInfo,
        };
        let mut layout = SymbolLayout::new(
            &parameters,
            &vec![LayerProperties {
                id: "layer".to_string(),
                layer: SymbolLayer {
                    layout: SymbolLayoutProperties_Unevaluated,
                },
            }],
            Box::new(SymbolGeometryTileLayer {
                name: "layer".to_string(),
                features: vec![SymbolGeometryTileFeature::new(Box::new(
                    VectorGeometryTileFeature {
                        geometry: vec![GeometryCoordinates(vec![Point2D::new(1024, 1024)])],
                    },
                ))],
            }),
            &mut LayoutParameters {
                bucketParameters: &mut parameters.clone(),
                glyphDependencies: &mut glyphDependencies,
                imageDependencies: &mut Default::default(),
                availableImages: &mut Default::default(),
            },
        ).unwrap();

        layout.prepareSymbols(&glyphs, &glyphPositions, &empty_image_map, &image_positions);

        let mut output = HashMap::new();
        layout.createBucket(
            image_positions,
            Box::new(FeatureIndex),
            &mut output,
            false,
            false,
            &tile_id.canonical,
        );

    }
}

impl<I> Default for TextTessellatorNew<I>
{
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

impl<I> GeomProcessor for TextTessellatorNew<I> {
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
    fn linestring_begin(
        &mut self,
        tagged: bool,
        size: usize,
        idx: usize,
    ) -> GeoResult<()> {
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

impl<I: std::ops::Add + From<lyon::tessellation::VertexId> + MaxIndex> PropertyProcessor
for TextTessellatorNew<I>
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

impl<I: std::ops::Add + From<lyon::tessellation::VertexId> + MaxIndex> FeatureProcessor
for TextTessellatorNew<I>
{
    fn feature_end(&mut self, _idx: u64) -> geozero::error::Result<()> {
        let geometry = self.geo_writer.take_geometry();

        match geometry {
            Some(Geometry::Point(point)) => self.geometries.push(

            ),
            Some(Geometry::Polygon(polygon)) => self.geometries.push(

            ),
            Some(Geometry::LineString(linestring)) => self.geometries.push(

            ),
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


    }
}