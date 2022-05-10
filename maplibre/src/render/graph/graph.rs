use std::collections::{HashMap, HashSet};
use std::{
    any::Any,
    cell::{RefCell, UnsafeCell},
    marker::PhantomData,
    mem,
    sync::Arc,
};

use wgpu::{
    CommandBuffer, CommandEncoder, CommandEncoderDescriptor, LoadOp, Operations, RenderPass,
    RenderPassColorAttachment, RenderPassDepthStencilAttachment, RenderPassDescriptor, TextureView,
    TextureViewDescriptor,
};

use super::{
    output::OutputFrame, types::RendererStatistics, DataHandle, DepthHandle, GraphResource,
    PassthroughDataContainer, RenderGraphDataStore, RenderGraphEncoderOrPass,
    RenderGraphEncoderOrPassInner, RenderGraphNode, RenderGraphNodeBuilder, RenderPassTargets,
    RenderTargetDescriptor, RenderTargetHandle, RpassTemporaryPool,
};

/// Output of calling ready on various managers.
#[derive(Clone)]
pub struct ReadyData {
    pub d2_texture: TextureManagerReadyOutput,
    pub d2c_texture: TextureManagerReadyOutput,
    pub directional_light_cameras: Vec<CameraManager>,
}

/// Implementation of a rendergraph. See module docs for details.
pub struct RenderGraph<'node> {
    pub(super) targets: Vec<RenderTargetDescriptor>,
    pub(super) shadows: HashSet<usize>,
    pub(super) data: Vec<Box<dyn Any>>, // Any is RefCell<Option<T>> where T is the stored data
    pub(super) nodes: Vec<RenderGraphNode<'node>>,
}
impl<'node> RenderGraph<'node> {
    pub fn new() -> Self {
        Self {
            targets: Vec::with_capacity(32),
            shadows: HashSet::with_capacity(32),
            data: Vec::with_capacity(32),
            nodes: Vec::with_capacity(64),
        }
    }

