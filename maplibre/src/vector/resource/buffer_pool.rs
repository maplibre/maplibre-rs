//! A ring-buffer like pool of [buffers](wgpu::Buffer).

use std::{
    collections::{btree_map, BTreeMap, HashSet, VecDeque},
    fmt::Debug,
    marker::PhantomData,
    mem::size_of,
    ops::Range,
};

use bytemuck::Pod;

use crate::{
    coords::{Quadkey, WorldTileCoords},
    render::{
        resource::{BackingBufferDescriptor, Queue},
        tile_view_pattern::HasTile,
    },
    style::layer::StyleLayer,
    tcs::world::World,
    tessellation::OverAlignedVertexBuffer,
};

// TODO: Too low values can cause a back-and-forth between unloading and loading layers
pub const VERTEX_SIZE: wgpu::BufferAddress = 10 * 1_000_000;
pub const INDICES_SIZE: wgpu::BufferAddress = 10 * 1_000_000;

pub const FEATURE_METADATA_SIZE: wgpu::BufferAddress = 10 * 1024 * 1000;
pub const LAYER_METADATA_SIZE: wgpu::BufferAddress = 10 * 1024;

/// This is inspired by the memory pool in Vulkan documented
/// [here](https://gpuopen-librariesandsdks.github.io/VulkanMemoryAllocator/html/custom_memory_pools.html).
#[derive(Debug)]
pub struct BufferPool<Q, B, V, I, TM, FM> {
    vertices: BackingBuffer<B>,
    indices: BackingBuffer<B>,
    layer_metadata: BackingBuffer<B>,
    feature_metadata: BackingBuffer<B>,

    index: RingIndex,
    phantom_v: PhantomData<V>,
    phantom_i: PhantomData<I>,
    phantom_q: PhantomData<Q>,
    phantom_m: PhantomData<TM>,
    phantom_fm: PhantomData<FM>,
}

#[derive(Clone, Copy, Debug)]
pub enum BackingBufferType {
    Vertices,
    Indices,
    Metadata,
    FeatureMetadata,
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
}

