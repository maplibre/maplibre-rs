use wgpu::{CommandEncoder, RenderPass};
use wgpu_profiler::ProfilerCommandRecorder;

use super::DeclaredDependency;

/// Handle to a declared renderpass output.
pub struct RenderPassHandle;

pub(super) enum RenderGraphEncoderOrPassInner<'a, 'pass> {
    Encoder(&'a mut CommandEncoder),
    RenderPass(&'a mut RenderPass<'pass>),
}

impl<'a, 'pass> ProfilerCommandRecorder for RenderGraphEncoderOrPassInner<'a, 'pass> {
    fn write_timestamp(&mut self, query_set: &wgpu::QuerySet, query_index: u32) {
        match self {
            RenderGraphEncoderOrPassInner::Encoder(e) => e.write_timestamp(query_set, query_index),
            RenderGraphEncoderOrPassInner::RenderPass(rp) => {
                rp.write_timestamp(query_set, query_index)
            }
        }
    }

    fn push_debug_group(&mut self, label: &str) {
        match self {
            RenderGraphEncoderOrPassInner::Encoder(e) => e.push_debug_group(label),
            RenderGraphEncoderOrPassInner::RenderPass(rp) => rp.push_debug_group(label),
        }
    }

    fn pop_debug_group(&mut self) {
        match self {
            RenderGraphEncoderOrPassInner::Encoder(e) => e.pop_debug_group(),
            RenderGraphEncoderOrPassInner::RenderPass(rp) => rp.pop_debug_group(),
        }
    }
}

/// Holds either a renderpass or an encoder.
pub struct RenderGraphEncoderOrPass<'a, 'pass>(pub(super) RenderGraphEncoderOrPassInner<'a, 'pass>);

impl<'a, 'pass> RenderGraphEncoderOrPass<'a, 'pass> {
    /// Get an encoder.
    ///
    /// # Panics
    ///
    /// If this node requested a renderpass, this will panic.
    pub fn get_encoder(self) -> &'a mut CommandEncoder {
        match self.0 {
            RenderGraphEncoderOrPassInner::Encoder(e) => e,
            RenderGraphEncoderOrPassInner::RenderPass(_) => {
                panic!("called get_encoder when the rendergraph node asked for a renderpass");
            }
        }
    }

    /// Get an renderpass from the given handle.
    pub fn get_rpass(
        self,
        _handle: DeclaredDependency<RenderPassHandle>,
    ) -> &'a mut RenderPass<'pass> {
        match self.0 {
            RenderGraphEncoderOrPassInner::Encoder(_) => {
                panic!("Internal rendergraph error: trying to get renderpass when one was not asked for")
            }
            RenderGraphEncoderOrPassInner::RenderPass(rpass) => rpass,
        }
    }
}
