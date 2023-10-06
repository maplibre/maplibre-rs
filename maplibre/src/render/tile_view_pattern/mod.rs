//! Utility for generating a tile pattern which can be used for masking.

mod pattern;

use std::{marker::PhantomData, mem::size_of, ops::Range};

use cgmath::Matrix4;
pub use pattern::{TileViewPattern, DEFAULT_TILE_VIEW_PATTERN_SIZE};

use crate::{
    coords::{WorldTileCoords, Zoom},
    render::shaders::ShaderTileMetadata,
    tcs::{resources::ResourceQuery, world::World},
};

pub type WgpuTileViewPattern = TileViewPattern<wgpu::Queue, wgpu::Buffer>;

/// If not otherwise specified, raster tiles usually are 512.0 by 512.0 pixel.
/// In order to support 256.0 x 256.0 raster tiles 256.0 must be used.
///
/// Vector tiles always have a size of 512.0.
pub const DEFAULT_TILE_SIZE: f64 = 512.0;

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

pub trait HasTile {
    fn has_tile(&self, coords: WorldTileCoords, world: &World) -> bool;

    fn get_available_parent(
        &self,
        coords: WorldTileCoords,
        world: &World,
    ) -> Option<WorldTileCoords> {
        let mut current = coords;
        loop {
            if self.has_tile(current, world) {
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
        coords: WorldTileCoords,
        world: &World,
        search_depth: usize,
    ) -> Option<Vec<WorldTileCoords>> {
        let mut children = coords.get_children().to_vec();

        let mut output = Vec::new();

        for _ in 0..search_depth {
            let mut new_children = Vec::with_capacity(children.len() * 4);

            for child in children {
                if self.has_tile(child, world) {
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

impl<A: HasTile> HasTile for &A {
    fn has_tile(&self, coords: WorldTileCoords, world: &World) -> bool {
        A::has_tile(*self, coords, world)
    }
}

impl<A: HasTile> HasTile for (A,) {
    fn has_tile(&self, coords: WorldTileCoords, world: &World) -> bool {
        self.0.has_tile(coords, world)
    }
}

impl<A: HasTile, B: HasTile> HasTile for (A, B) {
    fn has_tile(&self, coords: WorldTileCoords, world: &World) -> bool {
        self.0.has_tile(coords, world) && self.1.has_tile(coords, world)
    }
}

impl<A: HasTile, B: HasTile, C: HasTile> HasTile for (A, B, C) {
    fn has_tile(&self, coords: WorldTileCoords, world: &World) -> bool {
        self.0.has_tile(coords, world)
            && self.1.has_tile(coords, world)
            && self.2.has_tile(coords, world)
    }
}

pub struct QueryHasTile<Q> {
    phantom_q: PhantomData<Q>,
}

impl<Q: ResourceQuery> Default for QueryHasTile<Q> {
    fn default() -> Self {
        Self {
            phantom_q: Default::default(),
        }
    }
}

impl<Q: ResourceQuery> HasTile for QueryHasTile<Q>
where
    for<'a> Q::Item<'a>: HasTile,
{
    fn has_tile(&self, coords: WorldTileCoords, world: &World) -> bool {
        let resources = world
            .resources
            .query::<Q>()
            .expect("resource not found for has_tile check");

        resources.has_tile(coords, world)
    }
}

#[derive(Default)]
pub struct ViewTileSources {
    items: Vec<Box<dyn HasTile>>,
}

impl ViewTileSources {
    pub fn add<H: HasTile + 'static + Default>(&mut self) -> &mut Self {
        self.items.push(Box::<H>::default());
        self
    }

    pub fn add_resource_query<Q: ResourceQuery + 'static>(&mut self) -> &mut Self
    where
        for<'a> Q::Item<'a>: HasTile,
    {
        self.items.push(Box::new(QueryHasTile::<Q>::default()));
        self
    }

    pub fn clear(&mut self) {
        self.items.clear()
    }
}

impl HasTile for ViewTileSources {
    fn has_tile(&self, coords: WorldTileCoords, world: &World) -> bool {
        self.items.iter().all(|item| item.has_tile(coords, world))
    }
}
