//! This module implements the rendering algorithm of mapr. It manages the whole communication with
//! the GPU.

mod buffer_pool;
mod options;
mod piplines;
mod shaders;
mod texture;
mod tile_mask_pattern;

pub mod camera;
pub mod render_state;

// These are created during tessellation and must be public
pub use shaders::ShaderVertex;
