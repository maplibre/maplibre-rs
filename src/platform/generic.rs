//! Module which is used if android, apple and web is not used.

pub use std::time::Instant;

// Vulkan/OpenGL
pub const COLOR_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;
