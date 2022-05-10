use crate::render::Renderer;
use std::{cell::RefCell, sync::Arc};

use super::{
    DataHandle, GraphResource, PassthroughDataContainer, PassthroughDataRef, PassthroughDataRefMut,
    ReadyData, RenderGraph, RenderGraphDataStore, RenderGraphEncoderOrPass, RenderPassHandle,
    RenderPassTargets, RenderTargetHandle, RpassTemporaryPool,
};

/// Wraps a handle proving you have declared it as a dependency.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct DeclaredDependency<Handle> {
    pub(super) handle: Handle,
}

#[allow(clippy::type_complexity)]
pub(super) struct RenderGraphNode<'node> {
    pub inputs: Vec<GraphResource>,
    pub outputs: Vec<GraphResource>,
    pub label: String,
    pub rpass: Option<RenderPassTargets>,
    pub passthrough: PassthroughDataContainer<'node>,
    pub exec: Box<
        dyn for<'b, 'pass> FnOnce(
                &mut PassthroughDataContainer<'pass>,
                &Arc<dyn Renderer>,
                RenderGraphEncoderOrPass<'b, 'pass>,
                &'pass RpassTemporaryPool<'pass>,
                &'pass ReadyData,
                RenderGraphDataStore<'pass>,
            ) + 'node,
    >,
}

/// Builder for a graph node.
///
/// Calling build will automatically add the node to the rendergraph.
pub struct RenderGraphNodeBuilder<'a, 'node> {
    pub(super) graph: &'a mut RenderGraph<'node>,
    pub(super) label: String,
    pub(super) inputs: Vec<GraphResource>,
    pub(super) outputs: Vec<GraphResource>,
    pub(super) passthrough: PassthroughDataContainer<'node>,
    pub(super) rpass: Option<RenderPassTargets>,
}
impl<'a, 'node> RenderGraphNodeBuilder<'a, 'node> {
    /// Declares a rendertarget to be read from but not writen to.
    pub fn add_render_target_input(
        &mut self,
        handle: RenderTargetHandle,
    ) -> DeclaredDependency<RenderTargetHandle> {
        self.inputs.push(handle.resource);
        DeclaredDependency { handle }
    }

    /// Declares a rendertarget to be read or written to.
    pub fn add_render_target_output(
        &mut self,
        handle: RenderTargetHandle,
    ) -> DeclaredDependency<RenderTargetHandle> {
        self.inputs.push(handle.resource);
        self.outputs.push(handle.resource);
        DeclaredDependency { handle }
    }

    /// Sugar over [add_render_target_output][arto] which makes it easy to
    /// declare resolve textures, which are often Option<RenderTargetHandle>
    ///
    /// [arto]: RenderGraphNodeBuilder::add_render_target_output
    pub fn add_optional_render_target_output(
        &mut self,
        handle: Option<RenderTargetHandle>,
    ) -> Option<DeclaredDependency<RenderTargetHandle>> {
        Some(self.add_render_target_output(handle?))
    }

    /// Declares a renderpass that will be written to. Declaring a renderpass
    /// will prevent access to an encoder in the node.
    pub fn add_renderpass(
        &mut self,
        targets: RenderPassTargets,
    ) -> DeclaredDependency<RenderPassHandle> {
        assert!(
            self.rpass.is_none(),
            "Cannot have more than one graph-associated renderpass per node."
        );
        self.rpass = Some(targets);
        DeclaredDependency {
            handle: RenderPassHandle,
        }
    }

    /// Declares use of a data handle for reading.
    pub fn add_data_input<T>(&mut self, handle: DataHandle<T>) -> DeclaredDependency<DataHandle<T>>
    where
        T: 'static,
    {
        self.add_data(handle, false)
    }

    /// Declares use of a data handle for reading and writing.
    pub fn add_data_output<T>(&mut self, handle: DataHandle<T>) -> DeclaredDependency<DataHandle<T>>
    where
        T: 'static,
    {
        self.add_data(handle, true)
    }

    fn add_data<T>(
        &mut self,
        handle: DataHandle<T>,
        output: bool,
    ) -> DeclaredDependency<DataHandle<T>>
    where
        T: 'static,
    {
        // TODO: error handling
        // TODO: move this validation to all types
        self.graph
            .data
            .get(handle.idx)
            .expect("internal rendergraph error: cannot find data handle")
            .downcast_ref::<RefCell<Option<T>>>()
            .expect("used custom data that was previously declared with a different type");

        self.inputs.push(GraphResource::Data(handle.idx));
        if output {
            self.outputs.push(GraphResource::Data(handle.idx));
        }
        DeclaredDependency { handle }
    }

    /// Declares that this node has an "external" output, meaning it can never
    /// be culled.
    pub fn add_external_output(&mut self) {
        self.inputs.push(GraphResource::External);
        self.outputs.push(GraphResource::External);
    }

    /// Passthrough a bit of immutable external data with lifetime 'node so you
    /// can receieve it inside with lifetime 'rpass.
    ///
    /// Use [PassthroughDataContainer::get][g] to get the value on the inside.
    ///
    /// [g]: super::PassthroughDataContainer::get
    pub fn passthrough_ref<T: 'node>(&mut self, data: &'node T) -> PassthroughDataRef<T> {
        self.passthrough.add_ref(data)
    }

    /// Passthrough a bit of mutable external data with lifetime 'node so you
    /// can receieve it inside with lifetime 'rpass.
    ///
    /// Use [PassthroughDataContainer::get_mut][g] to get the value on the
    /// inside.
    ///
    /// [g]: super::PassthroughDataContainer::get_mut
    pub fn passthrough_ref_mut<T: 'node>(
        &mut self,
        data: &'node mut T,
    ) -> PassthroughDataRefMut<T> {
        self.passthrough.add_ref_mut(data)
    }

    /// Builds the rendergraph node and adds it into the rendergraph.
    ///
    /// Takes a function that is the body of the node. Nodes will only run if a
    /// following node consumes the output. See module level docs for more
    /// details.
    ///
    /// The function takes the following arguments (which I will give names):
    ///  - `pt`: a container which you can get all the passthrough data out of
    ///  - `renderer`: a reference to the renderer.
    ///  - `encoder_or_pass`: either the asked-for renderpass, or a command
    ///    encoder.
    ///  - `temps`: storage for temporary data that lasts the length of the
    ///    renderpass.
    ///  - `ready`: result from calling ready on various managers.
    ///  - `graph_data`: read-only access to various managers and access to
    ///    in-graph data.
    pub fn build<F>(self, exec: F)
    where
        F: for<'b, 'pass> FnOnce(
                &mut PassthroughDataContainer<'pass>,
                &Arc<dyn Renderer>,
                RenderGraphEncoderOrPass<'b, 'pass>,
                &'pass RpassTemporaryPool<'pass>,
                &'pass ReadyData,
                RenderGraphDataStore<'pass>,
            ) + 'node,
    {
        self.graph.nodes.push(RenderGraphNode {
            label: self.label,
            inputs: self.inputs,
            outputs: self.outputs,
            rpass: self.rpass,
            passthrough: self.passthrough,
            exec: Box::new(exec),
        });
    }
}
