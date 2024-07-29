//! Tessellation for lines and polygons is implemented here.

use std::cell::RefCell;

use geozero::{FeatureProcessor, GeomProcessor, PropertyProcessor};
use lyon::{
    geom,
    path::{path::Builder, Path},
    tessellation::{
        geometry_builder::MaxIndex, BuffersBuilder, FillOptions, FillRule, FillTessellator,
        StrokeOptions, StrokeTessellator,
    },
};
use std::fs;

use csscolorparser::Color;
use geozero::{ColumnValue};
use lyon::{
    geom::{euclid::Point2D, Box2D},

};

use crate::{
    render::ShaderVertex,
};

use bytemuck::Pod;
use lyon::tessellation::{
    FillVertex, FillVertexConstructor, StrokeVertex, StrokeVertexConstructor, VertexBuffers,
};
use crate::render::shaders::SymbolVertex;
use crate::vector::text::{Anchor, GlyphSet, SymbolVertexBuilder};

const DEFAULT_TOLERANCE: f32 = 0.02;

/// Vertex buffers index data type.
pub type IndexDataType = u32; // Must match INDEX_FORMAT

/// Constructor for Fill and Stroke vertices.
pub struct VertexConstructor {}

impl FillVertexConstructor<ShaderVertex> for VertexConstructor {
    fn new_vertex(&mut self, vertex: FillVertex) -> ShaderVertex {
        ShaderVertex::new(vertex.position().to_array(), [0.0, 0.0])
    }
}

impl StrokeVertexConstructor<ShaderVertex> for VertexConstructor {
    fn new_vertex(&mut self, vertex: StrokeVertex) -> ShaderVertex {
        ShaderVertex::new(
            vertex.position_on_path().to_array(),
            vertex.normal().to_array(),
        )
    }
}

/// Vertex buffer which includes additional padding to fulfill the `wgpu::COPY_BUFFER_ALIGNMENT`.
#[derive(Clone)]
pub struct OverAlignedVertexBuffer<V, I> {
    pub buffer: VertexBuffers<V, I>,
    pub usable_indices: u32,
}

impl<V, I> OverAlignedVertexBuffer<V, I> {
    pub fn empty() -> Self {
        Self {
            buffer: VertexBuffers::with_capacity(0, 0),
            usable_indices: 0,
        }
    }

    pub fn from_iters<IV, II>(vertices: IV, indices: II, usable_indices: u32) -> Self
    where
        IV: IntoIterator<Item=V>,
        II: IntoIterator<Item=I>,
        IV::IntoIter: ExactSizeIterator,
        II::IntoIter: ExactSizeIterator,
    {
        let vertices = vertices.into_iter();
        let indices = indices.into_iter();
        let mut buffers = VertexBuffers::with_capacity(vertices.len(), indices.len());
        buffers.vertices.extend(vertices);
        buffers.indices.extend(indices);
        Self {
            buffer: buffers,
            usable_indices,
        }
    }
}

impl<V: Pod, I: Pod> From<VertexBuffers<V, I>> for OverAlignedVertexBuffer<V, I> {
    fn from(mut buffer: VertexBuffers<V, I>) -> Self {
        let usable_indices = buffer.indices.len() as u32;
        buffer.align_vertices();
        buffer.align_indices();
        Self {
            buffer,
            usable_indices,
        }
    }
}

trait Align<V: Pod, I: Pod> {
    fn align_vertices(&mut self);
    fn align_indices(&mut self);
}

impl<V: Pod, I: Pod> Align<V, I> for VertexBuffers<V, I> {
    fn align_vertices(&mut self) {
        let align = wgpu::COPY_BUFFER_ALIGNMENT;
        let stride = std::mem::size_of::<ShaderVertex>() as wgpu::BufferAddress;
        let unpadded_bytes = self.vertices.len() as wgpu::BufferAddress * stride;
        let padding_bytes = (align - unpadded_bytes % align) % align;

        if padding_bytes != 0 {
            panic!(
                "vertices are always aligned to wgpu::COPY_BUFFER_ALIGNMENT \
                    because GpuVertexUniform is aligned"
            )
        }
    }

