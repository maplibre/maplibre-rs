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
    word_offset: cgmath::Vector3<f32>, // offset in glyph coordinates (i.e. x and y advance!)
}

impl GlyphBuilder {
    pub fn new() -> GlyphBuilder {
        let mut builder = GlyphBuilder {
            vertices: Vec::new(),
            indices: Vec::new(),
            current_index: 0,
            added_points: 0,
            last_start_index: 0,
            word_offset: (0.0, 0.0, 0.0).into(),
        };

        builder.vertices.push(Vertex::new_2d((0.0, 0.0).into()));

        builder
    }

    pub fn new_with_offset(word_offset: cgmath::Vector3<f32>) -> GlyphBuilder {
        let mut builder = GlyphBuilder {
            vertices: Vec::new(),
            indices: Vec::new(),
            current_index: 0,
            added_points: 0,
            last_start_index: 0,
            word_offset,
        };

        builder.add_vertex(0.0, 0.0); // Base Vertex for normal triangles, will be adjusted in finalize()

        builder
    }

    fn add_vertex(&mut self, x: f32, y: f32) {
        let point = cgmath::Vector3::new(x, y, 0.0);
        self.vertices.push(Vertex::new_3d(point));
    }

    fn add_vertex_uv(&mut self, x: f32, y: f32, u: f32, v: f32) {
        let point = cgmath::Vector3::new(x, y, 0.0);
        self.vertices.push(Vertex::new_3d_uv(point, (u, v).into()));
    }

    fn make_triangle(&mut self) {
        self.indices.push(0);
        self.indices.push(self.current_index);
        self.indices.push(self.current_index + 1);

        self.current_index += 1;
        self.added_points = 1;
    }

    fn scale(&mut self, s: f32) {
        let s_vec = cgmath::Vector3::new(s, s, 1.0);
        for v in &mut self.vertices {
            v.scale_3d(&s_vec);
        }
    }

    fn finalize(&mut self, bbox: &ttf::Rect) {
        // Move the first vertex (base for all the triangles) into the center of the bounding box
        // This hopefully avoids overlapping triangles between different glyphs (which would torpedo the winding number hack of the fragment shader)
        if let Some(first_vertex) = self.vertices.first_mut() {
            (*first_vertex).position[0] = bbox.width() as f32 * 0.5;
            (*first_vertex).position[1] = bbox.height() as f32 * 0.5;
        }
    }

    pub fn prepare_for_screen(
        &mut self,
        bbox: &ttf::Rect,
        scale: f32,
        world_translation: &cgmath::Vector3<f32>,
    ) {
        self.finalize(bbox);
        // Now we translate to the appropriate offset in font space
        self.translate(&self.word_offset.clone());
        // Scale it to world space
        self.scale(scale);
        // Move it in world space
        self.translate(world_translation);
    }

    fn translate(&mut self, trans: &cgmath::Vector3<f32>) {
        for v in &mut self.vertices {
            v.position[0] += trans.x;
            v.position[1] += trans.y;
            v.position[2] += trans.z;
        }
    }
}

impl ttf::OutlineBuilder for GlyphBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        self.add_vertex(x, y);

        // Move-to starts a new shape
        self.last_start_index = self.vertices.len() as u16 - 1;
        self.added_points = 1;
        self.current_index += 1;
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.add_vertex(x, y);
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

        self.add_vertex_uv(x1, y1, 0.5, 0.0);
        self.add_vertex_uv(x, y, 1.0, 1.0);

        // The special triangle
        self.indices.push(self.current_index);
        self.indices.push(self.current_index + 1);
        self.indices.push(self.current_index + 2);

        self.added_points += 1;
        if self.added_points == 2 {
            self.add_vertex(x, y); // duplicate of the end point without special uv coordinates

            self.indices.push(0);
            self.indices.push(self.current_index);
            self.indices.push(self.current_index + 3);

            self.added_points = 1;
            self.current_index += 3;
        }
    }

    fn curve_to(&mut self, _x1: f32, _y1: f32, _x2: f32, _y2: f32, _x: f32, _y: f32) {
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
