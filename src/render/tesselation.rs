use std::ops::Range;

use lyon::extra::rust_logo::build_logo_path;
use lyon::lyon_tessellation::{FillTessellator, StrokeTessellator};
use lyon::tessellation;
use lyon::tessellation::{
    BuffersBuilder, FillOptions, FillVertexConstructor, StrokeOptions, StrokeVertexConstructor,
    VertexBuffers,
};
use lyon::tessellation::geometry_builder::MaxIndex;
use lyon_path::builder::SvgPathBuilder;
use lyon_path::Path;

use vector_tile::geometry::{Command, Geometry};
use vector_tile::tile::Tile;

use super::shader_ffi::GpuVertexUniform;

const DEFAULT_TOLERANCE: f32 = 0.02;

pub trait Tesselated<OutputIndex: std::ops::Add> {
    fn tesselate_stroke(&self, buffer: &mut VertexBuffers<GpuVertexUniform, OutputIndex>, prim_id: u32) -> Range<u32>;
    fn tesselate_fill(&self, buffer: &mut VertexBuffers<GpuVertexUniform, OutputIndex>, prim_id: u32) -> Range<u32>;

    fn empty_range(&self, buffer: &mut VertexBuffers<GpuVertexUniform, OutputIndex>,
                   _prim_id: u32) -> Range<u32> {
        let initial_indices_count = buffer.indices.len() as u32;
        initial_indices_count..initial_indices_count
    }
}

/// This vertex constructor forwards the positions and normals provided by the
/// tessellators and add a shape id.
pub struct WithId(pub u32);

impl FillVertexConstructor<GpuVertexUniform> for WithId {
    fn new_vertex(&mut self, vertex: tessellation::FillVertex) -> GpuVertexUniform {
        GpuVertexUniform::new(vertex.position().to_array(), [0.0, 0.0], self.0)
    }
}

impl StrokeVertexConstructor<GpuVertexUniform> for WithId {
    fn new_vertex(&mut self, vertex: tessellation::StrokeVertex) -> GpuVertexUniform {
        GpuVertexUniform::new(
            vertex.position_on_path().to_array(),
            vertex.normal().to_array(),
            self.0,
        )
    }
}


fn build_path(
    tile: &Tile,
    fill: bool
) -> Path {
    let mut tile_builder = Path::builder().with_svg();

    for layer in tile.layers() {
        if layer.name() != "transportation" {
            continue;
        }

        for feature in layer.features() {
            let geo = feature.geometry();

            match geo {
                Geometry::GeometryPolygon(polygon) => {
                    for command in &polygon.commands {
                        match command {
                            Command::MoveTo(cmd) => {
                                tile_builder.relative_move_to(lyon_path::math::vector(
                                    cmd.x as f32,
                                    cmd.y as f32,
                                ));
                            }
                            Command::LineTo(cmd) => {
                                tile_builder.relative_line_to(lyon_path::math::vector(
                                    cmd.x as f32,
                                    cmd.y as f32,
                                ));
                            }
                            Command::Close => {
                                tile_builder.close();
                            }
                        };
                    }
                }
                Geometry::GeometryLineString(polygon) => {
                    if !fill {
                        for command in &polygon.commands {
                            match command {
                                Command::MoveTo(cmd) => {
                                    tile_builder.relative_move_to(lyon_path::math::vector(
                                        cmd.x as f32,
                                        cmd.y as f32,
                                    ));
                                }
                                Command::LineTo(cmd) => {
                                    tile_builder.relative_line_to(lyon_path::math::vector(
                                        cmd.x as f32,
                                        cmd.y as f32,
                                    ));
                                }
                                Command::Close => {
                                    panic!("error")
                                }
                            };
                        }
                    }
                }
                _ => {}
            };
            tile_builder.move_to(lyon_path::math::point(0.0, 0.0));
        }
    }

    tile_builder.build()
}

impl<OutputIndex: std::ops::Add + std::convert::From<lyon::lyon_tessellation::VertexId> + MaxIndex> Tesselated<OutputIndex> for Tile {
    fn tesselate_stroke(
        &self,
        buffer: &mut VertexBuffers<GpuVertexUniform, OutputIndex>,
        prim_id: u32,
    ) -> Range<u32> {
        let mut tesselator = StrokeTessellator::new();

        let initial_indices_count = buffer.indices.len();

        let tile_path = build_path(self, false);

        tesselator
            .tessellate_path(
                &tile_path,
                &StrokeOptions::tolerance(DEFAULT_TOLERANCE),
                &mut BuffersBuilder::new(buffer, WithId(prim_id)),
            )
            .unwrap();

        initial_indices_count as u32..buffer.indices.len() as u32
    }

    fn tesselate_fill(&self, buffer: &mut VertexBuffers<GpuVertexUniform, OutputIndex>, prim_id: u32) -> Range<u32> {
        let mut tesselator = FillTessellator::new();

        let initial_indices_count = buffer.indices.len();

        let tile_path = build_path(self, true);

        tesselator
            .tessellate_path(
                &tile_path,
                &FillOptions::tolerance(DEFAULT_TOLERANCE),
                &mut BuffersBuilder::new(buffer, WithId(prim_id)),
            )
            .unwrap();

        initial_indices_count as u32..buffer.indices.len() as u32
    }
}

pub struct RustLogo();

impl<OutputIndex: std::ops::Add + std::convert::From<lyon::lyon_tessellation::VertexId> + MaxIndex> Tesselated<OutputIndex> for RustLogo {
    fn tesselate_stroke(&self, buffer: &mut VertexBuffers<GpuVertexUniform, OutputIndex>, prim_id: u32) -> Range<u32> {
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

    fn tesselate_fill(&self, buffer: &mut VertexBuffers<GpuVertexUniform, OutputIndex>, prim_id: u32) -> Range<u32> {
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


const EXTENT: f32 = 4096.0;

pub struct TileMask();

impl Tesselated<u32> for TileMask {
    fn tesselate_stroke(&self, _buffer: &mut VertexBuffers<GpuVertexUniform, u32>, _prim_id: u32) -> Range<u32> {
        0..0
    }

    fn tesselate_fill(&self, buffer: &mut VertexBuffers<GpuVertexUniform, u32>, prim_id: u32) -> Range<u32> {
        let initial_indices_count = buffer.indices.len();

        buffer.vertices = vec![
            GpuVertexUniform::new([0.0, 0.0], [0.0, 0.0], prim_id),
            GpuVertexUniform::new([EXTENT, 0.0], [0.0, 0.0], prim_id),
            GpuVertexUniform::new([0.0, EXTENT], [0.0, 0.0], prim_id),
            GpuVertexUniform::new([EXTENT, EXTENT], [0.0, 0.0], prim_id),
        ];

        buffer.indices = vec![0, 2, 1, 3, 2, 1];

        initial_indices_count as u32..buffer.indices.len() as u32
    }
}