impl<V: Pod, I: Pod, TM: Pod, FM: Pod> BufferPool<wgpu::Queue, wgpu::Buffer, V, I, TM, FM> {
    pub fn from_device(device: &wgpu::Device) -> Self {
        let vertex_buffer_desc = wgpu::BufferDescriptor {
            label: Some("vertex buffer"),
            size: size_of::<V>() as wgpu::BufferAddress * VERTEX_SIZE,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        };

        let indices_buffer_desc = wgpu::BufferDescriptor {
            label: Some("indices buffer"),
            size: size_of::<I>() as wgpu::BufferAddress * INDICES_SIZE,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        };

        let feature_metadata_desc = wgpu::BufferDescriptor {
            label: Some("feature metadata buffer"),
            size: size_of::<FM>() as wgpu::BufferAddress * FEATURE_METADATA_SIZE,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        };

        let layer_metadata_desc = wgpu::BufferDescriptor {
            label: Some("layer metadata buffer"),
            size: size_of::<TM>() as wgpu::BufferAddress * LAYER_METADATA_SIZE,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        };

        BufferPool::new(
            BackingBufferDescriptor::new(
                device.create_buffer(&vertex_buffer_desc),
                vertex_buffer_desc.size,
            ),
            BackingBufferDescriptor::new(
                device.create_buffer(&indices_buffer_desc),
                indices_buffer_desc.size,
            ),
            BackingBufferDescriptor::new(
                device.create_buffer(&layer_metadata_desc),
                layer_metadata_desc.size,
            ),
            BackingBufferDescriptor::new(
                device.create_buffer(&feature_metadata_desc),
                feature_metadata_desc.size,
            ),
        )
    }
}
impl<Q: Queue<B>, B, V: Pod, I: Pod, TM: Pod, FM: Pod> BufferPool<Q, B, V, I, TM, FM> {
    pub fn new(
        vertices: BackingBufferDescriptor<B>,
        indices: BackingBufferDescriptor<B>,
        layer_metadata: BackingBufferDescriptor<B>,
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
            layer_metadata: BackingBuffer::new(
                layer_metadata.buffer,
                layer_metadata.inner_size,
                BackingBufferType::Metadata,
            ),
            feature_metadata: BackingBuffer::new(
                feature_metadata.buffer,
                feature_metadata.inner_size,
                BackingBufferType::FeatureMetadata,
            ),
            index: RingIndex::new(),
            phantom_v: Default::default(),
            phantom_i: Default::default(),
            phantom_q: Default::default(),
            phantom_m: Default::default(),
            phantom_fm: Default::default(),
        }
    }

    pub fn clear(&mut self) {
        self.index.clear()
    }

    #[cfg(test)]
    fn available_space(&self, typ: BackingBufferType) -> wgpu::BufferAddress {
        let gap = self.index.find_largest_gap(
            typ,
            match typ {
                BackingBufferType::Vertices => &self.vertices,
                BackingBufferType::Indices => &self.indices,
                BackingBufferType::Metadata => &self.layer_metadata,
                BackingBufferType::FeatureMetadata => &self.feature_metadata,
            }
            .inner_size,
        );

        gap.end - gap.start
    }

    pub fn vertices(&self) -> &B {
        &self.vertices.inner
    }

    pub fn indices(&self) -> &B {
        &self.indices.inner
    }

    pub fn metadata(&self) -> &B {
        &self.layer_metadata.inner
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
    ) -> (wgpu::BufferAddress, wgpu::BufferAddress) {
        let bytes = elements * stride;

        let usable_bytes = (usable_elements * stride) as wgpu::BufferAddress;

        let align = wgpu::COPY_BUFFER_ALIGNMENT;
        let padding = (align - usable_bytes % align) % align;

        let aligned_bytes = usable_bytes + padding;

        (bytes, aligned_bytes)
    }

    pub fn get_loaded_source_layers_at(&self, coords: WorldTileCoords) -> Option<HashSet<&str>> {
        self.index.get_layers(coords).map(|layers| {
            layers
                .iter()
                .map(|entry| entry.style_layer.source_layer.as_ref().unwrap().as_str()) // TODO: Remove unwrap
                .collect()
        })
    }

    /// Allocates
    /// * `geometry`
    /// * `layer_metadata` and
    /// * `feature_metadata` for a layer. This function is able to dynamically evict layers if there
    /// is not enough space available.
    #[tracing::instrument(skip_all)]
    pub fn allocate_layer_geometry(
        &mut self,
        queue: &Q,
        coords: WorldTileCoords,
        style_layer: StyleLayer,
        geometry: &OverAlignedVertexBuffer<V, I>,
        layer_metadata: TM,
        feature_metadata: &[FM],
    ) {
        let vertices_stride = size_of::<V>() as wgpu::BufferAddress;
        let indices_stride = size_of::<I>() as wgpu::BufferAddress;
        let layer_metadata_stride = size_of::<TM>() as wgpu::BufferAddress;
        let feature_metadata_stride = size_of::<FM>() as wgpu::BufferAddress;

        let (vertices_bytes, aligned_vertices_bytes) = Self::align(
            vertices_stride,
            geometry.buffer.vertices.len() as wgpu::BufferAddress,
            geometry.buffer.vertices.len() as wgpu::BufferAddress,
        );
        let (indices_bytes, aligned_indices_bytes) = Self::align(
            indices_stride,
            geometry.buffer.indices.len() as wgpu::BufferAddress,
            geometry.usable_indices as wgpu::BufferAddress,
        );
        let (layer_metadata_bytes, aligned_layer_metadata_bytes) =
            Self::align(layer_metadata_stride, 1, 1);

        let (feature_metadata_bytes, aligned_feature_metadata_bytes) = Self::align(
            feature_metadata_stride,
            feature_metadata.len() as wgpu::BufferAddress,
            feature_metadata.len() as wgpu::BufferAddress,
        );

        if feature_metadata_bytes != aligned_feature_metadata_bytes {
            // TODO: align if not aligned?
            panic!(
                "feature_metadata is not aligned. This should not happen as long as size_of::<FM>() is a multiple of the alignment."
            )
        }

        let maybe_entry = IndexEntry {
            coords,
            style_layer,
            buffer_vertices: self.index.make_room(
                vertices_bytes,
                self.vertices.typ,
                self.vertices.inner_size,
            ),
            buffer_indices: self.index.make_room(
                indices_bytes,
                self.indices.typ,
                self.indices.inner_size,
            ),
            usable_indices: geometry.usable_indices,
            buffer_layer_metadata: self.index.make_room(
                layer_metadata_bytes,
                self.layer_metadata.typ,
                self.layer_metadata.inner_size,
            ),
            buffer_feature_metadata: self.index.make_room(
                feature_metadata_bytes,
                self.feature_metadata.typ,
                self.feature_metadata.inner_size,
            ),
        };

        // write_buffer() is the preferred method for WASM: https://toji.github.io/webgpu-best-practices/buffer-uploads.html#when-in-doubt-writebuffer
        queue.write_buffer(
            &self.vertices.inner,
            maybe_entry.buffer_vertices.start,
            &bytemuck::cast_slice(&geometry.buffer.vertices)[0..aligned_vertices_bytes as usize],
        );

        queue.write_buffer(
            &self.indices.inner,
            maybe_entry.buffer_indices.start,
            &bytemuck::cast_slice(&geometry.buffer.indices)[0..aligned_indices_bytes as usize],
        );

        queue.write_buffer(
            &self.layer_metadata.inner,
            maybe_entry.buffer_layer_metadata.start,
            &bytemuck::cast_slice(&[layer_metadata])[0..aligned_layer_metadata_bytes as usize],
        );

        queue.write_buffer(
            &self.feature_metadata.inner,
            maybe_entry.buffer_feature_metadata.start,
            &bytemuck::cast_slice(feature_metadata)[0..aligned_feature_metadata_bytes as usize],
        );

        self.index.push_back(maybe_entry);
    }

    #[tracing::instrument(skip_all)]
    pub fn update_layer_metadata(&self, queue: &Q, entry: &IndexEntry, layer_metadata: TM) {
        let layer_metadata_stride = size_of::<TM>() as wgpu::BufferAddress; // TODO: deduplicate
        let (layer_metadata_bytes, aligned_layer_metadata_bytes) =
            Self::align(layer_metadata_stride, 1, 1);

        if entry.buffer_layer_metadata.end - entry.buffer_layer_metadata.start
            != layer_metadata_bytes
        {
            panic!("Updated layer metadata has wrong size!");
        }

        queue.write_buffer(
            &self.layer_metadata.inner,
            entry.buffer_layer_metadata.start,
            &bytemuck::cast_slice(&[layer_metadata])[0..aligned_layer_metadata_bytes as usize],
        );
    }

    #[tracing::instrument(skip_all)]
    pub fn update_feature_metadata(&self, queue: &Q, entry: &IndexEntry, feature_metadata: &[FM]) {
        let feature_metadata_stride = size_of::<FM>() as wgpu::BufferAddress; // TODO: deduplicate

        let (feature_metadata_bytes, aligned_feature_metadata_bytes) = Self::align(
            feature_metadata_stride,
            feature_metadata.len() as wgpu::BufferAddress,
            feature_metadata.len() as wgpu::BufferAddress,
        );

        if entry.buffer_feature_metadata.end - entry.buffer_feature_metadata.start
            != feature_metadata_bytes
        {
            panic!("Updated feature metadata has wrong size!");
        }

        if feature_metadata_bytes != aligned_feature_metadata_bytes {
            // FIXME: align if not aligned?
            panic!(
                "feature_metadata is not aligned. This should not happen as long as size_of::<FM>() is a multiple of the alignment."
            )
        }

        queue.write_buffer(
            &self.feature_metadata.inner,
            entry.buffer_feature_metadata.start,
            &bytemuck::cast_slice(feature_metadata)[0..aligned_feature_metadata_bytes as usize],
        );
    }

    pub fn index(&self) -> &RingIndex {
        &self.index
    }
}

