use std::ops::Range;

use lyon::lyon_tessellation::LineJoin;
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
use crate::tesselation::{Align, IndexDataType, Tesselated, VertexConstructor, DEFAULT_TOLERANCE};

fn build_path(tile: &Tile, fill: bool) -> Path {
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

                                    //print!("M{} {} ", cmd.x, cmd.y);
                                }
                                Command::LineTo(cmd) => {
                                    tile_builder.relative_line_to(lyon_path::math::vector(
                                        cmd.x as f32,
                                        cmd.y as f32,
                                    ));

                                    //print!("l{} {} ", cmd.x, cmd.y);
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

impl<
        OutputIndex: std::ops::Add
            + std::convert::From<lyon::lyon_tessellation::VertexId>
            + MaxIndex
            + bytemuck::Pod,
    > Tesselated<OutputIndex> for Tile
{
    fn tesselate_stroke(
        &self,
        buffer: &mut VertexBuffers<GpuVertexUniform, OutputIndex>,
    ) -> Range<IndexDataType> {
        let mut tesselator = StrokeTessellator::new();

        let initial_indices_count = buffer.indices.len();

        let tile_path = build_path(self, false);

        tesselator
            .tessellate_path(
                &tile_path,
                &StrokeOptions::tolerance(DEFAULT_TOLERANCE),
                &mut BuffersBuilder::new(buffer, VertexConstructor()),
            )
            .unwrap();

        buffer.align_indices();

        initial_indices_count as IndexDataType..buffer.indices.len() as IndexDataType
    }

    fn tesselate_fill(
        &self,
        buffer: &mut VertexBuffers<GpuVertexUniform, OutputIndex>,
    ) -> Range<IndexDataType> {
        let mut tesselator = FillTessellator::new();

        let initial_indices_count = buffer.indices.len();

        let tile_path = build_path(self, true);

        tesselator
            .tessellate_path(
                &tile_path,
                &FillOptions::tolerance(DEFAULT_TOLERANCE),
                &mut BuffersBuilder::new(buffer, VertexConstructor()),
            )
            .unwrap();

        initial_indices_count as IndexDataType..buffer.indices.len() as IndexDataType
    }
}
