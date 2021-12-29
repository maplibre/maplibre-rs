use std::ops::Range;

use lyon::extra::rust_logo::build_logo_path;
use lyon::tessellation;
use lyon::tessellation::geometry_builder::MaxIndex;
use lyon::tessellation::{
    BuffersBuilder, FillOptions, FillTessellator, FillVertexConstructor, StrokeOptions,
    StrokeTessellator, StrokeVertexConstructor, VertexBuffers,
};
use lyon_path::builder::SvgPathBuilder;
use lyon_path::Path;

use vector_tile::geometry::{Command, Geometry};
use vector_tile::tile::Tile;

use crate::render::shader_ffi::GpuVertexUniform;
use crate::tesselation::{Tesselated, WithId, DEFAULT_TOLERANCE};

pub struct RustLogo();

impl<
        OutputIndex: std::ops::Add + std::convert::From<lyon::lyon_tessellation::VertexId> + MaxIndex,
    > Tesselated<OutputIndex> for RustLogo
{
    fn tesselate_stroke(
        &self,
        buffer: &mut VertexBuffers<GpuVertexUniform, OutputIndex>,
        prim_id: u32,
    ) -> Range<u32> {
        let mut stroke_tess = StrokeTessellator::new();

        let initial_indices_count = buffer.indices.len();

        // Build a Path for the rust logo.
        let mut rust_logo_builder = Path::builder().with_svg();
        build_logo_path(&mut rust_logo_builder);
        let rust_logo = rust_logo_builder.build();

        stroke_tess
            .tessellate_path(
                &rust_logo,
                &StrokeOptions::tolerance(DEFAULT_TOLERANCE),
                &mut BuffersBuilder::new(buffer, WithId(prim_id)),
            )
            .unwrap();

        initial_indices_count as u32..buffer.indices.len() as u32
    }

    fn tesselate_fill(
        &self,
        buffer: &mut VertexBuffers<GpuVertexUniform, OutputIndex>,
        prim_id: u32,
    ) -> Range<u32> {
        let mut fill_tess = FillTessellator::new();

        let initial_indices_count = buffer.indices.len();

        // Build a Path for the rust logo.
        let mut rust_logo_builder = Path::builder().with_svg();
        build_logo_path(&mut rust_logo_builder);
        let rust_logo = rust_logo_builder.build();

        fill_tess
            .tessellate_path(
                &rust_logo,
                &FillOptions::tolerance(DEFAULT_TOLERANCE)
                    .with_fill_rule(lyon_path::FillRule::NonZero),
                &mut BuffersBuilder::new(buffer, WithId(prim_id as u32)),
            )
            .unwrap();

        initial_indices_count as u32..buffer.indices.len() as u32
    }
}
