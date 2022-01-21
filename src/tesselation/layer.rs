
use std::ops::Add;

use bytemuck::Pod;

use lyon::lyon_tessellation::VertexBuffers;
use lyon::tessellation::geometry_builder::MaxIndex;
use lyon::tessellation::{
    BuffersBuilder, FillOptions, FillTessellator, StrokeOptions, StrokeTessellator,
};
use lyon_path::traits::SvgPathBuilder;
use lyon_path::Path;

use vector_tile::geometry::{Command, Geometry};
use vector_tile::tile::{Layer};

use crate::render::ShaderVertex;
use crate::tesselation::{Tesselated, VertexConstructor, DEFAULT_TOLERANCE};

impl<I: Add + From<lyon::lyon_tessellation::VertexId> + MaxIndex + Pod> Tesselated<I> for Layer {
    fn tesselate(&self) -> Option<(VertexBuffers<ShaderVertex, I>, Vec<u32>)> {
        let mut buffer: VertexBuffers<ShaderVertex, I> = VertexBuffers::new();
        let mut feature_vertices: Vec<u32> = Vec::new();
        let mut last = 0;

        for feature in self.features() {
            match feature.geometry() {
                Geometry::GeometryPolygon(polygon) => {
                    let mut tile_builder = Path::builder().with_svg();
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

                    let mut tesselator = FillTessellator::new();
                    tesselator
                        .tessellate_path(
                            &tile_builder.build(),
                            &FillOptions::tolerance(DEFAULT_TOLERANCE),
                            &mut BuffersBuilder::new(&mut buffer, VertexConstructor {}),
                        )
                        .ok()?;
                }
                Geometry::GeometryLineString(polygon) => {
                    let mut tile_builder = Path::builder().with_svg();
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

                    let mut tesselator = StrokeTessellator::new();

                    tesselator
                        .tessellate_path(
                            &tile_builder.build(),
                            &StrokeOptions::tolerance(DEFAULT_TOLERANCE),
                            &mut BuffersBuilder::new(&mut buffer, VertexConstructor {}),
                        )
                        .ok()?;
                }
                _ => {}
            };

            let new_length = buffer.indices.len();
            feature_vertices.push((new_length - last) as u32);
            last = new_length;
        }

        Some((buffer, feature_vertices))
    }
}
