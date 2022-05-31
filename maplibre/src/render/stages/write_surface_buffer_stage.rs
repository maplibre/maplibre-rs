//! Sorts items of the [RenderPhases](RenderPhase).

use crate::context::MapContext;
use crate::coords::{ViewRegion, Zoom};
use crate::io::tile_repository::TileRepository;
use crate::render::camera::ViewProjection;
use crate::render::render_phase::RenderPhase;
use crate::render::resource::{BufferDimensions, BufferedTextureHead, Head, IndexEntry};
use crate::render::shaders::{
    ShaderCamera, ShaderFeatureStyle, ShaderGlobals, ShaderLayerMetadata, Vec4f32,
};
use crate::render::tile_view_pattern::TileInView;
use crate::render::util::Eventually::Initialized;
use crate::schedule::Stage;
use crate::{RenderState, Renderer, Style};
use std::fs::File;
use std::future::Future;
use std::io::Write;
use std::iter;
use std::ops::Deref;
use tokio::runtime::Handle;
use tokio::task;
use wgpu::{BufferAsyncError, BufferSlice};

#[derive(Default)]
pub struct WriteSurfaceBufferStage {
    frame: u64,
}

impl Stage for WriteSurfaceBufferStage {
    fn run(
        &mut self,
        MapContext {
            renderer: Renderer { state, device, .. },
            ..
        }: &mut MapContext,
    ) {
        match state.surface.head() {
            Head::Headed(_) => {}
            Head::Headless(buffered_texture) => {
                let buffered_texture = buffered_texture.clone();

                let device = device.clone();
                let current_frame = self.frame;

                task::block_in_place(|| {
                    Handle::current().block_on(async {
                        buffered_texture
                            .create_png(&device, format!("frame_{}.png", current_frame).as_str())
                            .await;
                    })
                });

                self.frame += 1;
            }
        }
    }
}
