//! Utilities which holds references to GPU-owned. Usually a resource is a wrapper which makes using
//! buffers or textures simpler.

mod buffer_pool;
mod globals;
mod pipeline;
mod shader;
mod surface;
mod texture;
mod tracked_render_pass;

pub use buffer_pool::*;
pub use globals::*;
pub use pipeline::*;
pub use shader::*;
pub use surface::*;
pub use texture::*;
pub use tracked_render_pass::*;
