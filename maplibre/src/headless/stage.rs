use std::sync::Arc;

use tokio::{runtime::Handle, task};

use crate::{
    context::MapContext,
    render::{
        resource::{BufferedTextureHead, Head},
        Renderer,
    },
    schedule::Stage,
};

/// Stage which writes the current contents of the GPU/CPU buffer in [`BufferedTextureHead`]
/// to disk as PNG.
pub struct WriteSurfaceBufferStage {
    frame: u64,
    write_to_disk: bool,
}

impl WriteSurfaceBufferStage {
    pub fn new(write_to_disk: bool) -> Self {
        Self {
            frame: 0,
            write_to_disk,
        }
    }
}

impl Stage for WriteSurfaceBufferStage {
    fn run(
        &mut self,
        MapContext {
            renderer: Renderer { state, device, .. },
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

                if self.write_to_disk {
                    task::block_in_place(|| {
                        Handle::current().block_on(async {
                            buffered_texture
                                .create_png(
                                    &device,
                                    format!("frame_{}.png", current_frame).as_str(),
                                )
                                .await;
                        })
                    });
                }

                self.frame += 1;
            }
        }
    }
}
