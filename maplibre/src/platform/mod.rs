//! Handles platform specific code. Depending on the compilation target, different
//! parts of this module are used.

#[cfg(not(target_arch = "wasm32"))]
pub use noweb::run_multithreaded;
#[cfg(not(target_arch = "wasm32"))]
pub use noweb::ReqwestOffscreenKernelEnvironment;

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

/// Minimum WebGPU buffer size
///
/// FIXME: This limit is enforced by WebGL. Actually this makes sense!
/// FIXME: This can also be achieved by _pad attributes in shader_ffi.rs
pub const MIN_WEBGL_BUFFER_SIZE: u64 = 32;
