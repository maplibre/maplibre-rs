//! GPU cache for projection-aware raster and stencil tile meshes.

use std::{collections::HashMap, num::TryFromIntError};

use thiserror::Error;
use wgpu::util::DeviceExt;

use crate::{
    coords::{WorldTileCoords, ZOOM_BOUNDS},
    projection::globe::tile_mesh::{
        create_tile_mesh, TileIndexType, TileMeshError, TileMeshIndices, TileMeshOptions,
    },
};

/// Rendering purpose controlling globe mesh subdivision.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TileMeshUsage {
    /// Textured tile geometry.
    Raster,
    /// Tile clipping-mask geometry.
    Stencil,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct TileMeshKey {
    granularity: u32,
    generate_borders: bool,
    extend_to_north_pole: bool,
    extend_to_south_pole: bool,
}

/// GPU buffers for one tile-mesh variant.
pub struct GpuTileMesh {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_format: wgpu::IndexFormat,
    index_count: u32,
}

impl GpuTileMesh {
    /// Returns the vertex buffer containing signed 16-bit tile positions.
    pub fn vertex_buffer(&self) -> &wgpu::Buffer {
        &self.vertex_buffer
    }

    /// Returns the triangle index buffer.
    pub fn index_buffer(&self) -> &wgpu::Buffer {
        &self.index_buffer
    }

    /// Returns the index width used by the buffer.
    pub fn index_format(&self) -> wgpu::IndexFormat {
        self.index_format
    }

    /// Returns the number of indices to draw.
    pub fn index_count(&self) -> u32 {
        self.index_count
    }
}

/// Failure while preparing a GPU tile mesh.
#[derive(Debug, Error)]
pub enum GpuTileMeshError {
    /// CPU mesh generation failed.
    #[error("failed to generate tile mesh")]
    Generate {
        /// Underlying mesh error.
        #[source]
        source: TileMeshError,
    },
    /// Index count cannot be represented by a WebGPU draw range.
    #[error("tile mesh index count exceeds u32")]
    IndexCount {
        /// Failed integer conversion.
        #[source]
        source: TryFromIntError,
    },
}

/// Lazily populated GPU cache keyed by subdivision, borders, and pole extensions.
#[derive(Default)]
pub struct GlobeTileMeshCache {
    meshes: HashMap<TileMeshKey, GpuTileMesh>,
}

impl GlobeTileMeshCache {
    /// Creates the mesh variant needed by a draw if it is not cached.
    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        coords: WorldTileCoords,
        usage: TileMeshUsage,
        generate_borders: bool,
    ) -> Result<(), GpuTileMeshError> {
        let key = mesh_key(coords, usage, generate_borders);
        if self.meshes.contains_key(&key) {
            return Ok(());
        }
        let mesh = create_tile_mesh(
            TileMeshOptions {
                granularity: key.granularity,
                generate_borders: key.generate_borders,
                extend_to_north_pole: key.extend_to_north_pole,
                extend_to_south_pole: key.extend_to_south_pole,
            },
            TileIndexType::U16,
        )
        .map_err(|source| GpuTileMeshError::Generate { source })?;
        let index_count = u32::try_from(mesh.indices.len())
            .map_err(|source| GpuTileMeshError::IndexCount { source })?;
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("globe tile mesh vertices"),
            contents: bytemuck::cast_slice(&mesh.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let (index_buffer, index_format) = upload_indices(device, mesh.indices);
        self.meshes.insert(
            key,
            GpuTileMesh {
                vertex_buffer,
                index_buffer,
                index_format,
                index_count,
            },
        );
        Ok(())
    }

    /// Returns a previously prepared mesh variant.
    pub fn get(
        &self,
        coords: WorldTileCoords,
        usage: TileMeshUsage,
        generate_borders: bool,
    ) -> Option<&GpuTileMesh> {
        self.meshes.get(&mesh_key(coords, usage, generate_borders))
    }
}

fn upload_indices(
    device: &wgpu::Device,
    indices: TileMeshIndices,
) -> (wgpu::Buffer, wgpu::IndexFormat) {
    let (bytes, format) = match &indices {
        TileMeshIndices::U16(values) => (bytemuck::cast_slice(values), wgpu::IndexFormat::Uint16),
        TileMeshIndices::U32(values) => (bytemuck::cast_slice(values), wgpu::IndexFormat::Uint32),
    };
    let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("globe tile mesh indices"),
        contents: bytes,
        usage: wgpu::BufferUsages::INDEX,
    });
    (buffer, format)
}

fn mesh_key(coords: WorldTileCoords, usage: TileMeshUsage, generate_borders: bool) -> TileMeshKey {
    let zoom = u32::from(u8::from(coords.z));
    let minimum = match usage {
        TileMeshUsage::Raster => 32,
        TileMeshUsage::Stencil => 1,
    };
    let granularity = 128_u32.checked_shr(zoom).unwrap_or(0).max(minimum);
    let last_row = i64::from(ZOOM_BOUNDS[usize::from(u8::from(coords.z))]) - 1;
    TileMeshKey {
        granularity,
        generate_borders,
        extend_to_north_pole: coords.y == 0,
        extend_to_south_pole: i64::from(coords.y) == last_row,
    }
}

#[cfg(test)]
mod tests;
