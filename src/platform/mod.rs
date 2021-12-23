#[cfg(target_arch = "wasm32")]
pub mod web;

#[cfg(target_arch = "aarch64")]
pub mod apple;

#[cfg(target_os = "android")]
pub mod android;


#[cfg(target_arch = "wasm32")]
pub use instant::Instant;

#[cfg(not(target_arch = "wasm32"))]
pub use std::time::Instant;
