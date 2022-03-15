use std::ops::Add;

use bytemuck::Pod;
use lyon::geom::point;

use lyon::lyon_tessellation::VertexBuffers;
use lyon::tessellation::geometry_builder::MaxIndex;
use lyon::tessellation::{
    BuffersBuilder, FillOptions, FillTessellator, StrokeOptions, StrokeTessellator,
};
use lyon_path::traits::SvgPathBuilder;
use lyon_path::{FillRule, Path};

use crate::error::Error;
use vector_tile::geometry::{Command, Geometry};
use vector_tile::tile::Layer;

use crate::render::ShaderVertex;
use crate::tessellation::{Tessellated, VertexConstructor, DEFAULT_TOLERANCE};

impl<I: Add + From<lyon::lyon_tessellation::VertexId> + MaxIndex + Pod> Tessellated<I> for Layer {
    fn tessellate(&self) -> Result<(VertexBuffers<ShaderVertex, I>, Vec<u32>), Error> {
        let mut buffer: VertexBuffers<ShaderVertex, I> = VertexBuffers::new();
        let mut feature_indices: Vec<u32> = Vec::new();
        let mut current_index = 0;

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

                    let mut fill_tessellator = FillTessellator::new();
                    fill_tessellator.tessellate_path(
                        &polygon_builder.build(),
                        &FillOptions::tolerance(DEFAULT_TOLERANCE)
                            .with_fill_rule(FillRule::NonZero),
                        &mut BuffersBuilder::new(&mut buffer, VertexConstructor {}),
                    )?;
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

                    let mut stroke_tessellator = StrokeTessellator::new();

                    stroke_tessellator.tessellate_path(
                        &line_string_builder.build(),
                        &StrokeOptions::tolerance(DEFAULT_TOLERANCE),
                        &mut BuffersBuilder::new(&mut buffer, VertexConstructor {}),
                    )?;
                }
                _ => {}
            };

            let next_index = buffer.indices.len();
            let indices = (next_index - current_index) as u32;
            feature_indices.push(indices);
            current_index = next_index;
        }

        Ok((buffer, feature_indices))
    }
}
