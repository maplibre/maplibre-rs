use std::any::Any;
use std::collections::vec_deque::Iter;
use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use std::marker::PhantomData;
use std::mem::size_of;
use std::ops::Range;

use lyon::tessellation::VertexBuffers;

use crate::io::TileCoords;
use crate::render::shader_ffi::GpuVertexUniform;
use crate::tesselation::IndexDataType;

/// Buffer and its size
pub struct BackingBufferDescriptor<B>(pub B, pub wgpu::BufferAddress);

pub trait Queue<B> {
    fn write_buffer(&self, buffer: &B, offset: wgpu::BufferAddress, data: &[u8]);
}

impl Queue<wgpu::Buffer> for wgpu::Queue {
    fn write_buffer(&self, buffer: &wgpu::Buffer, offset: wgpu::BufferAddress, data: &[u8]) {
        self.write_buffer(buffer, offset, data)
    }
}

/// This is inspired by the memory pool in Vulkan documented
/// [here](https://gpuopen-librariesandsdks.github.io/VulkanMemoryAllocator/html/custom_memory_pools.html).
#[derive(Debug)]
pub struct BufferPool<Q, B, V, I> {
    vertices: BackingBuffer<B>,
    indices: BackingBuffer<B>,

    pub index: VecDeque<IndexEntry>,
    phantom_v: PhantomData<V>,
    phantom_i: PhantomData<I>,
    phantom_q: PhantomData<Q>,
}

impl<Q: Queue<B>, B, V: bytemuck::Pod, I: bytemuck::Pod> BufferPool<Q, B, V, I> {
    pub fn new(vertices: BackingBufferDescriptor<B>, indices: BackingBufferDescriptor<B>) -> Self {
        Self {
            vertices: BackingBuffer::new(vertices.0, vertices.1),
            indices: BackingBuffer::new(indices.0, indices.1),
            index: VecDeque::new(), // TODO: Approximate amount of buffers in pool
            phantom_v: Default::default(),
            phantom_i: Default::default(),
            phantom_q: Default::default(),
        }
    }

    fn available_space(&self, vertices: bool) -> wgpu::BufferAddress {
        let gap = self.vertices.find_gap(&self.index, vertices);
        gap.end - gap.start
    }
    pub fn vertices(&self) -> &B {
        &self.vertices.inner
    }

    pub fn indices(&self) -> &B {
        &self.indices.inner
    }

    /// Allocates `buffer` and uploads it to the GPU
    pub fn allocate_geometry(
        &mut self,
        queue: &Q,
        id: u32,
        coords: TileCoords,
        geometry: &VertexBuffers<V, I>,
    ) {
        let vertices_stride = size_of::<V>();
        let new_vertices = (geometry.vertices.len() * vertices_stride) as wgpu::BufferAddress;
        let indices_stride = size_of::<I>();
        let new_indices = (geometry.indices.len() * indices_stride) as wgpu::BufferAddress;

        let maybe_entry = IndexEntry {
            id,
            coords,
            indices_stride: indices_stride as u64,
            vertices: self.vertices.make_room(new_vertices, &mut self.index, true),
            indices: self.indices.make_room(new_indices, &mut self.index, false),
        };

        assert_eq!(
            maybe_entry.vertices.end - &maybe_entry.vertices.start,
            new_vertices
        );
        assert_eq!(
            maybe_entry.indices.end - &maybe_entry.indices.start,
            new_indices
        );

        queue.write_buffer(
            &self.vertices.inner,
            maybe_entry.vertices.start,
            bytemuck::cast_slice(&geometry.vertices),
        );
        queue.write_buffer(
            &self.indices.inner,
            maybe_entry.indices.start,
            bytemuck::cast_slice(&geometry.indices),
        );
        self.index.push_back(maybe_entry);
    }

    pub fn available_vertices(&self) -> Iter<'_, IndexEntry> {
        self.index.iter()
    }
}

#[derive(Debug)]
struct BackingBuffer<B> {
    /// The internal structure which is used for storage
    inner: B,
    /// The size of the `inner` buffer
    inner_size: wgpu::BufferAddress,
    /// The offset within `inner`
    inner_offset: wgpu::BufferAddress,
}

