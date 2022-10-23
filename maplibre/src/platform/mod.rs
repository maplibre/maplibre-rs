//! Handles platform specific code. Depending on the compilation target, different
//! parts of this module are used.

// WebGPU
#[cfg(all(target_arch = "wasm32", not(feature = "web-webgl")))]
pub const COLOR_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8Unorm;

// WebGL
#[cfg(all(target_arch = "wasm32", feature = "web-webgl"))]
pub const COLOR_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

// Vulkan Android
#[cfg(target_os = "android")]
pub const COLOR_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

/// MacOS and iOS (Metal).
#[cfg(any(target_os = "macos", target_os = "ios"))]
pub const COLOR_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;

/// For Vulkan/OpenGL
#[cfg(not(any(
    target_os = "android",
    target_os = "macos",
    any(target_os = "macos", target_os = "ios"),
    target_arch = "wasm32"
)))]
pub const COLOR_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;

#[cfg(not(target_arch = "wasm32"))]
mod noweb;

/// Http client for non-web targets.
pub mod http_client {
    #[cfg(not(target_arch = "wasm32"))]
    pub use super::noweb::http_client::*;
}

/// Scheduler for non-web targets.
pub mod scheduler {
    #[cfg(not(target_arch = "wasm32"))]
    pub use super::noweb::scheduler::*;
}

pub mod trace {
    #[cfg(not(target_arch = "wasm32"))]
    pub use super::noweb::trace::*;
}

#[cfg(not(target_arch = "wasm32"))]
pub use noweb::run_multithreaded;

/// Minimum WebGPU buffer size
///
/// FIXME: This limit is enforced by WebGL. Actually this makes sense!
/// FIXME: This can also be achieved by _pad attributes in shader_ffi.rs
pub const MIN_BUFFER_SIZE: u64 = 32;
