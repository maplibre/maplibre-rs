use crate::render::{
    resource::{RenderPipeline, Texture},
    settings::{Msaa, RendererSettings},
    shaders::{RasterTileShader, Shader, ShaderTextureVertex},
    tile_pipeline::TilePipeline,
};

pub const INDICES: &[u32] = &[0, 1, 3, 1, 2, 3];

pub const ROOT: &[ShaderTextureVertex] = &[
    ShaderTextureVertex {
        position: [-1.0, 1.0],
        tex_coords: [0.0, 0.0], // A
    }, // A
    ShaderTextureVertex {
        position: [-1.0, -1.0],
        tex_coords: [0.0, 1.0], // B
    }, // B
    ShaderTextureVertex {
        position: [1.0, -1.0],
        tex_coords: [1.0, 1.0], // C
    }, // C
    ShaderTextureVertex {
        position: [1.0, 1.0],
        tex_coords: [1.0, 0.0], // D
    }, // D
];

pub const UPPER_LEFT: &[ShaderTextureVertex] = &[
    ShaderTextureVertex {
        position: [-1.0, 1.0],
        tex_coords: [0.0, 0.0], // A
    }, // A
    ShaderTextureVertex {
        position: [-1.0, 0.0],
        tex_coords: [0.0, 1.0], // B
    }, // B
    ShaderTextureVertex {
        position: [0.0, 0.0],
        tex_coords: [1.0, 1.0], // C
    }, // C
    ShaderTextureVertex {
        position: [0.0, 1.0],
        tex_coords: [1.0, 0.0], // D
    }, // D
];

pub const UPPER_RIGHT: &[ShaderTextureVertex] = &[
    ShaderTextureVertex {
        position: [0.0, 1.0],
        tex_coords: [0.0, 0.0], // A
    }, // A
    ShaderTextureVertex {
        position: [0.0, 0.0],
        tex_coords: [0.0, 1.0], // B
    }, // B
    ShaderTextureVertex {
        position: [1.0, 0.0],
        tex_coords: [1.0, 1.0], // C
    }, // C
    ShaderTextureVertex {
        position: [1.0, 1.0],
        tex_coords: [1.0, 0.0], // D
    }, // D
];

pub const LOWER_LEFT: &[ShaderTextureVertex] = &[
    ShaderTextureVertex {
        position: [-1.0, 0.0],
        tex_coords: [0.0, 0.0], // A
    }, // A
    ShaderTextureVertex {
        position: [-1.0, -1.0],
        tex_coords: [0.0, 1.0], // B
    }, // B
    ShaderTextureVertex {
        position: [0.0, -1.0],
        tex_coords: [1.0, 1.0], // C
    }, // C
    ShaderTextureVertex {
        position: [0.0, 0.0],
        tex_coords: [1.0, 0.0], // D
    }, // D
];

pub const LOWER_RIGHT: &[ShaderTextureVertex] = &[
    ShaderTextureVertex {
        position: [0.0, 0.0],
        tex_coords: [0.0, 0.0], // A
    }, // A
    ShaderTextureVertex {
        position: [0.0, -1.0],
        tex_coords: [0.0, 1.0], // B
    }, // B
    ShaderTextureVertex {
        position: [1.0, -1.0],
        tex_coords: [1.0, 1.0], // C
    }, // C
    ShaderTextureVertex {
        position: [1.0, 0.0],
        tex_coords: [1.0, 0.0], // D
    }, // D
];

pub struct RasterResources {
    pub sampler: Option<wgpu::Sampler>,
    pub msaa: Option<Msaa>,
    pub texture: Option<Texture>,
    pub pipeline: Option<wgpu::RenderPipeline>,
    pub bind_group: Option<wgpu::BindGroup>,
}

impl RasterResources {
    pub fn set_msaa(&mut self, msaa: Msaa) {
        self.msaa = Some(msaa);
    }

    pub fn set_texture(
        &mut self,
        label: wgpu::Label,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        width: u32,
        height: u32,
        usage: wgpu::TextureUsages,
    ) {
        self.texture = Some(Texture::new(
            label,
            device,
            format,
            width,
            height,
            self.msaa.unwrap().clone(),
            usage,
        ));
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
        shader: &RasterTileShader,
    ) {
        self.pipeline = Some(
            TilePipeline::new(
                *settings,
                shader.describe_vertex(),
                shader.describe_fragment(),
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
        self.bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.pipeline.as_ref().unwrap().get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &self.texture.as_ref().unwrap().view,
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(self.sampler.as_ref().unwrap()),
                },
            ],
            label: None,
        }));
    }
}

impl Default for RasterResources {
    fn default() -> Self {
        RasterResources {
            sampler: None,
            msaa: None,
            texture: None,
            pipeline: None,
            bind_group: None,
        }
    }
}
