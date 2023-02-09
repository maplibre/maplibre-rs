//! Utility for generating a tile pattern which can be used for masking.

use std::{marker::PhantomData, mem::size_of, ops::Range};

use cgmath::Matrix4;
use log::warn;

use crate::{
    coords::{ViewRegion, WorldTileCoords, Zoom},
    render::{
        camera::ViewProjection,
        resource::{BackingBufferDescriptor, Queue},
        shaders::ShaderTileMetadata,
    },
};

pub const DEFAULT_TILE_VIEW_PATTERN_SIZE: wgpu::BufferAddress = 32 * 4;
pub const CHILDREN_SEARCH_DEPTH: usize = 4;

pub trait HasTile {
    fn has_tile(&self, coords: &WorldTileCoords) -> bool;

    fn get_available_parent(&self, coords: &WorldTileCoords) -> Option<WorldTileCoords> {
        let mut current = *coords;
        loop {
            if self.has_tile(&current) {
                return Some(current);
            } else if let Some(parent) = current.get_parent() {
                current = parent
            } else {
                return None;
            }
        }
    }

    fn get_available_children(
        &self,
        coords: &WorldTileCoords,
        search_depth: usize,
    ) -> Option<Vec<WorldTileCoords>> {
        let mut children = coords.get_children().to_vec();

        let mut output = Vec::new();

        for _ in 0..search_depth {
            let mut new_children = Vec::with_capacity(children.len() * 4);

            for child in children {
                if self.has_tile(&child) {
                    output.push(child);
                } else {
                    new_children.extend(child.get_children())
                }
            }

            children = new_children;
        }

        Some(output)
    }
}

impl<A: HasTile, B: HasTile> HasTile for (&A, &B) {
    fn has_tile(&self, coords: &WorldTileCoords) -> bool {
        self.0.has_tile(coords) || self.1.has_tile(coords)
    }
}

/// Defines the exact location where a specific tile on the map is rendered. It defines the shape
/// of the tile with its location for the current zoom factor.
#[derive(Debug, Clone)]
pub struct TileShape {
    coords: WorldTileCoords,

    // TODO: optimization, `zoom_factor` and `transform` are no longer required if `buffer_range` is Some()
    zoom_factor: f64,
    transform: Matrix4<f64>,

    buffer_range: Option<Range<wgpu::BufferAddress>>,
}

impl TileShape {
    fn new(coords: WorldTileCoords, zoom: Zoom) -> Self {
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

    pub fn buffer_range(&self) -> Option<Range<wgpu::BufferAddress>> {
        self.buffer_range.clone()
    }

    pub fn coords(&self) -> WorldTileCoords {
        self.coords
    }
}

/// This defines the source tile shaped from which the content for the `target` is taken.
/// For example if the target is `(0, 0, 1)` (of [`ViewTile`]) , we might use
/// `SourceShapes::Parent((0, 0, 0))` as source.
/// Similarly if we have the target `(0, 0, 0)` we might use
/// `SourceShapes::Children((0, 0, 1), (0, 1, 1), (1, 0, 1), (1, 1, 1))` as sources.
#[derive(Debug, Clone)]
pub enum SourceShapes {
    /// Parent tile is the source. We construct the `target` from parts of a parent.
    Parent(TileShape),
    /// Children are the source. We construct the `target` from multiple children.
    Children(Vec<TileShape>),
    /// Source and target are equal, so no need to differentiate. We render the `source` shape
    /// exactly at the `target`.
    SourceEqTarget(TileShape),
    /// No data available so nothing to render
    None,
}

/// Defines the `target` tile and its `source` from which data tile data comes.
#[derive(Debug, Clone)]
pub struct ViewTile {
    target: WorldTileCoords,
    source: SourceShapes,
}

impl ViewTile {
    pub fn coords(&self) -> WorldTileCoords {
        self.target
    }