#[derive(Debug, Clone)]
pub struct IndexEntry {
    pub coords: WorldTileCoords, // TODO: replace with generic key
    pub style_layer: StyleLayer, // TODO: remove
    // Range of bytes within the backing buffer for vertices
    buffer_vertices: Range<wgpu::BufferAddress>,
    // Range of bytes within the backing buffer for indices
    buffer_indices: Range<wgpu::BufferAddress>,
    // Range of bytes within the backing buffer for metadata
    buffer_layer_metadata: Range<wgpu::BufferAddress>,
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

    pub fn layer_metadata_buffer_range(&self) -> Range<wgpu::BufferAddress> {
        self.buffer_layer_metadata.clone()
    }

    pub fn feature_metadata_buffer_range(&self) -> Range<wgpu::BufferAddress> {
        self.buffer_feature_metadata.clone()
    }
}

#[derive(Debug)]
pub struct RingIndexEntry {
    layers: VecDeque<IndexEntry>,
}

#[derive(Debug)]
pub struct RingIndex {
    tree_index: BTreeMap<Quadkey, RingIndexEntry>,
    linear_index: VecDeque<Quadkey>,
}

impl RingIndex {
    pub fn new() -> Self {
        Self {
            tree_index: Default::default(),
            linear_index: Default::default(),
        }
    }