    fn align_indices(&mut self) {
        let align = wgpu::COPY_BUFFER_ALIGNMENT;
        let stride = std::mem::size_of::<I>() as wgpu::BufferAddress;
        let unpadded_bytes = self.indices.len() as wgpu::BufferAddress * stride;
        let padding_bytes = (align - unpadded_bytes % align) % align;
        let overpad = (padding_bytes + stride - 1) / stride; // Divide by stride but round up

        for _ in 0..overpad {
            self.indices.push(I::zeroed());
        }
    }
}


type GeoResult<T> = geozero::error::Result<T>;

/// Build tessellations with vectors.
pub struct ZeroTessellator<I: std::ops::Add + From<lyon::tessellation::VertexId> + MaxIndex> {
    path_builder: RefCell<Builder>,
    path_open: bool,
    is_point: bool,

    pub buffer: VertexBuffers<ShaderVertex, I>,

    pub feature_indices: Vec<u32>,
    current_index: usize,
}

impl<I: std::ops::Add + From<lyon::tessellation::VertexId> + MaxIndex> Default
for ZeroTessellator<I>
{
    fn default() -> Self {
        Self {
            path_builder: RefCell::new(Path::builder()),
            buffer: VertexBuffers::new(),
            feature_indices: Vec::new(),
            current_index: 0,
            path_open: false,
            is_point: false,
        }
    }
}

impl<I: std::ops::Add + From<lyon::tessellation::VertexId> + MaxIndex> ZeroTessellator<I> {
    /// Stores current indices to the output. That way we know which vertices correspond to which
    /// feature in the output.
    fn update_feature_indices(&mut self) {
        let next_index = self.buffer.indices.len();
        let indices = (next_index - self.current_index) as u32;
        self.feature_indices.push(indices);
        self.current_index = next_index;
    }

    fn tessellate_strokes(&mut self) {
        let path_builder = self.path_builder.replace(Path::builder());

        StrokeTessellator::new()
            .tessellate_path(
                &path_builder.build(),
                &StrokeOptions::tolerance(DEFAULT_TOLERANCE),
                &mut BuffersBuilder::new(&mut self.buffer, VertexConstructor {}),
            )
            .unwrap(); // TODO: Remove unwrap
    }

    fn end(&mut self, close: bool) {
        if self.path_open {
            self.path_builder.borrow_mut().end(close);
            self.path_open = false;
        }
    }

    fn tessellate_fill(&mut self) {
        let path_builder = self.path_builder.replace(Path::builder());

        FillTessellator::new()
            .tessellate_path(
                &path_builder.build(),
                &FillOptions::tolerance(DEFAULT_TOLERANCE).with_fill_rule(FillRule::NonZero),
                &mut BuffersBuilder::new(&mut self.buffer, VertexConstructor {}),
            )
            .unwrap(); // TODO: Remove unwrap
    }
}