    pub fn render<F>(&self, mut callback: F)
    where
        F: FnMut(&TileShape),
    {
        match &self.source {
            SourceShapes::Parent(source_shape) => callback(source_shape),
            SourceShapes::Children(source_shapes) => {
                for shape in source_shapes {
                    callback(shape)
                }
            }
            SourceShapes::SourceEqTarget(source_shape) => callback(source_shape),
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

/// The tile mask pattern assigns each tile a value which can be used for stencil testing.
pub struct TileViewPattern<Q, B> {
    view_tiles: Vec<ViewTile>,
    view_tiles_buffer: BackingBuffer<B>,
    phantom_q: PhantomData<Q>,
}

impl<Q: Queue<B>, B> TileViewPattern<Q, B> {
    pub fn new(view_tiles_buffer: BackingBufferDescriptor<B>) -> Self {
        Self {
            view_tiles: Vec::with_capacity(64),
            view_tiles_buffer: BackingBuffer::new(
                view_tiles_buffer.buffer,
                view_tiles_buffer.inner_size,
            ),
            phantom_q: Default::default(),
        }
    }

    #[tracing::instrument(skip_all)]
    pub fn update_pattern<T: HasTile>(
        &mut self,
        view_region: &ViewRegion,
        container: &T,
        zoom: Zoom,
    ) {
        self.view_tiles.clear();

        for coords in view_region.iter() {
            if coords.build_quad_key().is_none() {
                continue;
            }

            let source_shapes = {
                if container.has_tile(&coords) {
                    SourceShapes::SourceEqTarget(TileShape::new(coords, zoom))
                } else if let Some(parent_coords) = container.get_available_parent(&coords) {
                    log::info!("Could not find data at {coords}. Falling back to {parent_coords}");

                    SourceShapes::Parent(TileShape::new(parent_coords, zoom))
                } else if let Some(children_coords) =
                    container.get_available_children(&coords, CHILDREN_SEARCH_DEPTH)
                {
                    log::info!(
                        "Could not find data at {coords}. Falling back children: {children_coords:?}"
                    );

                    SourceShapes::Children(
                        children_coords
                            .iter()
                            .map(|child_coord| TileShape::new(*child_coord, zoom))
                            .collect(),
                    )
                } else {
                    SourceShapes::None
                }
            };

            self.view_tiles.push(ViewTile {
                target: coords,
                source: source_shapes,
            });
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &ViewTile> + '_ {
        self.view_tiles.iter()
    }

    pub fn buffer(&self) -> &B {
        &self.view_tiles_buffer.inner
    }

    #[tracing::instrument(skip_all)]
    pub fn upload_pattern(&mut self, queue: &Q, view_proj: &ViewProjection) {
        let mut buffer = Vec::with_capacity(self.view_tiles.len());

        let mut add_to_buffer = |shape: &mut TileShape| {
            shape.set_buffer_range(buffer.len() as u64);
            // TODO: Name `ShaderTileMetadata` is unfortunate here, because for raster rendering it actually is a layer
            buffer.push(ShaderTileMetadata {
                // We are casting here from 64bit to 32bit, because 32bit is more performant and is
                // better supported.
                transform: view_proj
                    .to_model_view_projection(shape.transform)
                    .downcast()
                    .into(), // TODO: move this calculation to update() fn above
                zoom_factor: shape.zoom_factor as f32,
            });
        };

        for view_tile in &mut self.view_tiles {
            match &mut view_tile.source {
                SourceShapes::Parent(source_shape) => {
                    add_to_buffer(source_shape);
                }
                SourceShapes::Children(source_shapes) => {
                    for source_shape in source_shapes {
                        add_to_buffer(source_shape);
                    }
                }
                SourceShapes::SourceEqTarget(source_shape) => add_to_buffer(source_shape),
                SourceShapes::None => {}
            }
        }

        let raw_buffer = bytemuck::cast_slice(buffer.as_slice());
        if raw_buffer.len() as wgpu::BufferAddress > self.view_tiles_buffer.inner_size {
            /* TODO: We need to avoid this case by either choosing a proper size
            TODO: (DEFAULT_TILE_VIEW_SIZE), or resizing the buffer */
            panic!("Buffer is too small to store the tile pattern!");
        }
        queue.write_buffer(&self.view_tiles_buffer.inner, 0, raw_buffer);
    }
}
