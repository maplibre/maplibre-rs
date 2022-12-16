use std::{collections::BTreeMap, convert::TryFrom};

use image::{GenericImage, GenericImageView, GrayImage, ImageBuffer, Luma};

use crate::text::sdf_glyphs::{Glyph as ProtoGlyph, Glyphs};

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
    fn from_pbf(g: ProtoGlyph, origin_x: u32, origin_y: u32) -> Self {
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

impl From<Glyphs> for GlyphSet {
    fn from(pbf_glyphs: Glyphs) -> Self {
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
