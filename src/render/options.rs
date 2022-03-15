use wgpu::BufferAddress;

pub const DEBUG_WIREFRAME: bool = false;
pub const DEBUG_STENCIL_PATTERN: bool = false;
pub const INDEX_FORMAT: wgpu::IndexFormat = wgpu::IndexFormat::Uint32; // Must match IndexDataType

pub const VERTEX_BUFFER_SIZE: BufferAddress = 1024 * 1024 * 32;
pub const FEATURE_METADATA_BUFFER_SIZE: BufferAddress = 1024 * 1024 * 32;
pub const INDICES_BUFFER_SIZE: BufferAddress = 1024 * 1024 * 16;

pub const TILE_META_COUNT: BufferAddress = 1024 * 24;
