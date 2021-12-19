#[cfg(target_arch = "wasm32")]
pub use instant::Instant;

#[cfg(not(target_arch = "wasm32"))]
pub use std::time::Instant;
