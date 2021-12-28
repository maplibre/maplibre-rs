#[cfg(target_arch = "wasm32")]
mod web;

#[cfg(target_arch = "aarch64")]
mod apple;

#[cfg(target_os = "android")]
mod android;


#[cfg(not(any(
    target_os = "android",
    target_arch = "aarch64",
    target_arch = "wasm32"
)))]
mod generic;

#[cfg(target_arch = "wasm32")]
pub use web::*;

#[cfg(target_arch = "aarch64")]
pub use apple::*;

#[cfg(target_os = "android")]
pub use android::*;

#[cfg(not(any(
    target_os = "android",
    target_arch = "aarch64",
    target_arch = "wasm32"
)))]
pub use generic::*;


// FIXME: This limit is enforced by WebGL. Actually this makes sense!
// FIXME: This can also be achieved by _pad attributes in shader_ffi.rs
pub const MIN_BUFFER_SIZE: u64 = 32;
