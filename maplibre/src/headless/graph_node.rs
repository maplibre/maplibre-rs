use crate::render::{
    graph::{Node, NodeRunError, RenderContext, RenderGraphContext, SlotInfo},
    resource::Head,
    RenderState,
};

/// Node which copies the contents of the GPU-side texture in [`BufferedTextureHead`] to an
/// unmapped GPU-side buffer. This buffer will be mapped in
/// [`crate::render::stages::write_surface_buffer_stage::WriteSurfaceBufferStage`].
#[derive(Default)]
pub struct CopySurfaceBufferNode {}

impl CopySurfaceBufferNode {
    pub fn new() -> Self {
        Self {}
    }
}

impl Node for CopySurfaceBufferNode {
    fn input(&self) -> Vec<SlotInfo> {
        vec![]
    }

    fn update(&mut self, _state: &mut RenderState) {}

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        RenderContext {
            command_encoder, ..
        }: &mut RenderContext,
        state: &RenderState,
    ) -> Result<(), NodeRunError> {
        let surface = state.surface();
        match surface.head() {
            Head::Headed(_) => {}
            Head::Headless(buffered_texture) => {
                let size = surface.size();
                command_encoder.copy_texture_to_buffer(
                    buffered_texture.texture.as_image_copy(),
                    wgpu::ImageCopyBuffer {
                        buffer: &buffered_texture.output_buffer,
                        layout: wgpu::ImageDataLayout {
                            offset: 0,
                            bytes_per_row: Some(
                                std::num::NonZeroU32::new(
                                    buffered_texture.buffer_dimensions.padded_bytes_per_row as u32,
                                )
                                .unwrap(), // TODO: remove unwrap
                            ),
                            rows_per_image: None,
                        },
                    },
                    wgpu::Extent3d {
                        width: size.width() as u32,
                        height: size.height() as u32,
                        depth_or_array_layers: 1,
                    },
                );
            }
        }

        Ok(())
    }
}