impl<B> BackingBuffer<B> {
    fn new(inner: B, inner_size: wgpu::BufferAddress) -> Self {
        Self {
            inner,
            inner_size,
            inner_offset: 0,
        }
    }

    fn make_room(
        &mut self,
        new_data: wgpu::BufferAddress,
        index: &mut VecDeque<IndexEntry>,
        vertices: bool,
    ) -> Range<wgpu::BufferAddress> {
        if new_data > self.inner_size {
            panic!("can not allocate because backing buffers are too small")
        }

        let mut available_gap = self.find_gap(index, vertices);

        while new_data > available_gap.end - available_gap.start {
            // no more space, we need to evict items
            if let Some(_) = index.pop_front() {
                available_gap = self.find_gap(index, vertices);
            } else {
                panic!("evicted even though index is empty")
            }
        }

        available_gap.start..available_gap.start + new_data
    }

    fn find_gap(&self, index: &VecDeque<IndexEntry>, vertices: bool) -> Range<wgpu::BufferAddress> {
        let start = index
            .front()
            .map(|first| {
                if vertices {
                    first.vertices.start
                } else {
                    first.indices.start
                }
            })
            .unwrap_or(0);
        let end = index
            .back()
            .map(|first| {
                if vertices {
                    first.vertices.end
                } else {
                    first.indices.end
                }
            })
            .unwrap_or(0);

        // as soon as we have a gap in between choose that one
        if end >= start {
            end..self.inner_size
        } else {
            start..end
        }
    }
}

#[derive(Debug)]
pub struct IndexEntry {
    pub id: u32,
    pub coords: TileCoords,
    indices_stride: u64,
    vertices: Range<wgpu::BufferAddress>,
    indices: Range<wgpu::BufferAddress>,
}

impl IndexEntry {
    pub fn indices_range(&self) -> Range<u32> {
        0..((self.indices.end - self.indices.start) / self.indices_stride) as u32
    }

    pub fn indices_buffer_range(&self) -> Range<wgpu::BufferAddress> {
        self.indices.clone()
    }

    pub fn vertices_buffer_range(&self) -> Range<wgpu::BufferAddress> {
        self.vertices.clone()
    }
}

#[cfg(test)]
mod tests {
    use lyon::tessellation::VertexBuffers;
    use wgpu::BufferAddress;

    use crate::render::buffer_pool::{BackingBufferDescriptor, BufferPool, Queue};
    use crate::render::shader_ffi::GpuVertexUniform;

    #[derive(Debug)]
    struct TestBuffer {
        size: BufferAddress,
    }
    struct TestQueue;

    impl Queue<TestBuffer> for TestQueue {
        fn write_buffer(&self, buffer: &TestBuffer, offset: BufferAddress, data: &[u8]) {
            if offset + data.len() as BufferAddress > buffer.size {
                panic!("write out of bounds");
            }
        }
    }

    fn create_48byte() -> Vec<GpuVertexUniform> {
        vec![GpuVertexUniform::default(), GpuVertexUniform::default()]
    }

    fn create_24byte() -> Vec<GpuVertexUniform> {
        vec![GpuVertexUniform::default()]
    }

    #[test]
    fn test_allocate() {
        let mut pool: BufferPool<TestQueue, TestBuffer, GpuVertexUniform, u32> = BufferPool::new(
            BackingBufferDescriptor(TestBuffer { size: 128 }, 128),
            BackingBufferDescriptor(TestBuffer { size: 1024 }, 1024),
        );

        let queue = TestQueue {};

        let mut data = VertexBuffers::new();
        data.vertices.append(&mut create_48byte());
        data.indices.append(&mut vec![1, 2, 3, 4]);
        for i in 0..2 {
            pool.allocate_geometry(&queue, (0, 0, 0).into(), &data);
        }

        assert_eq!(128 - 2 * 48, pool.available_space(true));

        let mut data = VertexBuffers::new();
        data.vertices.append(&mut create_24byte());
        data.indices.append(&mut vec![1, 2, 3, 4]);
        pool.allocate_geometry(&queue, (0, 0, 0).into(), &data);

        assert_eq!(128 - 2 * 48 - 24, pool.available_space(true));
        //println!("{:?}", &pool.index)
    }
}
