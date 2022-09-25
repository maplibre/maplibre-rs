//! # Maplibre-rs
//!
//! A multi-platform library for rendering vector tile maps with WebGPU.
//!
//! Maplibre-rs is a map renderer that can run natively on MacOS, Linux, Windows, Android, iOS and the web.
//! It takes advantage of Lyon to tessellate vector tiles and WebGPU to display them efficiently.
//! Maplibre-rs also has an headless mode (*work in progress*) that can generate rasters.
//!
//! The official guide book can be found [here](https://maplibre.org/maplibre-rs/docs/book/).
//!
//! ### Example
//!
//! To import maplibre-rs in your `Cargo.toml`:
//!
//! ```toml
//! maplibre = "0.0.2"
//! ```

pub mod context;
pub mod coords;
pub mod error;
#[cfg(feature = "headless")]
pub mod headless;
pub mod io;
// Exposed because of input handlers in maplibre-winit
pub mod map_schedule;
pub mod platform;
// TODO: Exposed because of camera
pub mod render;
pub mod style;
pub mod util;

pub mod window;
// Exposed because of doc-strings
pub mod schedule;
// Exposed because of SharedThreadState
pub mod stages;

pub mod environment;

// Used for benchmarking
pub mod benchmarking;

// Internal modules
pub(crate) mod tessellation;

// Export tile format
pub use geozero::mvt::tile;
