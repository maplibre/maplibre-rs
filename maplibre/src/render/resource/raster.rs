use crate::render::{
    resource::{BackingBufferDescriptor, BufferPool, Globals, RenderPipeline, Texture},
    settings::RendererSettings,
    shaders,
    shaders::{RasterTileShader, Shader},
    tile_pipeline::TilePipeline,
};

pub struct RasterResources {
    pub sampler: Option<wgpu::Sampler>,
    pub view: Option<wgpu::TextureView>,
    pub raster_pipeline: Option<wgpu::RenderPipeline>,
    pub raster_bind_group: Option<wgpu::BindGroup>,
}

impl RasterResources {
    pub fn set_view(&mut self, texture: &wgpu::Texture) {
        self.view = Some(texture.create_view(&wgpu::TextureViewDescriptor::default()));
    }

    pub fn set_sampler(&mut self, device: &wgpu::Device) {
        self.sampler = Some(device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        }));
    }

    pub fn set_raster_pipeline(
        &mut self,
        device: &wgpu::Device,
        settings: &RendererSettings,
        tile_shader: &RasterTileShader,
    ) {
        self.raster_pipeline = Some(
            TilePipeline::new(
                settings.msaa,
                tile_shader.describe_vertex(),
                tile_shader.describe_fragment(),
                false,
                false,
                false,
                false,
                true,
            )
            .describe_render_pipeline()
            .initialize(device),
        );
    }

    pub fn set_raster_bind_group(&mut self, device: &wgpu::Device) {
        self.raster_bind_group = Some(
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self
                    .raster_pipeline
                    .as_ref()
                    .unwrap()
                    .get_bind_group_layout(0),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(self.view.as_ref().unwrap()),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(self.sampler.as_ref().unwrap()),
                    },
                ],
                label: None,
            }),
        );
    }
}

impl Default for RasterResources {
    fn default() -> Self {
        RasterResources {
            sampler: None,
            view: None,
            raster_pipeline: None,
            raster_bind_group: None,
        }
    }
}
