#[cfg(target_arch = "wasm32")]
mod web;

#[cfg(all(target_arch = "aarch64", not(target_os = "android")))]
mod apple;

#[cfg(target_os = "android")]
mod android;

#[cfg(not(target_arch = "wasm32"))]
mod noweb;

#[cfg(not(any(
    target_os = "android",
    all(target_arch = "aarch64", not(target_os = "android")),
    target_arch = "wasm32"
)))]
mod generic;

#[cfg(target_arch = "wasm32")]
pub use web::*;

#[cfg(all(target_arch = "aarch64", not(target_os = "android")))]
pub use apple::*;

#[cfg(target_os = "android")]
pub use android::*;

#[cfg(not(target_arch = "wasm32"))]
pub use noweb::*;

#[cfg(not(any(
    target_os = "android",
    all(target_arch = "aarch64", not(target_os = "android")),
    target_arch = "wasm32"
)))]
pub use generic::*;

// FIXME: This limit is enforced by WebGL. Actually this makes sense!
// FIXME: This can also be achieved by _pad attributes in shader_ffi.rs
pub const MIN_BUFFER_SIZE: u64 = 32;

use std::io;
