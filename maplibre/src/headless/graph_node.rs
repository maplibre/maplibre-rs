use crate::{
    render::{
        graph::{Node, NodeRunError, RenderContext, RenderGraphContext, SlotInfo},
        resource::Head,
        RenderResources,
    },
    tcs::world::World,
};

/// Node which copies the contents of the GPU-side texture in [`BufferedTextureHead`] to an
/// unmapped GPU-side buffer. This buffer will be mapped in
/// [`crate::render::stages::write_surface_buffer_stage::WriteSurfaceBufferStage`].
#[derive(Default)]
pub struct CopySurfaceBufferNode;

impl Node for CopySurfaceBufferNode {
    fn input(&self) -> Vec<SlotInfo> {
        vec![]
    }

    fn update(&mut self, _state: &mut RenderResources) {}

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        RenderContext {
            command_encoder, ..
        }: &mut RenderContext,
        state: &RenderResources,
        _world: &World,
    ) -> Result<(), NodeRunError> {
        let surface = state.surface();
        match surface.head() {
            Head::Headed(_) => {}
            Head::Headless(buffered_texture) => {
                let size = surface.size();
                command_encoder.copy_texture_to_buffer(
                    buffered_texture.copy_texture(),
                    wgpu::ImageCopyBuffer {
                        buffer: buffered_texture.buffer(),
                        layout: wgpu::ImageDataLayout {
                            offset: 0,
                            bytes_per_row: Some(buffered_texture.bytes_per_row()),
                            rows_per_image: None,
                        },
                    },
                    wgpu::Extent3d {
                        width: size.width(),
                        height: size.height(),
                        depth_or_array_layers: 1,
                    },
                );
            }
        }

        Ok(())
    }
}
