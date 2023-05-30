//! A render pass which allows tracking, for example using a tracing framework.

use std::ops::Range;

use log::trace;

/// A [`RenderPass`], which tracks the current pipeline state to ensure all draw calls are valid.
/// It is used to set the current [`RenderPipeline`], [`BindGroups`](BindGroup) and buffers.
/// After all requirements are specified, draw calls can be issued.
pub struct TrackedRenderPass<'a> {
    pass: wgpu::RenderPass<'a>,
}

impl<'a> TrackedRenderPass<'a> {
    /// Tracks the supplied render pass.
    pub fn new(pass: wgpu::RenderPass<'a>) -> Self {
        Self { pass }
    }

    /// Sets the active [`RenderPipeline`].
    ///
    /// Subsequent draw calls will exhibit the behavior defined by the `pipeline`.
    pub fn set_render_pipeline(&mut self, pipeline: &'a wgpu::RenderPipeline) {
        trace!("set pipeline: {pipeline:?}");
        self.pass.set_pipeline(pipeline);
    }

    /// Sets the active [`BindGroup`] for a given bind group index. The bind group layout in the
    /// active pipeline when any `draw()` function is called must match the layout of this `bind group`.
    pub fn set_bind_group(
        &mut self,
        index: usize,
        bind_group: &'a wgpu::BindGroup,
        dynamic_uniform_indices: &[u32],
    ) {
        self.pass
            .set_bind_group(index as u32, bind_group, dynamic_uniform_indices);
    }

    /// Assign a vertex buffer to a slot.
    ///
    /// Subsequent calls to [`TrackedRenderPass::draw`] and [`TrackedRenderPass::draw_indexed`]
    /// will use the buffer referenced by `buffer_slice` as one of the source vertex buffer(s).
    ///
    /// The `slot_index` refers to the index of the matching descriptor in
    /// [`VertexState::buffers`](crate::render_resource::VertexState::buffers).
    pub fn set_vertex_buffer(&mut self, slot_index: usize, buffer_slice: wgpu::BufferSlice<'a>) {
        self.pass.set_vertex_buffer(slot_index as u32, buffer_slice);
    }

    /// Sets the active index buffer.
    ///
    /// Subsequent calls to [`TrackedRenderPass::draw_indexed`] will use the buffer referenced by
    /// `buffer_slice` as the source index buffer.
    pub fn set_index_buffer(
        &mut self,
        buffer_slice: wgpu::BufferSlice<'a>,
        index_format: wgpu::IndexFormat,
    ) {
        self.pass.set_index_buffer(buffer_slice, index_format);
    }

    /// Draws primitives from the active vertex buffer(s).
    ///
    /// The active vertex buffer(s) can be set with [`TrackedRenderPass::set_vertex_buffer`].
    pub fn draw(&mut self, vertices: Range<u32>, instances: Range<u32>) {
        trace!("draw: {vertices:?} {instances:?}");
        self.pass.draw(vertices, instances);
    }

    /// Draws indexed primitives using the active index buffer and the active vertex buffer(s).
    ///
    /// The active index buffer can be set with [`TrackedRenderPass::set_index_buffer`], while the
    /// active vertex buffer(s) can be set with [`TrackedRenderPass::set_vertex_buffer`].
    pub fn draw_indexed(&mut self, indices: Range<u32>, base_vertex: i32, instances: Range<u32>) {
        trace!("draw indexed: {indices:?} {base_vertex} {instances:?}");
        self.pass.draw_indexed(indices, base_vertex, instances);
    }

    /// Draws primitives from the active vertex buffer(s) based on the contents of the `indirect_buffer`.
    ///
    /// The active vertex buffers can be set with [`TrackedRenderPass::set_vertex_buffer`].
    ///
    /// The structure expected in `indirect_buffer` is the following:
    ///
    /// ```rust
    /// #[repr(C)]
    /// struct DrawIndirect {
    ///     vertex_count: u32, // The number of vertices to draw.
    ///     instance_count: u32, // The number of instances to draw.
    ///     first_vertex: u32, // The Index of the first vertex to draw.
    ///     first_instance: u32, // The instance ID of the first instance to draw.
    ///     // has to be 0, unless [`Features::INDIRECT_FIRST_INSTANCE`] is enabled.
    /// }
    /// ```
    pub fn draw_indirect(&mut self, indirect_buffer: &'a wgpu::Buffer, indirect_offset: u64) {
        trace!("draw indirect: {indirect_buffer:?} {indirect_offset}");
        self.pass.draw_indirect(indirect_buffer, indirect_offset);
    }

