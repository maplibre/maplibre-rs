//! Subdivided tile meshes used by globe-capable raster and stencil rendering.

use std::num::TryFromIntError;

use thiserror::Error;

use crate::coords::EXTENT_UINT;

/// Signed-Y marker used by shaders for vertices extending to the north pole.
pub const NORTH_POLE_Y: i16 = i16::MIN;
/// Signed-Y marker used by shaders for vertices extending to the south pole.
pub const SOUTH_POLE_Y: i16 = i16::MAX;

const EXTENT_STENCIL_BORDER: i16 = (EXTENT_UINT / 128) as i16;
const MAX_U16_VERTEX_COUNT: u64 = u16::MAX as u64 + 1;
const MAX_U32_VERTEX_COUNT: u64 = u32::MAX as u64 + 1;

/// A signed tile-local position suitable for direct GPU upload.
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, bytemuck_derive::Pod, bytemuck_derive::Zeroable)]
pub struct TileMeshVertex {
    /// Horizontal tile coordinate.
    pub x: i16,
    /// Vertical tile coordinate or pole marker.
    pub y: i16,
}

/// Requested index width for a tile mesh.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TileIndexType {
    /// Select 16-bit indices when the mesh fits, otherwise 32-bit indices.
    #[default]
    Auto,
    /// Require 16-bit indices.
    U16,
    /// Require 32-bit indices.
    U32,
}

/// Options controlling regular tile-grid generation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TileMeshOptions {
    /// Number of quads per tile axis. Zero is normalized to one.
    pub granularity: u32,
    /// Adds a stencil-safe ring around all non-pole edges.
    pub generate_borders: bool,
    /// Replaces the north border with vertices carrying [`NORTH_POLE_Y`].
    pub extend_to_north_pole: bool,
    /// Replaces the south border with vertices carrying [`SOUTH_POLE_Y`].
    pub extend_to_south_pole: bool,
}

impl Default for TileMeshOptions {
    fn default() -> Self {
        Self {
            granularity: 1,
            generate_borders: false,
            extend_to_north_pole: false,
            extend_to_south_pole: false,
        }
    }
}

/// Index storage selected for a generated tile mesh.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TileMeshIndices {
    /// Sixteen-bit triangle indices.
    U16(Vec<u16>),
    /// Thirty-two-bit triangle indices.
    U32(Vec<u32>),
}

impl TileMeshIndices {
    /// Returns the number of stored indices.
    pub fn len(&self) -> usize {
        match self {
            Self::U16(indices) => indices.len(),
            Self::U32(indices) => indices.len(),
        }
    }

    /// Returns whether no indices are stored.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns whether indices require the 32-bit GPU index format.
    pub fn uses_u32(&self) -> bool {
        matches!(self, Self::U32(_))
    }
}

/// CPU-side tile mesh ready for upload to vertex and index buffers.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TileMesh {
    /// Signed tile-local vertex positions.
    pub vertices: Vec<TileMeshVertex>,
    /// Triangle indices matching MapLibre GL JS winding.
    pub indices: TileMeshIndices,
}

/// Failure while deriving or allocating a tile mesh.
#[derive(Debug, Error)]
pub enum TileMeshError {
    /// Mesh dimensions overflow while evaluating the requested granularity.
    #[error("tile mesh dimensions overflow for granularity {granularity}")]
    DimensionOverflow {
        /// Requested subdivisions per tile axis.
        granularity: u32,
    },
    /// The requested mesh cannot address its vertices with 16-bit indices.
    #[error("tile mesh has {vertex_count} vertices and requires 32-bit indices")]
    RequiresU32Indices {
        /// Computed number of vertices.
        vertex_count: u64,
    },
    /// The requested mesh cannot address its vertices with supported GPU index formats.
    #[error("tile mesh has {vertex_count} vertices and exceeds 32-bit indices")]
    TooManyVertices {
        /// Computed number of vertices.
        vertex_count: u64,
    },
    /// A checked mesh count cannot be represented on the current platform.
    #[error("tile mesh count cannot be represented on this platform")]
    CountConversion {
        /// Integer conversion failure.
        #[source]
        source: TryFromIntError,
    },
}

#[derive(Clone, Copy)]
struct MeshDimensions {
    granularity: i64,
    quads_x: u64,
    quads_y: u64,
    vertices_x: u64,
    vertices_y: u64,
    offset_x: i64,
    offset_y: i64,
    end_x: i64,
    end_y: i64,
}

/// Creates the regular tile mesh used to approximate a curved globe surface.
pub fn create_tile_mesh(
    options: TileMeshOptions,
    index_type: TileIndexType,
) -> Result<TileMesh, TileMeshError> {
    let dimensions = mesh_dimensions(options)?;
    let vertex_count = dimensions
        .vertices_x
        .checked_mul(dimensions.vertices_y)
        .ok_or(TileMeshError::DimensionOverflow {
            granularity: options.granularity,
        })?;
    if index_type == TileIndexType::U16 && vertex_count > MAX_U16_VERTEX_COUNT {
        return Err(TileMeshError::RequiresU32Indices { vertex_count });
    }
    if vertex_count > MAX_U32_VERTEX_COUNT {
        return Err(TileMeshError::TooManyVertices { vertex_count });
    }

    let use_u32 = index_type == TileIndexType::U32 || vertex_count > MAX_U16_VERTEX_COUNT;
    let vertices = create_vertices(options, dimensions, vertex_count)?;
    let indices = create_indices(dimensions, use_u32)?;
    Ok(TileMesh { vertices, indices })
}

