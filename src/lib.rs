mod input;

pub(crate) mod coords;
pub(crate) mod error;
pub(crate) mod io;
pub(crate) mod main_loop;
pub(crate) mod render;
pub(crate) mod util;

// Used from outside to initialize mapr
pub mod platform;

// Used for benchmarking
pub mod tessellation;
