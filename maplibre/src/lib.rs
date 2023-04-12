//! # Maplibre-rs
//!
//! A multi-platform library for rendering vector tile maps with WebGPU.
//!
//! Maplibre-rs is a map renderer that can run natively on MacOS, Linux, Windows, Android, iOS and the web.
//! It takes advantage of Lyon to tessellate vector tiles and WebGPU to display them efficiently.
//! Maplibre-rs also has an headless mode (*work in progress*) that can generate raster images.
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

#![deny(unused_imports)]

extern crate core;

// Export tile format
pub use geozero::mvt::tile; // Used in transferables.rs in web/singlethreaded

// Internal modules
pub(crate) mod tessellation;

pub mod context;
pub mod coords;
#[cfg(feature = "headless")]
pub mod headless;
pub mod io;
pub mod platform;
// TODO: Exposed because of camera
pub mod render;
pub mod style;
pub mod util;

pub mod window;
// Exposed because of doc-strings
pub mod schedule;

pub mod environment;

// Used for benchmarking
pub mod benchmarking;

pub mod event_loop;
pub mod kernel;
pub mod map;
pub mod plugin;
pub mod tcs;
pub mod view_state;

// Plugins
pub mod debug;
pub mod raster;
pub mod vector;
