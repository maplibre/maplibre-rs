use lyon::tessellation::{FillVertex, FillVertexConstructor};

use crate::render::SymbolVertex;

pub mod glyph;

pub mod sdf_glyphs {
    include!(concat!(env!("OUT_DIR"), "/glyphs.rs"));
}

pub struct SymbolVertexBuilder {
    /// In meters
    pub anchor: [f32; 3],
    /// In meters
    pub texture_dimensions: (f32, f32),
    /// In meters
    pub sprite_dimensions: (f32, f32),
    /// In meters
    pub sprite_offset: (f32, f32),
    pub glyph: bool,
    pub color: [u8; 4],
}

impl FillVertexConstructor<SymbolVertex> for SymbolVertexBuilder {
    fn new_vertex(&mut self, vertex: FillVertex) -> SymbolVertex {
        let p = vertex.position();

        let sprite_ratio_x = self.sprite_dimensions.0 / self.texture_dimensions.0;
        let sprite_ratio_y = self.sprite_dimensions.1 / self.texture_dimensions.1;

        let x_offset = self.sprite_offset.0 / self.texture_dimensions.0;
        let y_offset = self.sprite_offset.1 / self.texture_dimensions.1;

        let tex_coords = [
            x_offset + ((p.x - self.anchor[0]) / self.sprite_dimensions.0) * sprite_ratio_x,
            y_offset
                + (sprite_ratio_y
                    - ((p.y - self.anchor[1]) / self.sprite_dimensions.1) * sprite_ratio_y),
        ];

        SymbolVertex {
            position: [p.x, p.y, 0.],
            origin: self.anchor,
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
