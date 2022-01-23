use std::collections::vec_deque::Iter;
use std::collections::VecDeque;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::mem::size_of;
use std::ops::Range;

use wgpu::BufferAddress;

use crate::coords::TileCoords;
use crate::tesselation::OverAlignedVertexBuffer;

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
pub struct BufferPool<Q, B, V, I, M, FM> {
    vertices: BackingBuffer<B>,
    indices: BackingBuffer<B>,
    metadata: BackingBuffer<B>,
    feature_metadata: BackingBuffer<B>,

    pub index: VecDeque<IndexEntry>,
    phantom_v: PhantomData<V>,
    phantom_i: PhantomData<I>,
    phantom_q: PhantomData<Q>,
    phantom_m: PhantomData<M>,
    phantom_fm: PhantomData<FM>,
}

#[derive(Debug)]
enum BackingBufferType {
    Vertices,
    Indices,
    Metadata,
    FeatureMetadata,
}

impl<Q: Queue<B>, B, V: bytemuck::Pod, I: bytemuck::Pod, M: bytemuck::Pod, FM: bytemuck::Pod>
    BufferPool<Q, B, V, I, M, FM>
{
    pub fn new(
        vertices: BackingBufferDescriptor<B>,
        indices: BackingBufferDescriptor<B>,
        metadata: BackingBufferDescriptor<B>,
        feature_metadata: BackingBufferDescriptor<B>,
    ) -> Self {
        Self {
            vertices: BackingBuffer::new(
                vertices.buffer,
                vertices.inner_size,
                BackingBufferType::Vertices,
            ),
            indices: BackingBuffer::new(
                indices.buffer,
                indices.inner_size,
                BackingBufferType::Indices,
            ),
            metadata: BackingBuffer::new(
                metadata.buffer,
                metadata.inner_size,
                BackingBufferType::Metadata,
            ),
            feature_metadata: BackingBuffer::new(
                feature_metadata.buffer,
                feature_metadata.inner_size,
                BackingBufferType::FeatureMetadata,
            ),
            index: VecDeque::new(), // TODO: Approximate amount of buffers in pool
            phantom_v: Default::default(),
            phantom_i: Default::default(),
            phantom_q: Default::default(),
            phantom_m: Default::default(),
            phantom_fm: Default::default(),
        }
    }

    #[cfg(test)]
    fn available_space(&self, typ: BackingBufferType) -> wgpu::BufferAddress {
        let gap = match typ {
            BackingBufferType::Vertices => &self.vertices,
            BackingBufferType::Indices => &self.indices,
            BackingBufferType::Metadata => &self.metadata,
            BackingBufferType::FeatureMetadata => &self.feature_metadata,
        }
        .find_largest_gap(&self.index);

        gap.end - gap.start
    }

    pub fn vertices(&self) -> &B {
        &self.vertices.inner
    }

    pub fn indices(&self) -> &B {
        &self.indices.inner
    }

    pub fn metadata(&self) -> &B {
        &self.metadata.inner
    }

    pub fn feature_metadata(&self) -> &B {
        &self.feature_metadata.inner
    }

    /// The VertexBuffers can contain padding elements. Not everything from a VertexBuffers is useable.
    /// The function returns the `bytes` and `aligned_bytes`. See [`OverAlignedVertexBuffer`].
    fn align(
        stride: wgpu::BufferAddress,
        elements: wgpu::BufferAddress,
        usable_elements: wgpu::BufferAddress,
    ) -> (BufferAddress, BufferAddress) {
        let bytes = elements * stride;

        let usable_bytes = (usable_elements * stride) as wgpu::BufferAddress;

        let align = wgpu::COPY_BUFFER_ALIGNMENT;
        let padding = (align - usable_bytes % align) % align;

        let aligned_bytes = usable_bytes + padding;

        (bytes, aligned_bytes)
    }

    /// Allocates `buffer` and uploads it to the GPU
    pub fn allocate_geometry(
        &mut self,
        queue: &Q,
        coords: TileCoords,
        over_aligned: &OverAlignedVertexBuffer<V, I>,
        metadata: M,
        feature_metadata: &Vec<FM>,
    ) {
        let vertices_stride = size_of::<V>() as wgpu::BufferAddress;
        let indices_stride = size_of::<I>() as wgpu::BufferAddress;
        let metadata_stride = size_of::<M>() as wgpu::BufferAddress;
        let feature_metadata_stride = size_of::<FM>() as wgpu::BufferAddress;

        let (vertices_bytes, aligned_vertices_bytes) = Self::align(
            vertices_stride,
            over_aligned.buffer.vertices.len() as BufferAddress,
            over_aligned.buffer.vertices.len() as BufferAddress,
        );
        let (indices_bytes, aligned_indices_bytes) = Self::align(
            indices_stride,
            over_aligned.buffer.indices.len() as BufferAddress,
            over_aligned.usable_indices as BufferAddress,
        );
        let (metadata_bytes, aligned_metadata_bytes) = Self::align(metadata_stride, 1, 1);
        let (feature_metadata_bytes, aligned_feature_metadata_bytes) = Self::align(
            feature_metadata_stride,
            feature_metadata.len() as BufferAddress,
            feature_metadata.len() as BufferAddress,
        );

        if feature_metadata_bytes != aligned_feature_metadata_bytes {
            // FIXME: align if not aligned?
            panic!(
                "feature_metadata is not aligned. This should not happen as long as size_of::<FM>() is a multiple of the alignment."
            )
        }

        let maybe_entry = IndexEntry {
            coords,
            buffer_vertices: self.vertices.make_room(vertices_bytes, &mut self.index),
            buffer_indices: self.indices.make_room(indices_bytes, &mut self.index),
            usable_indices: over_aligned.usable_indices as u32,
            buffer_metadata: self.metadata.make_room(metadata_bytes, &mut self.index),
            buffer_feature_metadata: self
                .feature_metadata
                .make_room(feature_metadata_bytes, &mut self.index),
        };

        // write_buffer() is the preferred method for WASM: https://toji.github.io/webgpu-best-practices/buffer-uploads.html#when-in-doubt-writebuffer
        queue.write_buffer(
            &self.vertices.inner,
            maybe_entry.buffer_vertices.start,
            &bytemuck::cast_slice(&over_aligned.buffer.vertices)
                [0..aligned_vertices_bytes as usize],
        );
        queue.write_buffer(
            &self.indices.inner,
            maybe_entry.buffer_indices.start,
            &bytemuck::cast_slice(&over_aligned.buffer.indices)[0..aligned_indices_bytes as usize],
        );
        queue.write_buffer(
            &self.metadata.inner,
            maybe_entry.buffer_metadata.start,
            &bytemuck::cast_slice(&[metadata])[0..aligned_metadata_bytes as usize],
        );
        queue.write_buffer(
            &self.feature_metadata.inner,
            maybe_entry.buffer_feature_metadata.start,
            &bytemuck::cast_slice(feature_metadata.as_slice())
                [0..aligned_feature_metadata_bytes as usize],
        );
        self.index.push_back(maybe_entry);
    }

    pub fn available_vertices(&self) -> Iter<'_, IndexEntry> {
        self.index.iter()
    }
}

