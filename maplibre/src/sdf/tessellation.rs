//! Tessellation for lines and polygons is implemented here.

use std::collections::HashMap;
use csscolorparser::Color;
use geozero::{ColumnValue, FeatureProcessor, GeomProcessor, PropertyProcessor};
use geozero::geo_types::GeoWriter;
use lyon::{
    geom::euclid::{Box2D, Point2D},
    tessellation::{
        geometry_builder::MaxIndex, BuffersBuilder, FillOptions, FillTessellator, VertexBuffers,
    },
};

use crate::{
    render::shaders::ShaderSymbolVertex,
    sdf::{
        text::{Anchor, Glyph, GlyphSet, SymbolVertexBuilder},
        Feature, TileSpace,
    },
};

const DEFAULT_TOLERANCE: f32 = 0.02;

/// Vertex buffers index data type.
pub type IndexDataType = u32; // Must match INDEX_FORMAT

type GeoResult<T> = geozero::error::Result<T>;

/// Build tessellations with vectors.
pub struct TextTessellator<I: std::ops::Add + From<lyon::tessellation::VertexId> + MaxIndex> {
    glyphs: GlyphSet,

    // output
    pub quad_buffer: VertexBuffers<ShaderSymbolVertex, I>,
    pub features: Vec<Feature>,

    // iteration variables
    current_index: usize,
    current_text: Option<String>,
    current_origin: Option<Box2D<f32, TileSpace>>,
}

impl<I: std::ops::Add + From<lyon::tessellation::VertexId> + MaxIndex> Default
    for TextTessellator<I>
{
    fn default() -> Self {
        let data = include_bytes!("../../../data/0-255.pbf");
        let glyphs = GlyphSet::try_from(data.as_slice()).unwrap();
        Self {
            glyphs,
            quad_buffer: VertexBuffers::new(),
            features: Vec::new(),
            current_index: 0,
            current_text: None,
            current_origin: None,
        }
    }
}

enum StringGlyph<'a> {
    Char(char),
    Glyph(&'a Glyph),
}

impl<I: std::ops::Add + From<lyon::tessellation::VertexId> + MaxIndex> TextTessellator<I> {
    pub fn tessellate_glyph_quads(
        &mut self,
        origin: [f32; 2],
        label_text: &str,
        color: Color,
    ) -> Option<Box2D<f32, TileSpace>> {
        let mut tessellator = FillTessellator::new();

        let mut next_origin = origin;

        let texture_dimensions = self.glyphs.get_texture_dimensions();
        let texture_dimensions = (texture_dimensions.0 as f32, texture_dimensions.1 as f32);

        // TODO: handle line wrapping / line height
        let mut bbox = None;
        for str_glyph in label_text
            .chars()
            .map(|c| {
                self.glyphs
                    .glyphs
                    .get(&c)
                    .map(|glyph| StringGlyph::Glyph(glyph))
                    .unwrap_or_else(|| StringGlyph::Char(c))
            })
            .collect::<Vec<_>>()
        {
            let glyph = match str_glyph {
                StringGlyph::Glyph(glyph) => glyph,
                StringGlyph::Char(c) => match c {
                    ' ' => {
                        next_origin[0] += 10.0; // Spaces are 10 units wide
                        continue;
                    }
                    _ => {
                        log::error!("unhandled char {}", c);
                        continue;
                    }
                },
            };

            let glyph_dims = glyph.buffered_dimensions();
            let width = glyph_dims.0 as f32;
            let height = glyph_dims.1 as f32;

            let glyph_anchor = [
                next_origin[0] + glyph.left_bearing as f32,
                next_origin[1] - glyph.top_bearing as f32,
                0.,
            ];

            let glyph_bbox = Box2D::new(
                (glyph_anchor[0], glyph_anchor[1]).into(),
                (glyph_anchor[0] + width, glyph_anchor[1] + height).into(),
            );

            bbox = bbox.map_or_else(
                || Some(glyph_bbox),
                |bbox: Box2D<_, TileSpace>| Some(bbox.union(&glyph_bbox)),
            );

            tessellator
                .tessellate_rectangle(
                    &glyph_bbox.cast_unit(),
                    &FillOptions::default(),
                    &mut BuffersBuilder::new(
                        &mut self.quad_buffer,
                        SymbolVertexBuilder {
                            glyph_anchor,
                            text_anchor: [origin[0], origin[1], 0.0],
                            texture_dimensions,
                            sprite_dimensions: (width, height),
                            sprite_offset: (
                                glyph.origin_offset().0 as f32,
                                glyph.origin_offset().1 as f32,
                            ),
                            color: color.to_rgba8(), // TODO: is this conversion oke?
                            glyph: true,             // Set here to true to use SDF rendering
                        },
                    ),
                )
                .ok()?;

            next_origin[0] += glyph.advance() as f32 + 1.0;
        }

        bbox
    }
}

