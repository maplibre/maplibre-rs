use crate::geom::{Mesh, Meshable, Vertex};
use std::fmt::Write;
use ttf_parser as ttf;

pub struct SVGBuilder(pub String);

impl ttf_parser::OutlineBuilder for SVGBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        write!(&mut self.0, "M {} {} ", x, y).unwrap();
    }

    fn line_to(&mut self, x: f32, y: f32) {
        write!(&mut self.0, "L {} {} ", x, y).unwrap();
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        write!(&mut self.0, "Q {} {} {} {} ", x1, y1, x, y).unwrap();
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        write!(&mut self.0, "C {} {} {} {} {} {} ", x1, y1, x2, y2, x, y).unwrap();
    }

    fn close(&mut self) {
        write!(&mut self.0, "Z ").unwrap();
    }
}

#[derive(Debug)]
pub struct GlyphBuilder {
    // Take lines from path description and turn into triangles with an arbitrary point (0, 0).
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
    current_index: u16,
    added_points: u16,
    last_start_index: u16,
}

impl GlyphBuilder {
    pub fn new() -> GlyphBuilder {
        let mut builder = GlyphBuilder {
            vertices: Vec::new(),
            indices: Vec::new(),
            current_index: 0,
            added_points: 0,
            last_start_index: 0,
        };

        builder.vertices.push(Vertex::new_2d(0.0, 0.0));

        builder
    }

    fn make_triangle(&mut self) {
        self.indices.push(0);
        self.indices.push(self.current_index);
        self.indices.push(self.current_index + 1);

        self.current_index += 1;
        self.added_points = 1;
    }

    pub fn normalize(&mut self, bbox: &ttf::Rect) {
        let width = bbox.width() as f32;
        let height = bbox.height() as f32;
        let mut first = true;
        for v in &mut self.vertices {
            if first {
                first = false;
                continue;
            }
            v.position[0] = (v.position[0] - bbox.x_min as f32) / width;
            v.position[1] = (v.position[1] - bbox.y_min as f32) / height;
        }
    }
}

impl ttf::OutlineBuilder for GlyphBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        self.vertices.push(Vertex::new_2d(x, y));

        // Move-to starts a new shape
        self.last_start_index = self.vertices.len() as u16 - 1;
        self.added_points = 1;
        self.current_index += 1;
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.vertices.push(Vertex::new_2d(x, y));
        self.added_points += 1;
        if self.added_points == 2 {
            self.make_triangle();
        }
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        // Quadratic curve (control point, end point), start point is endpoint of previous segment

        // The last pushed vertex is the start point of the curve
        // We need to construct 2 triangles
        // A "normal" triangle as with line segments
        // And another special triangle for the rounded area of the curve,
        //      which is equipped with special uv coordinates which the pixel shader can use to check if a pixel is inside or outside the curve
        // Because the endpoint is the start point of the next path segment, we first construct the special triangle

        self.vertices.push(Vertex::new_2d_uv(x1, y1, 0.5, 0.0));
        self.vertices.push(Vertex::new_2d_uv(x, y, 1.0, 1.0));

        // The special triangle
        self.indices.push(self.current_index);
        self.indices.push(self.current_index + 1);
        self.indices.push(self.current_index + 2);

        self.added_points += 1;
        if self.added_points == 2 {
            self.vertices.push(Vertex::new_2d(x, y)); // duplicate of the end point without special uv coordinates
            self.indices.push(0);
            self.indices.push(self.current_index);
            self.indices.push(self.current_index + 3);

            self.added_points = 1;
            self.current_index += 3;
        }
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        // Cubic curve (control point 1, control point 2, end point)
        panic!("Cubic bezier curves not yet supported!");
    }

    fn close(&mut self) {
        // triangle from current point (i.e. the last one that was read in) and the start point of this shape
        self.indices.push(0);
        self.indices.push(self.current_index);
        self.indices.push(self.last_start_index);

        self.indices.push(0);
        self.indices.push(self.last_start_index);
        self.indices.push(self.current_index);

        // the next command MUST be a move to if there are more shapes
        // This will reset the necessary counters
    }
}

impl Meshable for GlyphBuilder {
    fn as_mesh(&self, device: &wgpu::Device) -> Mesh {
        let mesh = Mesh::new(self.vertices.clone(), self.indices.clone(), device);
        mesh
    }
}