pub struct BackingBufferDescriptor<B> {
    /// The buffer which is used
    buffer: B,
    /// The size of buffer
    inner_size: wgpu::BufferAddress,
}

impl<B> BackingBufferDescriptor<B> {
    pub fn new(buffer: B, inner_size: wgpu::BufferAddress) -> Self {
        Self { buffer, inner_size }
    }
}

#[derive(Debug)]
struct BackingBuffer<B> {
    /// The internal structure which is used for storage
    inner: B,
    /// The size of the `inner` buffer
    inner_size: wgpu::BufferAddress,
    typ: BackingBufferType,
}

impl<B> BackingBuffer<B> {
    fn new(inner: B, inner_size: wgpu::BufferAddress, typ: BackingBufferType) -> Self {
        Self {
            inner,
            inner_size,
            typ,
        }
    }

    fn make_room(
        &mut self,
        new_data: wgpu::BufferAddress,
        index: &mut VecDeque<IndexEntry>,
    ) -> Range<wgpu::BufferAddress> {
        if new_data > self.inner_size {
            panic!("can not allocate because backing buffers are too small")
        }

        let mut available_gap = self.find_largest_gap(index);

        while new_data > available_gap.end - available_gap.start {
            // no more space, we need to evict items
            if index.pop_front().is_some() {
                available_gap = self.find_largest_gap(index);
            } else {
                panic!("evicted even though index is empty")
            }
        }

        available_gap.start..available_gap.start + new_data
    }

