use lyon::tessellation::{FillVertex, FillVertexConstructor};

use crate::render::SymbolVertex;

pub mod glyph;

pub mod sdf_glyphs {
    include!(concat!(env!("OUT_DIR"), "/glyphs.rs"));
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
