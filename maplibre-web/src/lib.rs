pub use maplibre_core::*;

#[cfg(not(target_arch = "wasm32"))]
compile_error!("maplibre-web works only on wasm32.");
