use std::{collections::HashSet, marker::PhantomData};

use crate::{
    coords::{ViewRegion, Zoom},
    render::{
        camera::ViewProjection,
        resource::{BackingBufferDescriptor, Queue},
        shaders::ShaderTileMetadata,
        tile_view_pattern::{HasTile, SourceShapes, TileShape, ViewTile},
    },
    tcs::world::World,
};

// FIXME: If network is very slow, this pattern size can
// increase dramatically.
// E.g. imagine if a pattern for zoom level 18 is drawn
// when completely zoomed out.
pub const DEFAULT_TILE_VIEW_PATTERN_SIZE: wgpu::BufferAddress = 512;
pub const CHILDREN_SEARCH_DEPTH: usize = 4;

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
    #[must_use]
    pub fn generate_pattern<T: HasTile>(
        &self,
        view_region: &ViewRegion,
        container: &T,
        zoom: Zoom,
        world: &World,
    ) -> Vec<ViewTile> {
        let mut view_tiles = Vec::with_capacity(self.view_tiles.len());
        let mut source_tiles = HashSet::new(); // TODO: Optimization potential: Replace wit a bitmap, that allows false-negative matches

        for coords in view_region.iter() {
            if coords.build_quad_key().is_none() {
                continue;
            }

            let source_shapes = {
                if container.has_tile(coords, world) {
                    SourceShapes::SourceEqTarget(TileShape::new(coords, zoom))
                } else if let Some(parent_coords) = container.get_available_parent(coords, world) {
                    log::debug!("Could not find data at {coords}. Falling back to {parent_coords}");

                    if source_tiles.contains(&parent_coords) {
                        // Performance optimization: Suppose the map only offers zoom levels 0-14.
                        // If we build the pattern for z=18, we won't find tiles. Thus we start
                        // looking for parents. We might find multiple times the same parent from
                        // tiles on z=18.
                        continue;
                    }

                    source_tiles.insert(parent_coords);

                    SourceShapes::Parent(TileShape::new(parent_coords, zoom))
                } else if let Some(children_coords) =
                    container.get_available_children(coords, world, CHILDREN_SEARCH_DEPTH)
                {
                    log::debug!(
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

            view_tiles.push(ViewTile {
                target: coords,
                source: source_shapes,
            });
        }

        view_tiles
    }

    pub fn update_pattern(&mut self, mut view_tiles: Vec<ViewTile>) {
        self.view_tiles.clear();
        self.view_tiles.append(&mut view_tiles)
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
            TODO: (DEFAULT_TILE_VIEW_PATTERN_SIZE), or resizing the buffer */
            panic!("Buffer is too small to store the tile pattern!");
        }
        queue.write_buffer(&self.view_tiles_buffer.inner, 0, raw_buffer);
    }
}
