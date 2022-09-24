//! Tessellation for lines and polygons is implemented here.

use std::ops::Add;

use bytemuck::Pod;
use lyon::tessellation::{
    FillVertex, FillVertexConstructor, StrokeVertex, StrokeVertexConstructor, VertexBuffers,
};

use crate::{error::Error, render::ShaderVertex};

pub mod zero_tessellator;

const DEFAULT_TOLERANCE: f32 = 0.02;

/// Vertex buffers index data type.
pub type IndexDataType = u32; // Must match INDEX_FORMAT

/// An element that can be tessellated into vertex buffers.
pub trait Tessellated<I: Add> {
    /// Returns a vertex buffer which represents some object like a layer. Each object can contain
    /// multiple features. For each feature also the amount of indices is returned.
    ///
    fn tessellate(&self) -> Result<(VertexBuffers<ShaderVertex, I>, Vec<u32>), Error>;
}

/// Constructor for Fill and Stroke vertices.
pub struct VertexConstructor {}

impl FillVertexConstructor<ShaderVertex> for VertexConstructor {
    fn new_vertex(&mut self, vertex: FillVertex) -> ShaderVertex {
        ShaderVertex::new(vertex.position().to_array(), [0.0, 0.0])
    }
}

impl StrokeVertexConstructor<ShaderVertex> for VertexConstructor {
    fn new_vertex(&mut self, vertex: StrokeVertex) -> ShaderVertex {
        ShaderVertex::new(
            vertex.position_on_path().to_array(),
            vertex.normal().to_array(),
        )
    }
}

/// Vertex buffer which includes additional padding to fulfill the `wgpu::COPY_BUFFER_ALIGNMENT`.
#[derive(Clone)]
pub struct OverAlignedVertexBuffer<V, I> {
    pub buffer: VertexBuffers<V, I>,
    pub usable_indices: u32,
}

impl<V, I> OverAlignedVertexBuffer<V, I> {
    pub fn empty() -> Self {
        Self {
            buffer: VertexBuffers::with_capacity(0, 0),
            usable_indices: 0,
        }
    }

    pub fn from_slices(vertices: &[V], indices: &[I], usable_indices: u32) -> Self
    where
        V: Copy,
        I: Copy,
    {
        // FIXME (wasm-executor), make this fn not needed
        let mut buffers = VertexBuffers::with_capacity(0, 0);
        buffers.vertices = Vec::from(vertices);
        buffers.indices = Vec::from(indices);
        Self {
            buffer: buffers,
            usable_indices,
        }
    }
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
        let stride = std::mem::size_of::<ShaderVertex>() as wgpu::BufferAddress;
        let unpadded_bytes = self.vertices.len() as wgpu::BufferAddress * stride;
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
        let stride = std::mem::size_of::<I>() as wgpu::BufferAddress;
        let unpadded_bytes = self.indices.len() as wgpu::BufferAddress * stride;
        let padding_bytes = (align - unpadded_bytes % align) % align;
        let overpad = (padding_bytes + stride - 1) / stride; // Divide by stride but round up

        for _ in 0..overpad {
            self.indices.push(I::zeroed());
        }
    }
}