    pub fn clear(&mut self) {
        self.linear_index.clear();
        self.tree_index.clear();
    }

    pub fn front(&self) -> Option<&IndexEntry> {
        self.linear_index.front().and_then(|key| {
            self.tree_index
                .get(key)
                .and_then(|entry| entry.layers.front())
        })
    }

    pub fn back(&self) -> Option<&IndexEntry> {
        self.linear_index.back().and_then(|key| {
            self.tree_index
                .get(key)
                .and_then(|entry| entry.layers.back())
        })
    }

    pub fn get_layers(&self, coords: WorldTileCoords) -> Option<&VecDeque<IndexEntry>> {
        coords
            .build_quad_key()
            .and_then(|key| self.tree_index.get(&key))
            .map(|entry| &entry.layers)
    }

    pub fn iter(&self) -> impl Iterator<Item = impl Iterator<Item = &IndexEntry>> + '_ {
        self.linear_index
            .iter()
            .flat_map(|key| self.tree_index.get(key).map(|entry| entry.layers.iter()))
    }

    fn pop_front(&mut self) -> Option<IndexEntry> {
        if let Some(entry) = self
            .linear_index
            .pop_front()
            .and_then(|key| self.tree_index.get_mut(&key))
        {
            entry.layers.pop_front()
        } else {
            None
        }
    }

    fn push_back(&mut self, entry: IndexEntry) {
        if let Some(key) = entry.coords.build_quad_key() {
            match self.tree_index.entry(key) {
                btree_map::Entry::Vacant(index_entry) => {
                    index_entry.insert(RingIndexEntry {
                        layers: VecDeque::from([entry]),
                    });
                }
                btree_map::Entry::Occupied(mut index_entry) => {
                    index_entry.get_mut().layers.push_back(entry);
                }
            }

            self.linear_index.push_back(key)
        } else {
            unreachable!() // TODO handle
        }
    }

    fn make_room(
        &mut self,
        new_data: wgpu::BufferAddress,
        typ: BackingBufferType,
        inner_size: wgpu::BufferAddress,
    ) -> Range<wgpu::BufferAddress> {
        if new_data > inner_size {
            panic!("can not allocate because backing buffer {typ:?} are too small")
        }

        let mut available_gap = self.find_largest_gap(typ, inner_size);

        while new_data > available_gap.end - available_gap.start {
            // no more space, we need to evict items
            if self.pop_front().is_some() {
                available_gap = self.find_largest_gap(typ, inner_size);
            } else {
                panic!("evicted even though index is empty")
            }
        }

        available_gap.start..available_gap.start + new_data
    }

    fn find_largest_gap(
        &self,
        typ: BackingBufferType,
        inner_size: wgpu::BufferAddress,
    ) -> Range<wgpu::BufferAddress> {
        let start = self.front().map(|first| match typ {
            BackingBufferType::Vertices => first.buffer_vertices.start,
            BackingBufferType::Indices => first.buffer_indices.start,
            BackingBufferType::Metadata => first.buffer_layer_metadata.start,
            BackingBufferType::FeatureMetadata => first.buffer_feature_metadata.start,
        });
        let end = self.back().map(|first| match typ {
            BackingBufferType::Vertices => first.buffer_vertices.end,
            BackingBufferType::Indices => first.buffer_indices.end,
            BackingBufferType::Metadata => first.buffer_layer_metadata.end,
            BackingBufferType::FeatureMetadata => first.buffer_feature_metadata.end,
        });

        if let Some(start) = start {
            if let Some(end) = end {
                if end > start {
                    // we haven't wrapped yet in the ring buffer

                    let gap_from_start = 0..start; // gap from beginning to first entry
                    let gap_to_end = end..inner_size;

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
            0..inner_size
        }
    }
}

impl<Q: Queue<B>, B, V: Pod, I: Pod, TM: Pod, FM: Pod> HasTile for BufferPool<Q, B, V, I, TM, FM> {
    fn has_tile(&self, coords: WorldTileCoords, _world: &World) -> bool {
        self.index().get_layers(coords).is_some()
    }
}

impl Default for RingIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use lyon::tessellation::VertexBuffers;

    use crate::{
        coords::ZoomLevel,
        render::resource::{BackingBufferDescriptor, Queue},
        style::layer::StyleLayer,
        vector::resource::{BackingBufferType, BufferPool},
    };

    #[derive(Debug)]
    struct TestBuffer {
        size: wgpu::BufferAddress,
    }
    struct TestQueue;

    impl Queue<TestBuffer> for TestQueue {
        fn write_buffer(&self, buffer: &TestBuffer, offset: wgpu::BufferAddress, data: &[u8]) {
            if offset + data.len() as wgpu::BufferAddress > buffer.size {
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
        let style_layer = StyleLayer::default();

        let mut data48bytes = VertexBuffers::new();
        data48bytes.vertices.append(&mut create_48byte());
        data48bytes.indices.append(&mut vec![1, 2, 3, 4]);
        let data48bytes_aligned = data48bytes.into();

        let mut data24bytes = VertexBuffers::new();
        data24bytes.vertices.append(&mut create_24byte());
        data24bytes.indices.append(&mut vec![1, 2, 3, 4]);
        let data24bytes_aligned = data24bytes.into();

        for _ in 0..2 {
            pool.allocate_layer_geometry(
                &queue,
                (0, 0, ZoomLevel::default()).into(),
                style_layer.clone(),
                &data48bytes_aligned,
                2,
                &[],
            );
        }
        assert_eq!(
            128 - 2 * 48,
            pool.available_space(BackingBufferType::Vertices)
        );

        pool.allocate_layer_geometry(
            &queue,
            (0, 0, ZoomLevel::default()).into(),
            style_layer.clone(),
            &data24bytes_aligned,
            2,
            &[],
        );
        assert_eq!(
            128 - 2 * 48 - 24,
            pool.available_space(BackingBufferType::Vertices)
        );
        println!("{:?}", pool.index);

        pool.allocate_layer_geometry(
            &queue,
            (0, 0, ZoomLevel::default()).into(),
            style_layer.clone(),
            &data24bytes_aligned,
            2,
            &[],
        );
        // appended now at the beginning
        println!("{:?}", pool.index);
        assert_eq!(24, pool.available_space(BackingBufferType::Vertices));

        pool.allocate_layer_geometry(
            &queue,
            (0, 0, ZoomLevel::default()).into(),
            style_layer.clone(),
            &data24bytes_aligned,
            2,
            &[],
        );
        println!("{:?}", pool.index);
        assert_eq!(0, pool.available_space(BackingBufferType::Vertices));

        pool.allocate_layer_geometry(
            &queue,
            (0, 0, ZoomLevel::default()).into(),
            style_layer.clone(),
            &data24bytes_aligned,
            2,
            &[],
        );
        println!("{:?}", pool.index);
        assert_eq!(24, pool.available_space(BackingBufferType::Vertices));

        pool.allocate_layer_geometry(
            &queue,
            (0, 0, ZoomLevel::default()).into(),
            style_layer,
            &data24bytes_aligned,
            2,
            &[],
        );
        println!("{:?}", pool.index);
        assert_eq!(0, pool.available_space(BackingBufferType::Vertices));
    }
}
