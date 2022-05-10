//! Rendergraph implementation that rend3 uses for all render work scheduling.
//!
//! Start with [`RenderGraph::new`] and add nodes and then
//! [`RenderGraph::execute`] to run everything.
//!
//! # High Level Overview
//!
//! The design consists of a series of nodes which have inputs and outputs.
//! These inputs can be render targets,  or custom user data. The
//! graph is laid out in order using the inputs/outputs then pruned.
//!
//! Each node is a pile of arbitrary code that can use various resources within
//! the renderer to do work.
//!
//! Two submits happen during execute. First, all work that doesn't interact
//! with the surface is submitted, then the surface is acquired, then all the
//! following work is submitted.
//!
//! # Nodes
//!
//! Nodes are made with [`RenderGraphNodeBuilder`]. The builder is used to
//! declare all the dependencies of the node ("outside" the node), then
//! [`RenderGraphNodeBuilder::build`] is called. This takes a callback that
//! contains all the code that will run as part of the node (the "inside").
//!
//! The arguments given to this callback give you all the data you need to do
//! your work, including turning handles-to-dependencies into actual concrete
//! resources. See the documentation for [`RenderGraphNodeBuilder::build`] for a
//! description of the arguments you are provided.
//!
//! # Renderpasses/Encoders
//!
//! The graph will automatically deduplicate renderpasses, such that if there
//! are two nodes in a row that have a compatible renderpass, they will use the
//! same renderpass. An encoder will not be available if a renderpass is in use.
//! This is intentional as there should be as few renderpasses as possible, so
//! you should separate the code that needs a raw encoder from the code that is
//! using a renderpass.
//!
//! Because renderpasses carry with them a lifetime that can cause problems, two
//! facilities are available.
//!
//! First is the [`PassthroughDataContainer`] which
//! allows you to take lifetimes of length `'node` and turn them into lifetimes
//! of length `'rpass`. This is commonly used to bring in any state from the
//! outside.
//!
//! Second is the [`RpassTemporaryPool`]. If, inside the node, you need to
//! create a temporary, you can put that temporary on the pool, and it will
//! automatically have lifetime `'rpass`. The temporary is destroyed right after
//! the renderpass is.

use glam::UVec2;
use types::SampleCount;
use wgpu::{Color, TextureFormat, TextureUsages, TextureView};

mod encpass;
#[allow(clippy::module_inception)] // lmao
mod graph;
mod node;
mod output;
mod passthrough;
mod store;
mod temp;
mod texture_store;
mod types;

pub(crate) use encpass::*;
pub(crate) use graph::*;
pub(crate) use node::*;
pub(crate) use passthrough::*;
pub(crate) use store::*;
pub(crate) use temp::*;
pub(crate) use texture_store::*;

/// Description of a single render target.
#[derive(Debug, Clone)]
pub struct RenderTargetDescriptor {
    pub label: Option<String>,
    pub resolution: UVec2,
    pub samples: SampleCount,
    pub format: TextureFormat,
    pub usage: TextureUsages,
}
impl RenderTargetDescriptor {
    fn to_core(&self) -> RenderTargetCore {
        RenderTargetCore {
            resolution: self.resolution,
            samples: self.samples,
            format: self.format,
            usage: self.usage,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub(crate) struct RenderTargetCore {
    pub resolution: UVec2,
    pub samples: SampleCount,
    pub format: TextureFormat,
    pub usage: TextureUsages,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum GraphResource {
    OutputTexture,
    External,
    Texture(usize),
    Data(usize),
}

/// Handle to a graph-stored render target.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct RenderTargetHandle {
    // Must only be OutputTexture or Texture
    resource: GraphResource,
}

/// Targets that make up a renderpass.
#[derive(Debug, PartialEq)]
pub struct RenderPassTargets {
    /// Color targets
    pub targets: Vec<RenderPassTarget>,
    /// Depth-stencil target
    pub depth_stencil: Option<RenderPassDepthTarget>,
}

impl RenderPassTargets {
    /// Determines if two renderpasses have compatible targets.
    ///
    /// `this: Some, other: Some` will check the contents  
    /// `this: None, other: None` is always true  
    /// one some and one none is always false.
    pub fn compatible(this: Option<&Self>, other: Option<&Self>) -> bool {
        match (this, other) {
            (Some(this), Some(other)) => {
                let targets_compatible = this.targets.len() == other.targets.len()
                    && this
                        .targets
                        .iter()
                        .zip(other.targets.iter())
                        .all(|(me, you)| me.color == you.color && me.resolve == you.resolve);

                let depth_compatible = match (&this.depth_stencil, &other.depth_stencil) {
                    (Some(this_depth), Some(other_depth)) => this_depth == other_depth,
                    (None, None) => true,
                    _ => false,
                };

                targets_compatible && depth_compatible
            }
            (None, None) => true,
            _ => false,
        }
    }
}

/// Color target in a renderpass.
#[derive(Debug, PartialEq)]
pub struct RenderPassTarget {
    /// Color attachment. Must be declared as a dependency of the node before it
    /// can be used.
    pub color: DeclaredDependency<RenderTargetHandle>,
    /// Color the attachment will be cleared with if this is the first use.
    pub clear: Color,
    /// Resolve attachment. Can only be present if color attachment has > 1
    /// sample.
    pub resolve: Option<DeclaredDependency<RenderTargetHandle>>,
}

/// Depth target in a renderpass.
#[derive(Debug, PartialEq)]
pub struct RenderPassDepthTarget {
    /// The target to use as depth.
    pub target: DepthHandle,
    /// Depth value the attachment will be cleared with if this is the first
    /// use.
    pub depth_clear: Option<f32>,
    /// Stencil value the attachment will be cleared with if this is the first
    /// use.
    pub stencil_clear: Option<u32>,
}

/// Handle to something that can be used as depth.
#[derive(Debug, PartialEq)]
pub enum DepthHandle {
    RenderTarget(DeclaredDependency<RenderTargetHandle>),
}
