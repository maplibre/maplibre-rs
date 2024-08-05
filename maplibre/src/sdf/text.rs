use std::{collections::BTreeMap, convert::TryFrom};

use image::{GenericImage, GenericImageView, GrayImage, ImageBuffer, Luma};
use lyon::tessellation::{FillVertex, FillVertexConstructor};
use prost::{DecodeError, Message};
use crate::render::shaders::SymbolVertex;

pub mod sdf_glyphs {
    include!(concat!(env!("OUT_DIR"), "/glyphs.rs"));
}

pub type UnicodePoint = char;

#[derive(Debug)]
pub struct Glyph {
    pub codepoint: UnicodePoint,
    pub width: u32,
    pub height: u32,
    pub left_bearing: i32,
    pub top_bearing: i32,
    h_advance: u32,
    /// x origin coordinate within the packed texture
    tex_origin_x: u32,
    /// y origin coordinate within the packed texture
    tex_origin_y: u32,
}

impl Glyph {
    fn from_pbf(g: sdf_glyphs::Glyph, origin_x: u32, origin_y: u32) -> Self {
        Self {
            codepoint: char::try_from(g.id).unwrap(),
            width: g.width,
            height: g.height,
            left_bearing: g.left,
            top_bearing: g.top,
            h_advance: g.advance,
            tex_origin_x: origin_x,
            tex_origin_y: origin_y,
        }
    }

    pub fn buffered_dimensions(&self) -> (u32, u32) {
        (self.width + 3 * 2, self.height + 3 * 2)
    }
    pub fn origin_offset(&self) -> (u32, u32) {
        (self.tex_origin_x, self.tex_origin_y)
    }
    pub fn advance(&self) -> u32 {
        self.h_advance
    }
}

pub struct GlyphSet {
    texture_bytes: Vec<u8>,
    texture_dimensions: (usize, usize),
    pub glyphs: BTreeMap<UnicodePoint, Glyph>,
}

impl TryFrom<&[u8]> for GlyphSet {
    type Error = DecodeError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Ok(GlyphSet::from(sdf_glyphs::Glyphs::decode(value)?))
    }
}

impl From<sdf_glyphs::Glyphs> for GlyphSet {
    fn from(pbf_glyphs: sdf_glyphs::Glyphs) -> Self {
        let stacks = pbf_glyphs.stacks;
        let mut texture: GrayImage = ImageBuffer::new(4096, 4096);
        let mut last_position = (0, 0);
        let mut max_height = 0;

        let glyphs = stacks
            .into_iter()
            .flat_map(|stack| {
                stack
                    .glyphs
                    .into_iter()
                    .filter_map(|mut glyph| {
                        // Save an extra copy operation by taking the bits out directly.
                        let bitmap = glyph.bitmap.take()?;

                        let glyph = Glyph::from_pbf(glyph, last_position.0, last_position.1);

                        let buffered_width = glyph.width + 3 * 2;
                        let buffered_height = glyph.height + 3 * 2;

                        let glyph_texture = ImageBuffer::<Luma<u8>, _>::from_vec(
                            buffered_width,
                            buffered_height,
                            bitmap,
                        )?;
                        assert_eq!(buffered_height, glyph_texture.height());
                        assert_eq!(buffered_width, glyph_texture.width());

                        // TODO: wraparound on texture width overflow
                        texture
                            .copy_from(&glyph_texture, last_position.0, last_position.1)
                            .expect("Unable to copy glyph texture.");

                        last_position.0 += glyph_texture.width();
                        max_height = max_height.max(glyph_texture.height());

                        Some((glyph.codepoint, glyph))
                    })
                    .collect::<Vec<_>>()
            })
            .collect();

        Self {
            texture_bytes: texture
                .view(0, 0, last_position.0, max_height)
                .pixels()
                .map(|(_x, _y, p)| p[0])
                .collect(),
            texture_dimensions: (last_position.0 as _, max_height as _),
            glyphs,
        }
    }
}

impl GlyphSet {
    pub fn get_texture_dimensions(&self) -> (usize, usize) {
        self.texture_dimensions
    }

    pub fn get_texture_bytes(&self) -> &[u8] {
        self.texture_bytes.as_slice()
    }
}

pub struct SymbolVertexBuilder {
    /// Where is the top-left anchor of the glyph box
    pub glyph_anchor: [f32; 3],
    /// Where is the top-left anchor of the text box
    pub text_anchor: [f32; 3],
    /// Size of sprite-sheet * font_scale
    pub texture_dimensions: (f32, f32),
    /// Size of individual glyph * font_scale
    pub sprite_dimensions: (f32, f32),
    /// where in the sheet is the sprite * font_scale
    pub sprite_offset: (f32, f32),
    pub glyph: bool,
    pub color: [u8; 4],
}

impl FillVertexConstructor<SymbolVertex> for SymbolVertexBuilder {
    fn new_vertex(&mut self, vertex: FillVertex) -> SymbolVertex {
        let vertex_position = vertex.position();

        let sprite_ratio_x = self.sprite_dimensions.0 / self.texture_dimensions.0;
        let sprite_ratio_y = self.sprite_dimensions.1 / self.texture_dimensions.1;

        let x_offset = self.sprite_offset.0 / self.texture_dimensions.0;
        let y_offset = self.sprite_offset.1 / self.texture_dimensions.1;

        let tex_coords = [
            x_offset
                + ((vertex_position.x - self.glyph_anchor[0]) / self.sprite_dimensions.0)
                * sprite_ratio_x,
            y_offset
                + ((vertex_position.y - self.glyph_anchor[1]) / self.sprite_dimensions.1)
                * sprite_ratio_y,
        ];

        SymbolVertex {
            position: [vertex_position.x, vertex_position.y, 0.],
            text_anchor: self.text_anchor,
            is_glyph: if self.glyph { 1 } else { 0 },
            color: self.color,
            tex_coords,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Anchor {
    Center,
    Left,
    Right,
    Top,
    Bottom,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}