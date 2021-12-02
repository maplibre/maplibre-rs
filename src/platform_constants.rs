
#[cfg(all(target_arch = "wasm32", not(feature = "web-webgl")))]
pub const COLOR_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8Unorm;
#[cfg(all(target_arch = "wasm32", feature = "web-webgl"))]
pub const COLOR_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;
#[cfg(not(target_arch = "wasm32"))]
pub const COLOR_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;
