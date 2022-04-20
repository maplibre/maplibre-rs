//! This module handles platform specific code. Depending on the compilation target different
//! parts of this module are used

#[cfg(target_arch = "wasm32")]
mod web;

#[cfg(any(target_os = "macos", target_os = "ios"))]
mod apple;

#[cfg(target_os = "android")]
mod android;

#[cfg(not(target_arch = "wasm32"))]
mod noweb;

/// For Vulkan/OpenGL
#[cfg(not(any(
    target_os = "android",
    target_os = "macos",
    target_os = "ios",
    target_arch = "wasm32"
)))]
pub const COLOR_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;

#[cfg(target_arch = "wasm32")]
pub use web::*;

#[cfg(any(target_os = "macos", target_os = "ios"))]
pub use apple::*;

#[cfg(target_os = "android")]
pub use android::*;

#[cfg(not(target_arch = "wasm32"))]
pub use noweb::*;

// FIXME: This limit is enforced by WebGL. Actually this makes sense!
// FIXME: This can also be achieved by _pad attributes in shader_ffi.rs
pub const MIN_BUFFER_SIZE: u64 = 32;
