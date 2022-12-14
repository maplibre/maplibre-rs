//! Utility for generating a tile pattern which can be used for masking.

use std::{marker::PhantomData, mem::size_of, ops::Range};

use cgmath::Matrix4;

use crate::{
    coords::{ViewRegion, WorldTileCoords, Zoom},
    render::{
        camera::ViewProjection,
        resource::{BackingBufferDescriptor, BufferPool, Queue},
        shaders::{ShaderFeatureStyle, ShaderLayerMetadata, ShaderTileMetadata},
        ShaderVertex,
    },
    tessellation::IndexDataType,
};

pub const DEFAULT_TILE_VIEW_PATTERN_SIZE: wgpu::BufferAddress = 32 * 4;

/// The tile mask pattern assigns each tile a value which can be used for stencil testing.
pub struct TileViewPattern<Q, B> {
    in_view: Vec<ViewTile>,
    buffer: BackingBuffer<B>,
    phantom_q: PhantomData<Q>,
}

#[derive(Clone)]
pub struct TileShape {
    pub zoom_factor: f64,

    pub coords: WorldTileCoords,

    pub transform: Matrix4<f64>,
    pub buffer_range: Option<Range<wgpu::BufferAddress>>,
}

impl TileShape {
    fn new(coords: WorldTileCoords, zoom: Zoom) -> Self {
        const STRIDE: u64 = size_of::<ShaderTileMetadata>() as u64;
        Self {
            coords,
            zoom_factor: zoom.scale_to_tile(&coords),
            transform: coords.transform_for_zoom(zoom),
            buffer_range: None,
        }
    }

    fn set_buffer_range(&mut self, index: u64) {
        const STRIDE: u64 = size_of::<ShaderTileMetadata>() as u64;
        self.buffer_range = Some(index * STRIDE..(index + 1) * STRIDE);
    }

    pub fn buffer_range(&self) -> Range<wgpu::BufferAddress> {
        self.buffer_range.as_ref().unwrap().clone()
    }
}

#[derive(Clone)]
pub enum SourceShapes {
    /// Parent tile is the source
    Parent(TileShape),
    /// Children are the source
    Children(Vec<TileShape>),
    /// Source and Target are equal, so no need to differentiate
    SourceEqTarget,
    /// No data available so nothing to render
    None,
}

#[derive(Clone)]
pub struct ViewTile {
    target_shape: TileShape,
    source_shapes: SourceShapes,
}

impl ViewTile {
    pub fn coords(&self) -> WorldTileCoords {
        self.target_shape.coords
    }

    pub fn source_available(&self) -> bool {
        match &self.source_shapes {
            SourceShapes::Parent(_) => true,
            SourceShapes::Children(_) => true,
            SourceShapes::SourceEqTarget => true,
            SourceShapes::None => false,
        }
    }

    pub fn render<F>(&self, mut callback: F)
    where
        F: FnMut(&TileShape, &TileShape),
    {
        match &self.source_shapes {
            SourceShapes::Parent(shape) => callback(&self.target_shape, shape),
            SourceShapes::Children(shapes) => {
                for shape in shapes {
                    callback(&self.target_shape, shape)
                }
            }
            SourceShapes::SourceEqTarget => callback(&self.target_shape, &self.target_shape),
            SourceShapes::None => {}
        }
    }
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
            wgpu::Buffer,
            ShaderVertex,
            IndexDataType,
            ShaderLayerMetadata,
            ShaderFeatureStyle,
        >,
        zoom: Zoom,
    ) {
        self.in_view.clear();

        let pool_index = buffer_pool.index();

        for coords in view_region.iter() {
            if coords.build_quad_key().is_none() {
                continue;
            }

            let source_shapes = {
                if pool_index.has_tile(&coords) {
                    SourceShapes::SourceEqTarget
                } else if let Some(fallback_coords) = pool_index.get_tile_coords_fallback(&coords) {
                    tracing::trace!(
                        "Could not find data at {coords}. Falling back to {fallback_coords}"
                    );

                    SourceShapes::Parent(TileShape::new(fallback_coords, zoom))
                } else {
                    SourceShapes::None
                }
            };

            self.in_view.push(ViewTile {
                target_shape: TileShape::new(coords, zoom),
                source_shapes,
            });
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &ViewTile> + '_ {
        self.in_view.iter()
    }

    pub fn buffer(&self) -> &B {
        &self.buffer.inner
    }

    #[tracing::instrument(skip_all)]
    pub fn upload_pattern(&mut self, queue: &Q, view_proj: &ViewProjection) {
        let mut buffer = Vec::with_capacity(self.in_view.len());

        for view_tile in &mut self.in_view {
            view_tile.target_shape.set_buffer_range(buffer.len() as u64);
            buffer.push(ShaderTileMetadata {
                // We are casting here from 64bit to 32bit, because 32bit is more performant and is
                // better supported.
                transform: view_proj
                    .to_model_view_projection(view_tile.target_shape.transform)
                    .downcast()
                    .into(),
                zoom_factor: view_tile.target_shape.zoom_factor as f32,
            });

            match &mut view_tile.source_shapes {
                SourceShapes::Parent(target_shape) => {
                    target_shape.set_buffer_range(buffer.len() as u64);
                    buffer.push(ShaderTileMetadata {
                        // We are casting here from 64bit to 32bit, because 32bit is more performant and is
                        // better supported.
                        transform: view_proj
                            .to_model_view_projection(target_shape.transform)
                            .downcast()
                            .into(),
                        zoom_factor: target_shape.zoom_factor as f32,
                    });
                }
                SourceShapes::Children(_) => {
                    unimplemented!()
                }
                SourceShapes::SourceEqTarget => {}
                SourceShapes::None => {}
            }
        }

        let raw_buffer = bytemuck::cast_slice(buffer.as_slice());
        if raw_buffer.len() as wgpu::BufferAddress > self.buffer.inner_size {
            /* TODO: We need to avoid this case by either choosing a proper size
            TODO: (DEFAULT_TILE_VIEW_SIZE), or resizing the buffer */
            panic!("Buffer is too small to store the tile pattern!");
        }
        queue.write_buffer(&self.buffer.inner, 0, raw_buffer);
    }

    pub fn stencil_reference_value(&self, world_coords: &WorldTileCoords) -> u8 {
        match (world_coords.x, world_coords.y) {
            (x, y) if x % 2 == 0 && y % 2 == 0 => 2,
            (x, y) if x % 2 == 0 && y % 2 != 0 => 1,
            (x, y) if x % 2 != 0 && y % 2 == 0 => 4,
            (x, y) if x % 2 != 0 && y % 2 != 0 => 3,
            _ => unreachable!(),
        }
    }
}
