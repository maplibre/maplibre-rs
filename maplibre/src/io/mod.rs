//! Handles IO related processing as well as multithreading.

pub use geozero::mvt::tile::Layer as RawLayer;

pub mod apc;
pub mod geometry_index;
pub mod scheduler;
pub mod source_client;
pub mod source_type;
#[cfg(feature = "embed-static-tiles")]
pub mod static_tile_fetcher;