impl<I: std::ops::Add + From<lyon::tessellation::VertexId> + MaxIndex> GeomProcessor
for ZeroTessellator<I>
{
    fn xy(&mut self, x: f64, y: f64, _idx: usize) -> GeoResult<()> {
        // log::info!("xy");

        if self.is_point {
            // log::info!("point");
        } else if !self.path_open {
            self.path_builder
                .borrow_mut()
                .begin(geom::point(x as f32, y as f32));
            self.path_open = true;
        } else {
            self.path_builder
                .borrow_mut()
                .line_to(geom::point(x as f32, y as f32));
        }
        Ok(())
    }

    fn point_begin(&mut self, _idx: usize) -> GeoResult<()> {
        // log::info!("point_begin");
        self.is_point = true;
        Ok(())
    }

    fn point_end(&mut self, _idx: usize) -> GeoResult<()> {
        // log::info!("point_end");
        self.is_point = false;
        Ok(())
    }

    fn multipoint_begin(&mut self, _size: usize, _idx: usize) -> GeoResult<()> {
        // log::info!("multipoint_begin");
        Ok(())
    }

    fn multipoint_end(&mut self, _idx: usize) -> GeoResult<()> {
        // log::info!("multipoint_end");
        Ok(())
    }

    fn linestring_begin(&mut self, _tagged: bool, _size: usize, _idx: usize) -> GeoResult<()> {
        // log::info!("linestring_begin");
        Ok(())
    }

    fn linestring_end(&mut self, tagged: bool, _idx: usize) -> GeoResult<()> {
        // log::info!("linestring_end");

        self.end(false);

        if tagged {
            self.tessellate_strokes();
        }
        Ok(())
    }

    fn multilinestring_begin(&mut self, _size: usize, _idx: usize) -> GeoResult<()> {
        // log::info!("multilinestring_begin");
        Ok(())
    }

    fn multilinestring_end(&mut self, _idx: usize) -> GeoResult<()> {
        // log::info!("multilinestring_end");
        self.tessellate_strokes();
        Ok(())
    }

    fn polygon_begin(&mut self, _tagged: bool, _size: usize, _idx: usize) -> GeoResult<()> {
        // log::info!("polygon_begin");
        Ok(())
    }

    fn polygon_end(&mut self, tagged: bool, _idx: usize) -> GeoResult<()> {
        // log::info!("polygon_end");

        self.end(true);
        if tagged {
            self.tessellate_fill();
        }
        Ok(())
    }

    fn multipolygon_begin(&mut self, _size: usize, _idx: usize) -> GeoResult<()> {
        // log::info!("multipolygon_begin");
        Ok(())
    }

    fn multipolygon_end(&mut self, _idx: usize) -> GeoResult<()> {
        // log::info!("multipolygon_end");

        self.tessellate_fill();
        Ok(())
    }
}

impl<I: std::ops::Add + From<lyon::tessellation::VertexId> + MaxIndex> PropertyProcessor
for ZeroTessellator<I>
{}

impl<I: std::ops::Add + From<lyon::tessellation::VertexId> + MaxIndex> FeatureProcessor
for ZeroTessellator<I>
{
    fn feature_end(&mut self, _idx: u64) -> geozero::error::Result<()> {
        self.update_feature_indices();
        Ok(())
    }
}

/// Build tessellations with vectors.
pub struct TextTessellator<I: std::ops::Add + From<lyon::tessellation::VertexId> + MaxIndex> {
    glyphs: GlyphSet,

    // output
    pub quad_buffer: VertexBuffers<SymbolVertex, I>,
    pub feature_indices: Vec<u32>,

    // iteration variables
    current_index: usize,
    current_text: Option<String>,
    current_bbox: Option<Box2D<f32>>,
}

impl<I: std::ops::Add + From<lyon::tessellation::VertexId> + MaxIndex> Default
for TextTessellator<I>
{
    fn default() -> Self {
        let data = fs::read("./data/0-255.pbf").unwrap();
        let glyphs = GlyphSet::try_from(data.as_slice()).unwrap();
        Self {
            glyphs,
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
        label_text: &str,
        color: Color,
    ) -> Option<Box2D<f32>> {
        let mut tessellator = FillTessellator::new();

        let mut next_origin = origin;

        let texture_dimensions = self.glyphs.get_texture_dimensions();
        let texture_dimensions = (texture_dimensions.0 as f32, texture_dimensions.1 as f32);

        // TODO: silently drops unknown characters
        // TODO: handle line wrapping / line height
        let mut bbox = None;
        for glyph in label_text
            .chars()
            .filter_map(|c| self.glyphs.glyphs.get(&c))
            .collect::<Vec<_>>()
        {
            let glyph_dims = glyph.buffered_dimensions();
            let width = glyph_dims.0 as f32;
            let height = glyph_dims.1 as f32;

            let glyph_anchor = [
                next_origin[0] + glyph.left_bearing as f32,
                next_origin[1] - glyph.top_bearing as f32,
                0.,
            ];

            let glyph_rect = Box2D::new(
                (glyph_anchor[0], glyph_anchor[1]).into(),
                (glyph_anchor[0] + width, glyph_anchor[1] + height).into(),
            );

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

            next_origin[0] += glyph.advance() as f32;
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
        if name == "name" { // TODO: Support different tags
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
            self.tessellate_glyph_quads(
                origin,
                text.as_str(),
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