fn mesh_dimensions(options: TileMeshOptions) -> Result<MeshDimensions, TileMeshError> {
    let granularity = i64::from(options.granularity.max(1));
    let border_quads = u64::from(options.generate_borders) * 2;
    let north_quads = u64::from(options.extend_to_north_pole || options.generate_borders);
    let south_quads = u64::from(options.extend_to_south_pole || options.generate_borders);
    let quads_x = u64::from(options.granularity.max(1))
        .checked_add(border_quads)
        .ok_or(TileMeshError::DimensionOverflow {
            granularity: options.granularity,
        })?;
    let quads_y = u64::from(options.granularity.max(1))
        .checked_add(north_quads)
        .and_then(|count| count.checked_add(south_quads))
        .ok_or(TileMeshError::DimensionOverflow {
            granularity: options.granularity,
        })?;

    Ok(MeshDimensions {
        granularity,
        quads_x,
        quads_y,
        vertices_x: quads_x + 1,
        vertices_y: quads_y + 1,
        offset_x: if options.generate_borders { -1 } else { 0 },
        offset_y: if options.generate_borders || options.extend_to_north_pole {
            -1
        } else {
            0
        },
        end_x: granularity + i64::from(options.generate_borders),
        end_y: granularity + i64::from(options.generate_borders || options.extend_to_south_pole),
    })
}

fn create_vertices(
    options: TileMeshOptions,
    dimensions: MeshDimensions,
    vertex_count: u64,
) -> Result<Vec<TileMeshVertex>, TileMeshError> {
    let capacity = usize::try_from(vertex_count)
        .map_err(|source| TileMeshError::CountConversion { source })?;
    let mut vertices = Vec::with_capacity(capacity);
    for y in dimensions.offset_y..=dimensions.end_y {
        for x in dimensions.offset_x..=dimensions.end_x {
            vertices.push(TileMeshVertex {
                x: vertex_x(x, dimensions.granularity),
                y: vertex_y(y, dimensions.granularity, options),
            });
        }
    }
    Ok(vertices)
}

fn vertex_x(axis: i64, granularity: i64) -> i16 {
    if axis == -1 {
        -EXTENT_STENCIL_BORDER
    } else if axis == granularity + 1 {
        EXTENT_UINT as i16 + EXTENT_STENCIL_BORDER
    } else {
        (axis * i64::from(EXTENT_UINT) / granularity) as i16
    }
}

fn vertex_y(axis: i64, granularity: i64, options: TileMeshOptions) -> i16 {
    if axis == -1 {
        if options.extend_to_north_pole {
            NORTH_POLE_Y
        } else {
            -EXTENT_STENCIL_BORDER
        }
    } else if axis == granularity + 1 {
        if options.extend_to_south_pole {
            SOUTH_POLE_Y
        } else {
            EXTENT_UINT as i16 + EXTENT_STENCIL_BORDER
        }
    } else {
        (axis * i64::from(EXTENT_UINT) / granularity) as i16
    }
}

fn create_indices(
    dimensions: MeshDimensions,
    use_u32: bool,
) -> Result<TileMeshIndices, TileMeshError> {
    let index_count = dimensions
        .quads_x
        .checked_mul(dimensions.quads_y)
        .and_then(|count| count.checked_mul(6))
        .ok_or(TileMeshError::DimensionOverflow {
            granularity: dimensions.granularity as u32,
        })?;
    let capacity =
        usize::try_from(index_count).map_err(|source| TileMeshError::CountConversion { source })?;
    if use_u32 {
        Ok(TileMeshIndices::U32(fill_indices(
            dimensions,
            capacity,
            |index| index as u32,
        )))
    } else {
        Ok(TileMeshIndices::U16(fill_indices(
            dimensions,
            capacity,
            |index| index as u16,
        )))
    }
}

fn fill_indices<I, F>(dimensions: MeshDimensions, capacity: usize, convert: F) -> Vec<I>
where
    F: Fn(u64) -> I,
{
    let mut indices = Vec::with_capacity(capacity);
    for y in 0..dimensions.quads_y {
        for x in 0..dimensions.quads_x {
            let v0 = x + y * dimensions.vertices_x;
            let v1 = v0 + 1;
            let v2 = v0 + dimensions.vertices_x;
            let v3 = v2 + 1;
            for index in [v0, v2, v1, v1, v2, v3] {
                indices.push(convert(index));
            }
        }
    }
    indices
}

#[cfg(test)]
mod tests;
