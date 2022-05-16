use crate::render::graph::{Node, NodeRunError, RenderContext, RenderGraphContext, SlotInfo};
use crate::render::render_phase::{DrawFunctionId, PhaseItem, TrackedRenderPass};
use crate::render::util::FloatOrd;
use crate::render::Eventually::Initialized;
use crate::render::RenderState;
use std::ops::{Deref, Range};

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub struct PipelineID(usize);

pub struct Transparent2d {
    pub sort_key: FloatOrd,
    pub pipeline: PipelineID,
    pub draw_function: DrawFunctionId,
    /// Range in the vertex buffer of this item
    pub batch_range: Option<Range<u32>>,
}

impl PhaseItem for Transparent2d {
    type SortKey = FloatOrd;

    #[inline]
    fn sort_key(&self) -> Self::SortKey {
        self.sort_key
    }

    #[inline]
    fn draw_function(&self) -> DrawFunctionId {
        self.draw_function
    }
}

// Plugins that contribute to the RenderGraph should use the following label conventions:
// 1. Graph modules should have a NAME, input module, and node module (where relevant)
// 2. The "top level" graph is the plugin module root. Just add things like `pub mod node` directly under the plugin module
// 3. "sub graph" modules should be nested beneath their parent graph module

pub mod node {
    pub const MAIN_PASS_DEPENDENCIES: &str = "main_pass_dependencies";
    pub const MAIN_PASS_DRIVER: &str = "main_pass_driver";
}

pub mod draw_graph {
    pub const NAME: &str = "draw";
    pub mod input {}
    pub mod node {
        pub const MAIN_PASS: &str = "main_pass";
    }
}

pub struct MainPassNode {}

impl MainPassNode {
    pub fn new() -> Self {
        Self {}
    }
}

impl Node for MainPassNode {
    fn input(&self) -> Vec<SlotInfo> {
        vec![]
    }

    fn update(&mut self, _state: &mut RenderState) {}

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        state: &RenderState,
    ) -> Result<(), NodeRunError> {
        let (render_target, multisampling_texture, depth_texture) = if let (
            Initialized(render_target),
            Initialized(multisampling_texture),
            Initialized(depth_texture),
        ) = (
            &state.render_target,
            &state.multisampling_texture,
            &state.depth_texture,
        ) {
            (render_target, multisampling_texture, depth_texture)
        } else {
            return Ok(());
        };

        let color_attachment = if let Some(texture) = multisampling_texture {
            wgpu::RenderPassColorAttachment {
                view: &texture.view,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                    store: true,
                },
                resolve_target: Some(render_target.deref()),
            }
        } else {
            wgpu::RenderPassColorAttachment {
                view: render_target.deref(),
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                    store: true,
                },
                resolve_target: None,
            }
        };

        let render_pass =
            render_context
                .command_encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[color_attachment],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &depth_texture.view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(0.0),
                            store: true,
                        }),
                        stencil_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(0),
                            store: true,
                        }),
                    }),
                });

        let _tracked_pass = TrackedRenderPass::new(render_pass);

        /*let index = self.buffer_pool.index();

        for TileInView { shape, fallback } in self.tile_view_pattern.iter() {
            let coords = shape.coords;
            tracing::trace!("Drawing tile at {coords}");

            let shape_to_render = fallback.as_ref().unwrap_or(shape);

            let reference =
                self.tile_view_pattern
                    .stencil_reference_value(&shape_to_render.coords) as u32;

            // Draw mask
            // FIXME

            if let Some(entries) = index.get_layers(&shape_to_render.coords) {
                let mut layers_to_render: Vec<&IndexEntry> = Vec::from_iter(entries);
                layers_to_render.sort_by_key(|entry| entry.style_layer.index);

                for entry in layers_to_render {
                    // Draw tile
                    // FIXME
                }
            } else {
                tracing::trace!("No layers found at {}", &shape_to_render.coords);
            }
        }*/

        /*let mut draw_functions = DrawFunctions::default();
        for item in &transparent_phase.items {
            let draw_function = draw_functions.get_mut(item.draw_function).unwrap();
            draw_function.draw(world, &mut tracked_pass, view_entity, item);
        }*/
        Ok(())
    }
}

pub struct MainPassDriverNode;

impl Node for MainPassDriverNode {
    fn run(
        &self,
        graph: &mut RenderGraphContext,
        _render_context: &mut RenderContext,
        _state: &RenderState,
    ) -> Result<(), NodeRunError> {
        graph.run_sub_graph(draw_graph::NAME, vec![])?;

        Ok(())
    }
}
