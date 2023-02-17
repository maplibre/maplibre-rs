//! Prepares GPU-owned resources by initializing them if they are uninitialized or out-of-date.

use std::borrow::Cow;

use crate::{
    context::MapContext,
    render::{resource::Texture, Renderer},
    tcs::system::System,
};

#[derive(Default)]
pub struct ResourceSystem;

impl System for ResourceSystem {
    fn name(&self) -> Cow<'static, str> {
        "resource_system".into()
    }

    fn run(
        &mut self,
        MapContext {
            renderer:
                Renderer {
                    settings,
                    device,
                    resources: state,
                    ..
                },
            ..
        }: &mut MapContext,
    ) {
        let surface = &mut state.surface;

        let size = surface.size();

        surface.reconfigure(device);

        state
            .render_target
            .initialize(|| surface.create_view(device));

        state.depth_texture.reinitialize(
            || {
                Texture::new(
                    Some("depth texture"),
                    device,
                    settings.depth_texture_format,
                    size.width(),
                    size.height(),
                    settings.msaa,
                    wgpu::TextureUsages::RENDER_ATTACHMENT,
                )
            },
            &(size.width(), size.height()),
        );

        state.multisampling_texture.reinitialize(
            || {
                if settings.msaa.is_active() {
                    Some(Texture::new(
                        Some("multisampling texture"),
                        device,
                        surface.surface_format(),
                        size.width(),
                        size.height(),
                        settings.msaa,
                        wgpu::TextureUsages::RENDER_ATTACHMENT,
                    ))
                } else {
                    None
                }
            },
            &(size.width(), size.height()),
        );
    }
}
