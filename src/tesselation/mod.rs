mod misc;
pub mod tile;
pub mod tile_mask;

use crate::render::shader_ffi::GpuVertexUniform;
use lyon::tessellation::{
    FillVertex, FillVertexConstructor, StrokeVertex, StrokeVertexConstructor, VertexBuffers,
};
use std::ops::Range;

const DEFAULT_TOLERANCE: f32 = 0.02;

pub trait Tesselated<OutputIndex: std::ops::Add> {
    fn tesselate_stroke(
        &self,
        buffer: &mut VertexBuffers<GpuVertexUniform, OutputIndex>,
        prim_id: u32,
    ) -> Range<u32>;
    fn tesselate_fill(
        &self,
        buffer: &mut VertexBuffers<GpuVertexUniform, OutputIndex>,
        prim_id: u32,
    ) -> Range<u32>;

    fn empty_range(
        &self,
        buffer: &mut VertexBuffers<GpuVertexUniform, OutputIndex>,
        _prim_id: u32,
    ) -> Range<u32> {
        let initial_indices_count = buffer.indices.len() as u32;
        initial_indices_count..initial_indices_count
    }
}

/// This vertex constructor forwards the positions and normals provided by the
/// tessellators and add a shape id.
pub struct WithId(pub u32);

impl FillVertexConstructor<GpuVertexUniform> for WithId {
    fn new_vertex(&mut self, vertex: FillVertex) -> GpuVertexUniform {
        GpuVertexUniform::new(vertex.position().to_array(), [0.0, 0.0], self.0)
    }
}

impl StrokeVertexConstructor<GpuVertexUniform> for WithId {
    fn new_vertex(&mut self, vertex: StrokeVertex) -> GpuVertexUniform {
        GpuVertexUniform::new(
            vertex.position_on_path().to_array(),
            vertex.normal().to_array(),
            self.0,
        )
    }
}
