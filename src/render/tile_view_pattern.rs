use crate::coords::{ViewRegion, WorldTileCoords, Zoom};

use crate::render::buffer_pool::{BackingBufferDescriptor, BufferPool, Queue};
use crate::render::camera::ViewProjection;
use crate::render::shaders::{ShaderFeatureStyle, ShaderLayerMetadata, ShaderTileMetadata};
use cgmath::Matrix4;

use crate::render::ShaderVertex;
use crate::tessellation::IndexDataType;
use std::marker::PhantomData;
use std::mem::size_of;
use std::ops::Range;
use wgpu::Buffer;

/// The tile mask pattern assigns each tile a value which can be used for stencil testing.
pub struct TileViewPattern<Q, B> {
    in_view: Vec<TileInView>,
    buffer: BackingBuffer<B>,
    phantom_q: PhantomData<Q>,
}

pub struct TileShape {
    pub zoom_factor: f64,

    pub coords: WorldTileCoords,

    pub transform: Matrix4<f64>,
    pub buffer_range: Range<wgpu::BufferAddress>,
}

impl TileShape {
    fn new(coords: WorldTileCoords, zoom: Zoom, index: u64) -> Self {
        const STRIDE: u64 = size_of::<ShaderTileMetadata>() as u64;
        Self {
            coords,
            zoom_factor: zoom.scale_to_tile(&coords),
            transform: coords.transform_for_zoom(zoom),
            buffer_range: index as u64 * STRIDE..(index as u64 + 1) * STRIDE,
        }
    }
}

pub struct TileInView {
    pub shape: TileShape,

    pub fallback: Option<TileShape>,
}

#[derive(Debug)]
struct BackingBuffer<B> {
    /// The internal structure which is used for storage
    inner: B,
    /// The size of the `inner` buffer
    inner_size: wgpu::BufferAddress,
}

impl<B> BackingBuffer<B> {
    fn new(inner: B, inner_size: wgpu::BufferAddress) -> Self {
        Self { inner, inner_size }
    }
}

impl<Q: Queue<B>, B> TileViewPattern<Q, B> {
    pub fn new(buffer: BackingBufferDescriptor<B>) -> Self {
        Self {
            in_view: Vec::with_capacity(64),
            buffer: BackingBuffer::new(buffer.buffer, buffer.inner_size),
            phantom_q: Default::default(),
        }
    }

    #[tracing::instrument(skip_all)]
    pub fn update_pattern(
        &mut self,
        view_region: &ViewRegion,
        buffer_pool: &BufferPool<
            wgpu::Queue,
            Buffer,
            ShaderVertex,
            IndexDataType,
            ShaderLayerMetadata,
            ShaderFeatureStyle,
        >,
        zoom: Zoom,
    ) {
        self.in_view.clear();

        let mut index = 0;

        let pool_index = buffer_pool.index();

        for coords in view_region.iter() {
            if coords.build_quad_key().is_none() {
                continue;
            }

            let shape = TileShape::new(coords, zoom, index);

            index += 1;

            let fallback = {
                if !pool_index.has_tile(&coords) {
                    if let Some(fallback_coords) = pool_index.get_tile_coords_fallback(&coords) {
                        tracing::trace!(
                            "Could not find data at {coords}. Falling back to {fallback_coords}"
                        );

                        let shape = TileShape::new(fallback_coords, zoom, index);

                        index += 1;
                        Some(shape)
                    } else {
                        None
                    }
                } else {
                    None
                }
            };

            self.in_view.push(TileInView { shape, fallback });
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &TileInView> + '_ {
        self.in_view.iter()
    }

    pub fn buffer(&self) -> &B {
        &self.buffer.inner
    }

    #[tracing::instrument(skip_all)]
    pub fn upload_pattern(&self, queue: &Q, view_proj: &ViewProjection) {
        let mut buffer = Vec::with_capacity(self.in_view.len());

        for tile in &self.in_view {
            buffer.push(ShaderTileMetadata {
                // We are casting here from 64bit to 32bit, because 32bit is more performant and is
                // better supported.
                transform: view_proj
                    .to_model_view_projection(tile.shape.transform)
                    .downcast()
                    .into(),
                zoom_factor: tile.shape.zoom_factor as f32,
            });

            if let Some(fallback_shape) = &tile.fallback {
                buffer.push(ShaderTileMetadata {
                    // We are casting here from 64bit to 32bit, because 32bit is more performant and is
                    // better supported.
                    transform: view_proj
                        .to_model_view_projection(fallback_shape.transform)
                        .downcast()
                        .into(),
                    zoom_factor: fallback_shape.zoom_factor as f32,
                });
            }
        }

        queue.write_buffer(
            &self.buffer.inner,
            0,
            bytemuck::cast_slice(buffer.as_slice()),
        );
    }

    pub fn stencil_reference_value(&self, world_coords: &WorldTileCoords) -> u8 {
        world_coords.z * 5
            + match (world_coords.x, world_coords.y) {
                (x, y) if x % 2 == 0 && y % 2 == 0 => 2,
                (x, y) if x % 2 == 0 && y % 2 != 0 => 1,
                (x, y) if x % 2 != 0 && y % 2 == 0 => 4,
                (x, y) if x % 2 != 0 && y % 2 != 0 => 3,
                _ => unreachable!(),
            }
    }
}
