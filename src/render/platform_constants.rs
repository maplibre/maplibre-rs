// WebGPU
#[cfg(all(target_arch = "wasm32", not(feature = "web-webgl")))]
pub const COLOR_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8Unorm;
// WebGL
#[cfg(all(target_arch = "wasm32", feature = "web-webgl"))]
pub const COLOR_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;
// Vulkan/Metal/OpenGL
#[cfg(target_os = "linux")]
pub const COLOR_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;

#[cfg(target_os = "android")]
pub const COLOR_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

// FIXME: This limit is enforced by WebGL. Actually this makes sense!
// FIXME: This can also be achieved by _pad attributes in shader_ffi.rs
pub const MIN_BUFFER_SIZE: u64 = 32;