    /// Draws indexed primitives using the active index buffer and the active vertex buffers,
    /// based on the contents of the `indirect_buffer`.
    ///
    /// The active index buffer can be set with [`TrackedRenderPass::set_index_buffer`], while the active
    /// vertex buffers can be set with [`TrackedRenderPass::set_vertex_buffer`].
    ///
    /// The structure expected in `indirect_buffer` is the following:
    ///
    /// ```rust
    /// #[repr(C)]
    /// struct DrawIndexedIndirect {
    ///     vertex_count: u32, // The number of vertices to draw.
    ///     instance_count: u32, // The number of instances to draw.
    ///     first_index: u32, // The base index within the index buffer.
    ///     vertex_offset: i32, // The value added to the vertex index before indexing into the vertex buffer.
    ///     first_instance: u32, // The instance ID of the first instance to draw.
    ///     // has to be 0, unless [`Features::INDIRECT_FIRST_INSTANCE`] is enabled.
    /// }
    /// ```
    pub fn draw_indexed_indirect(
        &mut self,
        indirect_buffer: &'a wgpu::Buffer,
        indirect_offset: u64,
    ) {
        trace!("draw indexed indirect: {indirect_buffer:?} {indirect_offset}");
        self.pass
            .draw_indexed_indirect(indirect_buffer, indirect_offset);
    }

    /// Sets the stencil reference.
    ///
    /// Subsequent stencil tests will test against this value.
    pub fn set_stencil_reference(&mut self, reference: u32) {
        trace!("set stencil reference: {reference}");
        self.pass.set_stencil_reference(reference);
    }

    /// Sets the scissor region.
    ///
    /// Subsequent draw calls will discard any fragments that fall outside this region.
    pub fn set_scissor_rect(&mut self, x: u32, y: u32, width: u32, height: u32) {
        trace!("set_scissor_rect: {x} {y} {width} {height}");
        self.pass.set_scissor_rect(x, y, width, height);
    }

    /// Set push constant data.
    ///
    /// `Features::PUSH_CONSTANTS` must be enabled on the device in order to call these functions.
    pub fn set_push_constants(&mut self, stages: wgpu::ShaderStages, offset: u32, data: &[u8]) {
        trace!(
            "set push constants: {stages:?} offset: {offset} data.len: {}",
            data.len()
        );
        self.pass.set_push_constants(stages, offset, data);
    }

    /// Set the rendering viewport.
    ///
    /// Subsequent draw calls will be projected into that viewport.
    pub fn set_viewport(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        min_depth: f32,
        max_depth: f32,
    ) {
        trace!("set viewport: {x} {y} {width} {height} {min_depth} {max_depth}");
        self.pass
            .set_viewport(x, y, width, height, min_depth, max_depth);
    }

    /// Insert a single debug marker.
    ///
    /// This is a GPU debugging feature. This has no effect on the rendering itself.
    pub fn insert_debug_marker(&mut self, label: &str) {
        trace!("insert debug marker: {label}");
        self.pass.insert_debug_marker(label);
    }

    /// Start a new debug group.
    ///
    /// Push a new debug group over the internal stack. Subsequent render commands and debug
    /// markers are grouped into this new group, until [`pop_debug_group`] is called.
    ///
    /// ```
    /// # fn example(mut pass: maplibre::render::resource::TrackedRenderPass<'static>) {
    /// pass.push_debug_group("Render the car");
    /// // [setup pipeline etc...]
    /// pass.draw(0..64, 0..1);
    /// pass.pop_debug_group();
    /// # }
    /// ```
    ///
    /// Note that [`push_debug_group`] and [`pop_debug_group`] must always be called in pairs.
    ///
    /// This is a GPU debugging feature. This has no effect on the rendering itself.
    ///
    /// [`push_debug_group`]: TrackedRenderPass::push_debug_group
    /// [`pop_debug_group`]: TrackedRenderPass::pop_debug_group
    pub fn push_debug_group(&mut self, label: &str) {
        trace!("push_debug_group marker: {label}");
        self.pass.push_debug_group(label);
    }

    /// End the current debug group.
    ///
    /// Subsequent render commands and debug markers are not grouped anymore in
    /// this group, but in the previous one (if any) or the default top-level one
    /// if the debug group was the last one on the stack.
    ///
    /// Note that [`push_debug_group`] and [`pop_debug_group`] must always be called in pairs.
    ///
    /// This is a GPU debugging feature. This has no effect on the rendering itself.
    ///
    /// [`push_debug_group`]: TrackedRenderPass::push_debug_group
    /// [`pop_debug_group`]: TrackedRenderPass::pop_debug_group
    pub fn pop_debug_group(&mut self) {
        trace!("pop_debug_group");
        self.pass.pop_debug_group();
    }

    pub fn set_blend_constant(&mut self, color: wgpu::Color) {
        trace!("set blend constant: {color:?}");
        self.pass.set_blend_constant(color);
    }
}