impl<I: std::ops::Add + From<lyon::tessellation::VertexId> + MaxIndex> GeomProcessor
    for TextTessellator<I>
{
    fn xy(&mut self, x: f64, y: f64, _idx: usize) -> GeoResult<()> {
        if let Some(_) = self.current_origin {
            //FIXME
            unreachable!("Text labels have only a single origin point")
        } else {
            self.current_origin = Some(Box2D::new(
                Point2D::new(x as f32, y as f32),
                Point2D::new(x as f32, y as f32),
            ))
        }

        Ok(())
    }

    fn point_begin(&mut self, _idx: usize) -> GeoResult<()> {
        Ok(())
    }

    fn point_end(&mut self, _idx: usize) -> GeoResult<()> {
        Ok(())
    }

    fn multipoint_begin(&mut self, _size: usize, _idx: usize) -> GeoResult<()> {
        Ok(())
    }

    fn multipoint_end(&mut self, _idx: usize) -> GeoResult<()> {
        Ok(())
    }

    fn linestring_begin(&mut self, _tagged: bool, _size: usize, _idx: usize) -> GeoResult<()> {
        Ok(())
    }

    fn linestring_end(&mut self, _tagged: bool, _idx: usize) -> GeoResult<()> {
        Ok(())
    }

    fn multilinestring_begin(&mut self, _size: usize, _idx: usize) -> GeoResult<()> {
        Ok(())
    }

    fn multilinestring_end(&mut self, _idx: usize) -> GeoResult<()> {
        Ok(())
    }

    fn polygon_begin(&mut self, _tagged: bool, _size: usize, _idx: usize) -> GeoResult<()> {
        Ok(())
    }

    fn polygon_end(&mut self, _tagged: bool, _idx: usize) -> GeoResult<()> {
        Ok(())
    }

    fn multipolygon_begin(&mut self, _size: usize, _idx: usize) -> GeoResult<()> {
        Ok(())
    }

    fn multipolygon_end(&mut self, _idx: usize) -> GeoResult<()> {
        Ok(())
    }
}

impl<I: std::ops::Add + From<lyon::tessellation::VertexId> + MaxIndex> PropertyProcessor
    for TextTessellator<I>
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
    for TextTessellator<I>
{
    fn feature_end(&mut self, _idx: u64) -> geozero::error::Result<()> {
        if let (Some(origin), Some(text)) = (&self.current_origin, self.current_text.clone()) {
            if text.is_empty() {
                panic!("dud")
            }
            let anchor = Anchor::BottomLeft;
            // TODO: add more anchor possibilities
            let origin = match anchor {
                Anchor::Center => origin.center(), // FIXME: origin is currently always a point
                Anchor::BottomLeft => origin.min,
                _ => unimplemented!("no support for this anchor"),
            };
            let bbox = self.tessellate_glyph_quads(
                origin.to_array(),
                text.as_str(),
                Color::from_linear_rgba(1.0, 0., 0., 1.),
            );

            let next_index = self.quad_buffer.indices.len();
            let start = self.current_index;
            let end = next_index;
            self.current_index = next_index;

            self.features.push(Feature {
                bbox: bbox.unwrap_or(Box2D::new(origin, origin)),
                indices: start..end,
                text_anchor: origin.cast(),
                str: text,
            });

            self.current_origin = None;
            self.current_text = None;
        }
        Ok(())
    }
}
