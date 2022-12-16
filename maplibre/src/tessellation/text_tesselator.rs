use std::fs;

use csscolorparser::Color;
use geozero::{ColumnValue, FeatureProcessor, GeomProcessor, PropertyProcessor};
use lyon::{
    geom::{euclid::Point2D, Box2D},
    tessellation::{
        geometry_builder::MaxIndex, BuffersBuilder, FillOptions, FillTessellator, VertexBuffers,
    },
};
use prost::Message;

use crate::{
    render::SymbolVertex,
    text::{glyph::GlyphSet, sdf_glyphs::Glyphs, Anchor, SymbolVertexBuilder},
};

type GeoResult<T> = geozero::error::Result<T>;

/// Build tessellations with vectors.
pub struct TextTessellator<I: std::ops::Add + From<lyon::tessellation::VertexId> + MaxIndex> {
    pub quad_buffer: VertexBuffers<SymbolVertex, I>,

    pub feature_indices: Vec<u32>,

    current_index: usize,
    current_text: Option<String>,
    current_bbox: Option<Box2D<f32>>,
}

impl<I: std::ops::Add + From<lyon::tessellation::VertexId> + MaxIndex> Default
    for TextTessellator<I>
{
    fn default() -> Self {
        Self {
            quad_buffer: VertexBuffers::new(),
            feature_indices: Vec::new(),
            current_index: 0,
            current_text: None,
            current_bbox: None,
        }
    }
}

impl<I: std::ops::Add + From<lyon::tessellation::VertexId> + MaxIndex> TextTessellator<I> {
    pub fn tessellate_glyph_quads(
        &mut self,
        origin: [f32; 2],
        glyphs: &GlyphSet,
        font_size: f32,
        label_text: &str,
        zoom: f32,
        color: Color,
    ) -> Option<Box2D<f32>> {
        let mut tessellator = FillTessellator::new();

        let font_scale = font_size / 24.;
        let m_p_px = meters_per_pixel(zoom.floor()) * font_scale;
        let m_p_px = 6.0;

        let mut next_glyph_origin = origin;

        let texture_dimensions = glyphs.get_texture_dimensions();
        let texture_dimensions = (
            texture_dimensions.0 as f32 * m_p_px,
            texture_dimensions.1 as f32 * m_p_px,
        );

        // TODO: silently drops unknown characters
        // TODO: handle line wrapping / line height
        let mut bbox = None;
        for glyph in label_text
            .chars()
            .filter_map(|c| glyphs.glyphs.get(&c))
            .collect::<Vec<_>>()
        {
            let glyph_dims = glyph.buffered_dimensions();
            let meter_width = glyph_dims.0 as f32 * m_p_px;
            let meter_height = glyph_dims.1 as f32 * m_p_px;

            let anchor = [
                next_glyph_origin[0] + glyph.left_bearing as f32 * m_p_px,
                next_glyph_origin[1] - meter_height + glyph.top_bearing as f32 * m_p_px,
                0.,
            ];

            let glyph_rect = Box2D::new(
                (anchor[0], anchor[1]).into(),
                (anchor[0] + meter_width, anchor[1] + meter_height).into(),
            );
            //let glyph_rect = Box2D::new((0.0, 0.0).into(), (100.0, 100.0).into());

            bbox = bbox.map_or_else(
                || Some(glyph_rect),
                |bbox: Box2D<_>| Some(bbox.union(&glyph_rect)),
            );

            tessellator
                .tessellate_rectangle(
                    &glyph_rect,
                    &FillOptions::default(),
                    &mut BuffersBuilder::new(
                        &mut self.quad_buffer,
                        SymbolVertexBuilder {
                            anchor,
                            texture_dimensions,
                            sprite_dimensions: (meter_width, meter_height),
                            sprite_offset: (
                                glyph.origin_offset().0 as f32 * m_p_px,
                                glyph.origin_offset().1 as f32 * m_p_px,
                            ),
                            color: color.to_rgba8(), // TODO: is this conversion oke?
                            glyph: true,             // Set here to true to use SDF rendering
                        },
                    ),
                )
                .ok()?;

            next_glyph_origin[0] += glyph.advance() as f32 * m_p_px;
        }

        bbox
    }
}

impl<I: std::ops::Add + From<lyon::tessellation::VertexId> + MaxIndex> GeomProcessor
    for TextTessellator<I>
{
    fn xy(&mut self, x: f64, y: f64, _idx: usize) -> GeoResult<()> {
        let new_box = Box2D::new(
            Point2D::new(x as f32, y as f32),
            Point2D::new(x as f32, y as f32),
        );
        if let Some(bbox) = self.current_bbox {
            self.current_bbox = Some(bbox.union(&new_box))
        } else {
            self.current_bbox = Some(new_box)
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

    fn linestring_end(&mut self, tagged: bool, _idx: usize) -> GeoResult<()> {
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

    fn polygon_end(&mut self, tagged: bool, _idx: usize) -> GeoResult<()> {
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
        idx: usize,
        name: &str,
        value: &ColumnValue,
    ) -> geozero::error::Result<bool> {
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
        if let (Some(bbox), Some(text)) = (&self.current_bbox, self.current_text.clone()) {
            let anchor = Anchor::Center;
            // TODO: add more anchor possibilities; only support center right now
            // TODO: document how anchor and glyph metrics work together to establish a baseline
            let origin = match anchor {
                Anchor::Center => bbox.center().to_array(),
                _ => unimplemented!("no support for this anchor"),
            };
            let data = fs::read("./data/0-255.pbf").unwrap();
            let glyphs = GlyphSet::from(Glyphs::decode(data.as_slice()).unwrap());
            self.tessellate_glyph_quads(
                origin,
                &glyphs,
                16.0,
                text.as_str(),
                10.0,
                Color::from_linear_rgba(1.0, 0., 0., 1.),
            );

            let next_index = self.quad_buffer.indices.len();
            let indices = (next_index - self.current_index) as u32;
            self.feature_indices.push(indices);
            self.current_index = next_index;
        }

        self.current_bbox = None;
        self.current_text = None;
        Ok(())
    }
}

#[inline]
pub fn tile_scale_for_zoom(zoom: f32) -> f32 {
    const MERCATOR_RADIUS: f32 = 6378137.; // in meters
    const MERCATOR_RADIUS_PI: f32 = MERCATOR_RADIUS * std::f32::consts::PI;
    const MERCATOR_RADIUS_2PI: f32 = 2. * MERCATOR_RADIUS_PI;
    // Each zoom should show Mercator points 2x larger than the previous zoom
    // level.
    MERCATOR_RADIUS_2PI / 2f32.powf(zoom)
}

#[inline]
pub fn meters_per_pixel(zoom: f32) -> f32 {
    tile_scale_for_zoom(zoom) / 512.0 // FIXME
}
