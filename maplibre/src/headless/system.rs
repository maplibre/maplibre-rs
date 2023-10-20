use std::{borrow::Cow, sync::Arc};

use crate::{
    context::MapContext,
    render::{
        resource::{BufferedTextureHead, Head},
        Renderer,
    },
    tcs::system::System,
};

/// Stage which writes the current contents of the GPU/CPU buffer in [`BufferedTextureHead`]
/// to disk as PNG.
pub struct WriteSurfaceBufferSystem {
    frame: u64,
    write_to_disk: bool,
}

impl WriteSurfaceBufferSystem {
    pub fn new(write_to_disk: bool) -> Self {
        Self {
            frame: 0,
            write_to_disk,
        }
    }
}

impl System for WriteSurfaceBufferSystem {
    fn name(&self) -> Cow<'static, str> {
        "write_surfaced_buffer".into()
    }

    fn run(
        &mut self,
        MapContext {
            renderer:
                Renderer {
                    resources: state,
                    device,
                    ..
                },
            ..
        }: &mut MapContext,
    ) {
        let surface = state.surface();
        match surface.head() {
            Head::Headed(_) => {}
            Head::Headless(buffered_texture) => {
                let buffered_texture: Arc<BufferedTextureHead> = buffered_texture.clone();

                let device = device.clone();
                let current_frame = self.frame;

                let buffer_slice = buffered_texture.map_async(&device);
                let padded_buffer = buffer_slice.get_mapped_range();

                if self.write_to_disk {
                    buffered_texture
                        .write_png(
                            &padded_buffer,
                            format!("frame_{current_frame}.png").as_str(),
                        )
                        .expect("Could save frame to disk");
                }

                // With the current interface, we have to make sure all mapped views are
                // dropped before we unmap the buffer.
                drop(padded_buffer);
                buffered_texture.unmap();

                self.frame += 1;
            }
        }
    }
}
