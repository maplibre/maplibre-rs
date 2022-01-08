use std::any::Any;
use std::collections::vec_deque::Iter;
use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use std::marker::PhantomData;
use std::mem::size_of;
use std::ops::Range;

use lyon::tessellation::VertexBuffers;

use crate::coords::TileCoords;
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
        let gap = self.vertices.find_largest_gap(&self.index, vertices);
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

        // write_buffer() is the preferred method for WASM: https://toji.github.io/webgpu-best-practices/buffer-uploads.html#when-in-doubt-writebuffer
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

        let mut available_gap = self.find_largest_gap(index, vertices);

        while new_data > available_gap.end - available_gap.start {
            // no more space, we need to evict items
            if let Some(_) = index.pop_front() {
                available_gap = self.find_largest_gap(index, vertices);
            } else {
                panic!("evicted even though index is empty")
            }
        }

        available_gap.start..available_gap.start + new_data
    }

    fn find_largest_gap(
        &self,
        index: &VecDeque<IndexEntry>,
        vertices: bool,
    ) -> Range<wgpu::BufferAddress> {
        let start = index.front().map(|first| {
            if vertices {
                first.vertices.start
            } else {
                first.indices.start
            }
        });
        let end = index.back().map(|first| {
            if vertices {
                first.vertices.end
            } else {
                first.indices.end
            }
        });

        if let Some(start) = start {
            if let Some(end) = end {
                if end > start {
                    // we haven't wrapped yet in the ring buffer

                    let gap_from_start = 0..start; // gap from beginning to first entry
                    let gap_to_end = end..self.inner_size;

                    if gap_to_end.end - gap_to_end.start > gap_from_start.end - gap_from_start.start
                    {
                        gap_to_end
                    } else {
                        gap_from_start
                    }
                } else {
                    // we already wrapped in the ring buffer
                    // we choose the gab between the two
                    end..start
                }
            } else {
                unreachable!()
            }
        } else {
            0..self.inner_size
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

    #[repr(C)]
    #[derive(Default, Copy, Clone, bytemuck_derive::Pod, bytemuck_derive::Zeroable)]
    struct TestVertex {
        data: [u8; 24],
    }

    fn create_48byte() -> Vec<TestVertex> {
        vec![TestVertex::default(), TestVertex::default()]
    }

    fn create_24byte() -> Vec<TestVertex> {
        vec![TestVertex::default()]
    }

    #[test]
    fn test_allocate() {
        let mut pool: BufferPool<TestQueue, TestBuffer, TestVertex, u32> = BufferPool::new(
            BackingBufferDescriptor(TestBuffer { size: 128 }, 128),
            BackingBufferDescriptor(TestBuffer { size: 1024 }, 1024),
        );

        let queue = TestQueue {};

        let mut data48bytes = VertexBuffers::new();
        data48bytes.vertices.append(&mut create_48byte());
        data48bytes.indices.append(&mut vec![1, 2, 3, 4]);

        let mut data24bytes = VertexBuffers::new();
        data24bytes.vertices.append(&mut create_24byte());
        data24bytes.indices.append(&mut vec![1, 2, 3, 4]);

        for i in 0..2 {
            pool.allocate_geometry(&queue, 0, (0, 0, 0).into(), &data48bytes);
        }
        assert_eq!(128 - 2 * 48, pool.available_space(true));

        pool.allocate_geometry(&queue, 1, (0, 0, 0).into(), &data24bytes);
        assert_eq!(128 - 2 * 48 - 24, pool.available_space(true));
        println!("{:?}", &pool.index);

        pool.allocate_geometry(&queue, 1, (0, 0, 0).into(), &data24bytes);
        // appended now at the beginning
        println!("{:?}", &pool.index);
        assert_eq!(24, pool.available_space(true));

        pool.allocate_geometry(&queue, 1, (0, 0, 0).into(), &data24bytes);
        println!("{:?}", &pool.index);
        assert_eq!(0, pool.available_space(true));

        pool.allocate_geometry(&queue, 1, (0, 0, 0).into(), &data24bytes);
        println!("{:?}", &pool.index);
        assert_eq!(24, pool.available_space(true));

        pool.allocate_geometry(&queue, 1, (0, 0, 0).into(), &data24bytes);
        println!("{:?}", &pool.index);
        assert_eq!(0, pool.available_space(true));
    }
}