    fn find_largest_gap(&self, index: &VecDeque<IndexEntry>) -> Range<wgpu::BufferAddress> {
        let start = index.front().map(|first| match self.typ {
            BackingBufferType::Vertices => first.buffer_vertices.start,
            BackingBufferType::Indices => first.buffer_indices.start,
            BackingBufferType::Metadata => first.buffer_metadata.start,
            BackingBufferType::FeatureMetadata => first.buffer_feature_metadata.start,
        });
        let end = index.back().map(|first| match self.typ {
            BackingBufferType::Vertices => first.buffer_vertices.end,
            BackingBufferType::Indices => first.buffer_indices.end,
            BackingBufferType::Metadata => first.buffer_metadata.end,
            BackingBufferType::FeatureMetadata => first.buffer_feature_metadata.end,
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
    pub coords: TileCoords,
    // Range of bytes within the backing buffer for vertices
    buffer_vertices: Range<wgpu::BufferAddress>,
    // Range of bytes within the backing buffer for indices
    buffer_indices: Range<wgpu::BufferAddress>,
    // Range of bytes within the backing buffer for metadata
    buffer_metadata: Range<wgpu::BufferAddress>,
    // Range of bytes within the backing buffer for feature metadata
    buffer_feature_metadata: Range<wgpu::BufferAddress>,
    // Amount of actually usable indices. Each index has the size/format `IndexDataType`.
    // Can be lower than size(buffer_indices) / indices_stride because of alignment.
    usable_indices: u32,
}

impl IndexEntry {
    pub fn indices_range(&self) -> Range<u32> {
        0..self.usable_indices
    }

    pub fn indices_buffer_range(&self) -> Range<wgpu::BufferAddress> {
        self.buffer_indices.clone()
    }

    pub fn vertices_buffer_range(&self) -> Range<wgpu::BufferAddress> {
        self.buffer_vertices.clone()
    }

    pub fn metadata_buffer_range(&self) -> Range<wgpu::BufferAddress> {
        self.buffer_metadata.clone()
    }

    pub fn feature_metadata_buffer_range(&self) -> Range<wgpu::BufferAddress> {
        self.buffer_feature_metadata.clone()
    }
}

#[cfg(test)]
mod tests {
    use lyon::tessellation::VertexBuffers;
    use wgpu::BufferAddress;

    use crate::render::buffer_pool::{
        BackingBufferDescriptor, BackingBufferType, BufferPool, Queue,
    };

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
        let mut pool: BufferPool<TestQueue, TestBuffer, TestVertex, u32, u32, u32> =
            BufferPool::new(
                BackingBufferDescriptor::new(TestBuffer { size: 128 }, 128),
                BackingBufferDescriptor::new(TestBuffer { size: 128 }, 128),
                BackingBufferDescriptor::new(TestBuffer { size: 128 }, 128),
                BackingBufferDescriptor::new(TestBuffer { size: 128 }, 128),
            );

        let queue = TestQueue {};

        let mut data48bytes = VertexBuffers::new();
        data48bytes.vertices.append(&mut create_48byte());
        data48bytes.indices.append(&mut vec![1, 2, 3, 4]);
        let data48bytes_aligned = data48bytes.into();

        let mut data24bytes = VertexBuffers::new();
        data24bytes.vertices.append(&mut create_24byte());
        data24bytes.indices.append(&mut vec![1, 2, 3, 4]);
        let data24bytes_aligned = data24bytes.into();

        for _ in 0..2 {
            pool.allocate_geometry(&queue, (0, 0, 0).into(), &data48bytes_aligned, 2, &vec![]);
        }
        assert_eq!(
            128 - 2 * 48,
            pool.available_space(BackingBufferType::Vertices)
        );

        pool.allocate_geometry(&queue, (0, 0, 0).into(), &data24bytes_aligned, 2, &vec![]);
        assert_eq!(
            128 - 2 * 48 - 24,
            pool.available_space(BackingBufferType::Vertices)
        );
        println!("{:?}", &pool.index);

        pool.allocate_geometry(&queue, (0, 0, 0).into(), &data24bytes_aligned, 2, &vec![]);
        // appended now at the beginning
        println!("{:?}", &pool.index);
        assert_eq!(24, pool.available_space(BackingBufferType::Vertices));

        pool.allocate_geometry(&queue, (0, 0, 0).into(), &data24bytes_aligned, 2, &vec![]);
        println!("{:?}", &pool.index);
        assert_eq!(0, pool.available_space(BackingBufferType::Vertices));

        pool.allocate_geometry(&queue, (0, 0, 0).into(), &data24bytes_aligned, 2, &vec![]);
        println!("{:?}", &pool.index);
        assert_eq!(24, pool.available_space(BackingBufferType::Vertices));

        pool.allocate_geometry(&queue, (0, 0, 0).into(), &data24bytes_aligned, 2, &vec![]);
        println!("{:?}", &pool.index);
        assert_eq!(0, pool.available_space(BackingBufferType::Vertices));
    }
}
