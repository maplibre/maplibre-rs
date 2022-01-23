use std::ops::Add;

use bytemuck::Pod;
use lyon::geom::{point, vector};

use lyon::lyon_tessellation::VertexBuffers;
use lyon::tessellation::geometry_builder::MaxIndex;
use lyon::tessellation::{
    BuffersBuilder, FillOptions, FillTessellator, StrokeOptions, StrokeTessellator,
};
use lyon_path::traits::SvgPathBuilder;
use lyon_path::{FillRule, Path};

use vector_tile::geometry::{Command, Geometry};
use vector_tile::tile::Layer;

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
                    let mut polygon_builder = Path::builder();
                    let mut cursor = point(0.0, 0.0);
                    for command in &polygon.commands {
                        match command {
                            Command::MoveTo(cmd) => {
                                let delta = lyon_path::math::vector(cmd.x as f32, cmd.y as f32);
                                cursor += delta;
                                polygon_builder.begin(cursor);
                            }
                            Command::LineTo(cmd) => {
                                let delta = lyon_path::math::vector(cmd.x as f32, cmd.y as f32);
                                cursor += delta;
                                polygon_builder.line_to(cursor);
                            }
                            Command::Close => {
                                polygon_builder.close();
                            }
                        };
                    }

                    let mut fill_tesselator = FillTessellator::new();
                    fill_tesselator
                        .tessellate_path(
                            &polygon_builder.build(),
                            &FillOptions::tolerance(DEFAULT_TOLERANCE)
                                .with_fill_rule(FillRule::NonZero),
                            &mut BuffersBuilder::new(&mut buffer, VertexConstructor {}),
                        )
                        .ok()?;
                }
                Geometry::GeometryLineString(line_string) => {
                    let mut line_string_builder = Path::builder();
                    let mut cursor = point(0.0, 0.0);
                    let mut subpath_open = false;
                    for command in &line_string.commands {
                        match command {
                            Command::MoveTo(cmd) => {
                                if subpath_open {
                                    line_string_builder.end(false);
                                }

                                let delta = lyon_path::math::vector(cmd.x as f32, cmd.y as f32);
                                cursor += delta;
                                line_string_builder.begin(cursor);
                                subpath_open = true;
                            }
                            Command::LineTo(cmd) => {
                                let delta = lyon_path::math::vector(cmd.x as f32, cmd.y as f32);
                                cursor += delta;
                                line_string_builder.line_to(cursor);
                            }
                            Command::Close => {
                                panic!("error")
                            }
                        };
                    }

                    if subpath_open {
                        line_string_builder.end(false);
                    }

                    let mut stroke_tesselator = StrokeTessellator::new();

                    stroke_tesselator
                        .tessellate_path(
                            &line_string_builder.build(),
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
