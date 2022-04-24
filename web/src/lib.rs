pub use maplibre::*;

#[cfg(not(target_arch = "wasm32"))]
compile_error!("web works only on wasm32.");
