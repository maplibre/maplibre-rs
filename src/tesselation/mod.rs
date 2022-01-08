mod misc;
pub mod tile;

use crate::render::shader_ffi::GpuVertexUniform;
use lyon::tessellation::{
    FillVertex, FillVertexConstructor, StrokeVertex, StrokeVertexConstructor, VertexBuffers,
};
use std::ops::Range;

const DEFAULT_TOLERANCE: f32 = 0.02;

pub type IndexDataType = u16; // Must match INDEX_FORMAT

pub trait Tesselated<OutputIndex: std::ops::Add> {
    fn tesselate_stroke(
        &self,
        buffer: &mut VertexBuffers<GpuVertexUniform, OutputIndex>,
    ) -> Range<IndexDataType>;
    fn tesselate_fill(
        &self,
        buffer: &mut VertexBuffers<GpuVertexUniform, OutputIndex>,
    ) -> Range<IndexDataType>;

    fn empty_range(
        &self,
        buffer: &mut VertexBuffers<GpuVertexUniform, OutputIndex>,
    ) -> Range<IndexDataType> {
        let initial_indices_count = buffer.indices.len() as IndexDataType;
        initial_indices_count..initial_indices_count
    }
}

pub struct VertexConstructor();

impl FillVertexConstructor<GpuVertexUniform> for VertexConstructor {
    fn new_vertex(&mut self, vertex: FillVertex) -> GpuVertexUniform {
        GpuVertexUniform::new(vertex.position().to_array(), [0.0, 0.0])
    }
}

impl StrokeVertexConstructor<GpuVertexUniform> for VertexConstructor {
    fn new_vertex(&mut self, vertex: StrokeVertex) -> GpuVertexUniform {
        GpuVertexUniform::new(
            vertex.position_on_path().to_array(),
            vertex.normal().to_array(),
        )
    }
}

trait Align<V: bytemuck::Pod, I: bytemuck::Pod> {
    fn align_indices(&mut self);
}

impl<V: bytemuck::Pod, I: bytemuck::Pod> Align<V, I> for VertexBuffers<V, I> {
    fn align_indices(&mut self) {
        let alignment = wgpu::COPY_BUFFER_ALIGNMENT as usize / std::mem::size_of::<I>();
        let padding = self.indices.len() % alignment;
        if padding > 0 {
            self.indices
                .extend(std::iter::repeat(I::zeroed()).take(alignment - padding));
        }
    }
}
