mod fps_meter;
mod input;
mod platform;

pub(crate) mod coords;
pub(crate) mod error;
pub(crate) mod render;
pub(crate) mod util;

// Used from outside to initialize mapr
pub mod io;
pub mod main_loop;

// Used for benchmarking
pub mod example;
pub mod tesselation;