    pub fn add_node<'a, S>(&'a mut self, label: S) -> RenderGraphNodeBuilder<'a, 'node>
    where
        S: Into<String>,
    {
        RenderGraphNodeBuilder {
            label: label.into(),
            graph: self,
            inputs: Vec::with_capacity(16),
            outputs: Vec::with_capacity(16),
            passthrough: PassthroughDataContainer::new(),
            rpass: None,
        }
    }

    pub fn add_render_target(&mut self, desc: RenderTargetDescriptor) -> RenderTargetHandle {
        let idx = self.targets.len();
        self.targets.push(desc);
        RenderTargetHandle {
            resource: GraphResource::Texture(idx),
        }
    }

    pub fn add_surface_texture(&mut self) -> RenderTargetHandle {
        RenderTargetHandle {
            resource: GraphResource::OutputTexture,
        }
    }

    pub fn add_data<T: 'static>(&mut self) -> DataHandle<T> {
        let idx = self.data.len();
        self.data.push(Box::new(RefCell::new(None::<T>)));
        DataHandle {
            idx,
            _phantom: PhantomData,
        }
    }

    pub fn execute(
        self,
        renderer: &Arc<Renderer>,
        output: OutputFrame,
        mut cmd_bufs: Vec<CommandBuffer>,
        ready_output: &ReadyData,
    ) -> Option<RendererStatistics> {
        profiling::scope!("RenderGraph::execute");

        let mut awaiting_inputs = HashSet::new();
        // The surface is used externally
        awaiting_inputs.insert(GraphResource::OutputTexture);
        // External deps are used externally
        awaiting_inputs.insert(GraphResource::External);

        let mut pruned_node_list = Vec::with_capacity(self.nodes.len());
        {
            profiling::scope!("Dead Node Elimination");
            // Iterate the nodes backwards to track dependencies
            for node in self.nodes.into_iter().rev() {
                // If any of our outputs are used by a previous node, we have reason to exist
                let outputs_used = node.outputs.iter().any(|o| awaiting_inputs.remove(o));

                if outputs_used {
                    // Add our inputs to be matched up with outputs.
                    awaiting_inputs.extend(node.inputs.iter().cloned());
                    // Push our node on the new list
                    pruned_node_list.push(node)
                }
            }
            // We iterated backwards to prune nodes, so flip it back to normal.
            pruned_node_list.reverse();
        }

        let mut resource_spans = HashMap::<_, (usize, Option<usize>)>::new();
        {
            profiling::scope!("Resource Span Analysis");
            // Iterate through all the nodes, tracking the index where they are first used,
            // and the index where they are last used.
            for (idx, node) in pruned_node_list.iter().enumerate() {
                // Add or update the range for all inputs
                for &input in &node.inputs {
                    resource_spans
                        .entry(input)
                        .and_modify(|range| range.1 = Some(idx))
                        .or_insert((idx, Some(idx)));
                }
                // And the outputs
                for &output in &node.outputs {
                    resource_spans
                        .entry(output)
                        .and_modify(|range| range.1 = Some(idx))
                        .or_insert((idx, Some(idx)));
                }
            }
        }

        // If the surface is used, we need treat it as if it has no end, as it will be
        // "used" after the graph is done.
        if let Some((_, surface_end)) = resource_spans.get_mut(&GraphResource::OutputTexture) {
            *surface_end = None;
        }

        // For each node, record the list of textures whose spans start and the list of
        // textures whose spans end.
        let mut resource_changes = vec![(Vec::new(), Vec::new()); pruned_node_list.len()];
        {
            profiling::scope!("Compute Resource Span Deltas");
            for (&resource, span) in &resource_spans {
                resource_changes[span.0].0.push(resource);
                if let Some(end) = span.1 {
                    resource_changes[end].1.push(resource);
                }
            }
        }

        let mut data_core = renderer.data_core.lock();
        let data_core = &mut *data_core;

        // Iterate through every node, allocating and deallocating textures as we go.

        // Maps a texture description to any available textures. Will try to pull from
        // here instead of making a new texture.
        let graph_texture_store = &mut data_core.graph_texture_store;
        // Mark all textures as unused, so the ones that are unused can be culled after
        // this pass.
        graph_texture_store.mark_unused();

        // Stores the Texture while a texture is using it
        let mut active_textures = HashMap::new();
        // Maps a name to its actual texture view.
        let mut active_views = HashMap::new();
        // Which node index needs acquire to happen.
        let mut acquire_idx = None;
        {
            profiling::scope!("Render Target Allocation");
            for (idx, (starting, ending)) in resource_changes.into_iter().enumerate() {
                for start in starting {
                    match start {
                        GraphResource::Texture(idx) => {
                            let desc = &self.targets[idx];
                            let tex =
                                graph_texture_store.get_texture(&renderer.device, desc.to_core());
                            let view = tex.create_view(&TextureViewDescriptor {
                                label: desc.label.as_deref(),
                                ..TextureViewDescriptor::default()
                            });
                            active_textures.insert(idx, tex);
                            active_views.insert(idx, view);
                        }
                        GraphResource::Shadow(..) => {}
                        GraphResource::Data(..) => {}
                        GraphResource::OutputTexture => {
                            acquire_idx = Some(idx);
                            continue;
                        }
                        GraphResource::External => {}
                    };
                }

                for end in ending {
                    match end {
                        GraphResource::Texture(idx) => {
                            let tex = active_textures
                                .remove(&idx)
                                .expect("internal rendergraph error: texture end with no start");

                            let desc = self.targets[idx].clone();
                            graph_texture_store.return_texture(desc.to_core(), tex);
                        }
                        GraphResource::Shadow(..) => {}
                        GraphResource::Data(..) => {}
                        GraphResource::OutputTexture => {}
                        GraphResource::External => {}
                    };
                }
            }
        }

        // All textures that were ever returned are marked as used, so anything in here
        // that wasn't ever returned, was unused throughout the whole graph.
        graph_texture_store.remove_unused();

        // Iterate through all nodes and describe the node when they _end_
        let mut renderpass_ends = Vec::with_capacity(16);
        // If node is compatible with the previous node
        let mut compatible = Vec::with_capacity(pruned_node_list.len());
        {
            profiling::scope!("Renderpass Description");
            for (idx, node) in pruned_node_list.iter().enumerate() {
                // We always assume the first node is incompatible so the codepaths below are
                // consistent.
                let previous = match idx.checked_sub(1) {
                    Some(prev) => pruned_node_list[prev].rpass.as_ref(),
                    None => {
                        compatible.push(false);
                        continue;
                    }
                };

                compatible.push(RenderPassTargets::compatible(previous, node.rpass.as_ref()))
            }

            for (idx, &compatible) in compatible.iter().enumerate() {
                if compatible {
                    *renderpass_ends.last_mut().unwrap() = idx;
                } else {
                    renderpass_ends.push(idx)
                }
            }
        }

        profiling::scope!("Run Nodes");

        let shadow_views = data_core.directional_light_manager.get_layer_views();

        let output_cell = UnsafeCell::new(output);
        let encoder_cell = UnsafeCell::new(
            renderer
                .device
                .create_command_encoder(&CommandEncoderDescriptor::default()),
        );
        let rpass_temps_cell = UnsafeCell::new(RpassTemporaryPool::new());

        let mut next_rpass_idx = 0;
        let mut rpass = None;

        // Iterate through all the nodes and actually execute them.
        for (idx, mut node) in pruned_node_list.into_iter().enumerate() {
            if acquire_idx == Some(idx) {
                // SAFETY: this drops the renderpass, letting us into everything it was
                // borrowing.
                rpass = None;

                // SAFETY: the renderpass has died, so there are no outstanding immutible
                // borrows of the structure, and all uses of the temporaries have died.
                unsafe { (*rpass_temps_cell.get()).clear() };

                cmd_bufs.push(
                    mem::replace(
                        // SAFETY: There are two things which borrow this encoder: the renderpass and the node's
                        // encoder reference. Both of these have died by this point.
                        unsafe { &mut *encoder_cell.get() },
                        renderer
                            .device
                            .create_command_encoder(&CommandEncoderDescriptor::default()),
                    )
                    .finish(),
                );

                // Early submit before acquire
                renderer.queue.submit(cmd_bufs.drain(..));

                // TODO: error
                // SAFETY: Same context as the above unsafe.
                unsafe { &mut *output_cell.get() }.acquire().unwrap();
            }

            if !compatible[idx] {
                // SAFETY: this drops the renderpass, letting us into everything it was
                // borrowing when we make the new renderpass.
                rpass = None;

                if let Some(ref desc) = node.rpass {
                    rpass = Some(Self::create_rpass_from_desc(
                        desc,
                        // SAFETY: There are two things which borrow this encoder: the renderpass and the node's
                        // encoder reference. Both of these have died by this point.
                        unsafe { &mut *encoder_cell.get() },
                        idx,
                        renderpass_ends[next_rpass_idx],
                        // SAFETY: Same context as above.
                        unsafe { &mut *output_cell.get() },
                        &resource_spans,
                        &active_views,
                        shadow_views,
                    ));
                }
                next_rpass_idx += 1;
            }

            {
                let store = RenderGraphDataStore {
                    texture_mapping: &active_views,
                    shadow_coordinates: data_core.directional_light_manager.get_coords(),
                    shadow_views: data_core.directional_light_manager.get_layer_views(),
                    data: &self.data,
                    // SAFETY: This is only viewed mutably when no renderpass exists
                    output: unsafe { &*output_cell.get() }.as_view(),
                };

                let mut encoder_or_rpass = match rpass {
                    Some(ref mut rpass) => RenderGraphEncoderOrPassInner::RenderPass(rpass),
                    // SAFETY: There is no active renderpass to borrow this. This reference lasts for the duration of
                    // the call to exec.
                    None => {
                        RenderGraphEncoderOrPassInner::Encoder(unsafe { &mut *encoder_cell.get() })
                    }
                };

                profiling::scope!(&node.label);

                data_core.profiler.begin_scope(
                    &node.label,
                    &mut encoder_or_rpass,
                    &renderer.device,
                );

                (node.exec)(
                    &mut node.passthrough,
                    renderer,
                    RenderGraphEncoderOrPass(encoder_or_rpass),
                    // SAFETY: This borrow, and all the objects allocated from it, lasts as long as the renderpass, and
                    // isn't used mutably until after the rpass dies
                    unsafe { &*rpass_temps_cell.get() },
                    ready_output,
                    store,
                );

                let mut encoder_or_rpass = match rpass {
                    Some(ref mut rpass) => RenderGraphEncoderOrPassInner::RenderPass(rpass),
                    // SAFETY: There is no active renderpass to borrow this. This reference lasts for the duration of
                    // the call to exec.
                    None => {
                        RenderGraphEncoderOrPassInner::Encoder(unsafe { &mut *encoder_cell.get() })
                    }
                };

                data_core.profiler.end_scope(&mut encoder_or_rpass);
            }
        }

        // SAFETY: We drop the renderpass to make sure we can access both encoder_cell
        // and output_cell safely
        drop(rpass);

        // SAFETY: the renderpass has dropped, and so has all the uses of the data, and
        // the immutable borrows of the allocator.
        unsafe { (*rpass_temps_cell.get()).clear() }
        drop(rpass_temps_cell);

        // SAFETY: this is safe as we've dropped all renderpasses that possibly borrowed
        // it
        cmd_bufs.push(encoder_cell.into_inner().finish());

        let mut resolve_encoder =
            renderer
                .device
                .create_command_encoder(&CommandEncoderDescriptor {
                    label: Some("profile resolve encoder"),
                });
        data_core.profiler.resolve_queries(&mut resolve_encoder);
        cmd_bufs.push(resolve_encoder.finish());

        renderer.queue.submit(cmd_bufs);

        // SAFETY: this is safe as we've dropped all renderpasses that possibly borrowed
        // it
        output_cell.into_inner().present();

        data_core.profiler.end_frame().unwrap();
        data_core.profiler.process_finished_frame()
    }

    #[allow(clippy::too_many_arguments)]
    fn create_rpass_from_desc<'rpass>(
        desc: &RenderPassTargets,
        encoder: &'rpass mut CommandEncoder,
        node_idx: usize,
        pass_end_idx: usize,
        output: &'rpass OutputFrame,
        resource_spans: &'rpass HashMap<GraphResource, (usize, Option<usize>)>,
        active_views: &'rpass HashMap<usize, TextureView>,
        shadow_views: &'rpass [TextureView],
    ) -> RenderPass<'rpass> {
        let color_attachments: Vec<_> = desc
            .targets
            .iter()
            .map(|target| {
                let view_span = resource_spans[&target.color.handle.resource];

                let load = if view_span.0 == node_idx {
                    LoadOp::Clear(target.clear)
                } else {
                    LoadOp::Load
                };

                let store = view_span.1 != Some(pass_end_idx);

                RenderPassColorAttachment {
                    view: match &target.color.handle.resource {
                        GraphResource::OutputTexture => output
                            .as_view()
                            .expect("internal rendergraph error: tried to use output texture before acquire"),
                        GraphResource::Texture(t) => &active_views[t],
                        _ => {
                            panic!("internal rendergraph error: using a non-texture as a renderpass attachment")
                        }
                    },
                    resolve_target: target.resolve.as_ref().map(|dep| match &dep.handle.resource {
                        GraphResource::OutputTexture => output
                            .as_view()
                            .expect("internal rendergraph error: tried to use output texture before acquire"),
                        GraphResource::Texture(t) => &active_views[t],
                        _ => {
                            panic!("internal rendergraph error: using a non-texture as a renderpass attachment")
                        }
                    }),
                    ops: Operations { load, store },
                }
            })
            .collect();
        let depth_stencil_attachment = desc.depth_stencil.as_ref().map(|ds_target| {
            let resource = match ds_target.target {
                DepthHandle::RenderTarget(ref dep) => dep.handle.resource,
                DepthHandle::Shadow(ref s) => GraphResource::Shadow(s.handle.idx),
            };

            let view_span = resource_spans[&resource];

            let store = view_span.1 != Some(pass_end_idx);

            let depth_ops = ds_target.depth_clear.map(|clear| {
                let load = if view_span.0 == node_idx {
                    LoadOp::Clear(clear)
                } else {
                    LoadOp::Load
                };

                Operations { load, store }
            });

            let stencil_load = ds_target.stencil_clear.map(|clear| {
                let load = if view_span.0 == node_idx {
                    LoadOp::Clear(clear)
                } else {
                    LoadOp::Load
                };

                Operations { load, store }
            });

            RenderPassDepthStencilAttachment {
                view: match &resource {
                    GraphResource::OutputTexture => output
                        .as_view()
                        .expect("internal rendergraph error: tried to use output texture before acquire"),
                    GraphResource::Texture(t) => &active_views[t],
                    GraphResource::Shadow(s) => &shadow_views[*s],
                    _ => {
                        panic!("internal rendergraph error: using a non-texture as a renderpass attachment")
                    }
                },
                depth_ops,
                stencil_ops: stencil_load,
            }
        });

        // TODO: Properly read viewport
        // rpass.set_viewport(
        //     shadow_map.offset.x as f32,
        //     shadow_map.offset.y as f32,
        //     shadow_map.size as f32,
        //     shadow_map.size as f32,
        //     0.0,
        //     1.0,
        // );
        encoder.begin_render_pass(&RenderPassDescriptor {
            label: None,
            color_attachments: &color_attachments,
            depth_stencil_attachment,
        })
    }
}

impl<'node> Default for RenderGraph<'node> {
    fn default() -> Self {
        Self::new()
    }
}
