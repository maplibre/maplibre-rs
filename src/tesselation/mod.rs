use bytemuck::Pod;
use std::ops::{Add, Range};

use lyon::tessellation::{
    FillVertex, FillVertexConstructor, StrokeVertex, StrokeVertexConstructor, VertexBuffers,
};
use wgpu::BufferAddress;

use crate::render::shader_ffi::GpuVertexUniform;

pub mod tile;

const DEFAULT_TOLERANCE: f32 = 0.02;

pub type IndexDataType = u16; // Must match INDEX_FORMAT

pub trait Tesselated<I: Add> {
    fn tesselate_stroke(&self) -> VertexBuffers<GpuVertexUniform, I>;
    fn tesselate_fill(&self) -> VertexBuffers<GpuVertexUniform, I>;

    fn empty_range(&self) -> VertexBuffers<GpuVertexUniform, I> {
        VertexBuffers::new()
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

#[derive(Clone)]
pub struct OverAlignedVertexBuffer<V, I> {
    pub buffer: VertexBuffers<V, I>,
    pub usable_indices: u32,
}

impl<V: Pod, I: Pod> From<VertexBuffers<V, I>> for OverAlignedVertexBuffer<V, I> {
    fn from(mut buffer: VertexBuffers<V, I>) -> Self {
        let usable_indices = buffer.indices.len() as u32;
        buffer.align_vertices();
        buffer.align_indices();
        Self {
            buffer,
            usable_indices,
        }
    }
}

trait Align<V: Pod, I: Pod> {
    fn align_vertices(&mut self);
    fn align_indices(&mut self);
}

impl<V: Pod, I: Pod> Align<V, I> for VertexBuffers<V, I> {
    fn align_vertices(&mut self) {
        let align = wgpu::COPY_BUFFER_ALIGNMENT;
        let stride = std::mem::size_of::<GpuVertexUniform>() as BufferAddress;
        let unpadded_bytes = self.vertices.len() as BufferAddress * stride;
        let padding_bytes = (align - unpadded_bytes % align) % align;

        if padding_bytes != 0 {
            panic!(
                "vertices are always aligned to wgpu::COPY_BUFFER_ALIGNMENT \
                    because GpuVertexUniform is aligned"
            )
        }
    }

    fn align_indices(&mut self) {
        let align = wgpu::COPY_BUFFER_ALIGNMENT;
        let stride = std::mem::size_of::<I>() as BufferAddress;
        let unpadded_bytes = self.indices.len() as BufferAddress * stride;
        let padding_bytes = (align - unpadded_bytes % align) % align;
        let overpad = (padding_bytes + stride - 1) / stride; // Divide by stride but round up

        for _ in 0..overpad {
            self.indices.push(I::zeroed());
        }
    }
}
