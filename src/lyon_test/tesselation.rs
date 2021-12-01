use lyon::extra::rust_logo::build_logo_path;
use lyon::lyon_tessellation::{FillTessellator, StrokeTessellator};
use lyon::tessellation;
use lyon::tessellation::{
    BuffersBuilder, FillOptions, FillVertexConstructor, StrokeOptions, StrokeVertexConstructor,
    VertexBuffers,
};
use lyon_path::builder::SvgPathBuilder;
use lyon_path::Path;
use vector_tile::geometry::{Command, Geometry};
use vector_tile::tile::Tile;

use crate::shader_ffi::GpuVertex;

const DEFAULT_TOLERANCE: f32 = 0.02;

pub trait Tesselated {
    fn tesselate_stroke(&self, buffer: &mut VertexBuffers<GpuVertex, u16>, prim_id: u32) -> u32;
    fn tesselate_fill(&self, buffer: &mut VertexBuffers<GpuVertex, u16>, prim_id: u32) -> u32;
}

/// This vertex constructor forwards the positions and normals provided by the
/// tessellators and add a shape id.
pub struct WithId(pub u32);

impl FillVertexConstructor<GpuVertex> for WithId {
    fn new_vertex(&mut self, vertex: tessellation::FillVertex) -> GpuVertex {
        GpuVertex {
            position: vertex.position().to_array(),
            normal: [0.0, 0.0],
            prim_id: self.0,
        }
    }
}

impl StrokeVertexConstructor<GpuVertex> for WithId {
    fn new_vertex(&mut self, vertex: tessellation::StrokeVertex) -> GpuVertex {
        GpuVertex {
            position: vertex.position_on_path().to_array(),
            normal: vertex.normal().to_array(),
            prim_id: self.0,
        }
    }
}

impl Tesselated for Tile {
    fn tesselate_stroke(&self, buffer: &mut VertexBuffers<GpuVertex, u16>, prim_id: u32) -> u32 {
        let mut stroke_tess = StrokeTessellator::new();
        let mut tile_builder = Path::builder().with_svg();

        for layer in self.layers() {
            if layer.name() != "water" {
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
                                        cmd.x as f32 / 10.0,
                                        cmd.y as f32 / 10.0,
                                    ));
                                }
                                Command::LineTo(cmd) => {
                                    tile_builder.relative_line_to(lyon_path::math::vector(
                                        cmd.x as f32 / 10.0,
                                        cmd.y as f32 / 10.0,
                                    ));
                                }
                                Command::Close => {
                                    tile_builder.close();
                                }
                            };
                        }
                    }
                    Geometry::GeometryLineString(polygon) => {
                        for command in &polygon.commands {
                            match command {
                                Command::MoveTo(cmd) => {
                                    tile_builder.relative_move_to(lyon_path::math::vector(
                                        cmd.x as f32 / 10.0,
                                        cmd.y as f32 / 10.0,
                                    ));
                                }
                                Command::LineTo(cmd) => {
                                    tile_builder.relative_line_to(lyon_path::math::vector(
                                        cmd.x as f32 / 10.0,
                                        cmd.y as f32 / 10.0,
                                    ));
                                }
                                Command::Close => {
                                    tile_builder.close();
                                }
                            };
                        }
                    }
                    _ => {}
                };
                //tile_builder.close();
                tile_builder.move_to(lyon_path::math::point(0.0, 0.0));
            }
        }

        let tile_path = tile_builder.build();

        stroke_tess
            .tessellate_path(
                &tile_path,
                &StrokeOptions::tolerance(DEFAULT_TOLERANCE),
                &mut BuffersBuilder::new(buffer, WithId(prim_id)),
            )
            .unwrap();

        buffer.indices.len() as u32
    }

    fn tesselate_fill(&self, _buffer: &mut VertexBuffers<GpuVertex, u16>, _prim_id: u32) -> u32 {
        return 0;
    }
}

pub struct RustLogo();

impl Tesselated for RustLogo {
    fn tesselate_stroke(&self, buffer: &mut VertexBuffers<GpuVertex, u16>, prim_id: u32) -> u32 {
        let mut stroke_tess = StrokeTessellator::new();

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

        buffer.indices.len() as u32
    }

    fn tesselate_fill(&self, buffer: &mut VertexBuffers<GpuVertex, u16>, prim_id: u32) -> u32 {
        let mut fill_tess = FillTessellator::new();

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

        buffer.indices.len() as u32
    }
}
