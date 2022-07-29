use wgpu::util::DeviceExt;

use crate::render::{
    resource::{BackingBufferDescriptor, BufferPool, Globals, RenderPipeline, Texture},
    settings::{Msaa, RendererSettings},
    shaders,
    shaders::{RasterTileShader, Shader, ShaderTextureVertex},
    tile_pipeline::TilePipeline,
};

pub const INDICES: &[u32] = &[0, 1, 3, 1, 2, 3];

pub const VERTICES: &[ShaderTextureVertex] = &[
    ShaderTextureVertex {
        position: [1.0, 1.0, 0.0],
        tex_coords: [1.0, 1.0], // D
    }, // D
    ShaderTextureVertex {
        position: [1.0, -1.0, 0.0],
        tex_coords: [1.0, 0.0], // C
    }, // C
    ShaderTextureVertex {
        position: [-1.0, -1.0, 0.0],
        tex_coords: [0.0, 0.0], // B
    }, // B
    ShaderTextureVertex {
        position: [-1.0, 1.0, 0.0],
        tex_coords: [0.0, 1.0], // A
    }, // A
];

// pub const VERTICES: &[ShaderTextureVertex] = &[
//     ShaderTextureVertex {
//         position: [-1.0, 1.0, 0.0],
//         tex_coords: [0.0, 1.0],
//     }, // A
//     ShaderTextureVertex {
//         position: [1.0, 1.0, 0.0],
//         tex_coords: [1.0, 1.0],
//     }, // D
//     ShaderTextureVertex {
//         position: [1.0, -1.0, 0.0],
//         tex_coords: [1.0, 0.0],
//     }, // C
//     ShaderTextureVertex {
//         position: [-1.0, -1.0, 0.0],
//         tex_coords: [0.0, 0.0],
//     }, // B
// ];

// pub const VERTICES: &[ShaderTextureVertex] = &[
//     ShaderTextureVertex {
//         position: [-1.0, 1.0, 0.0],
//         tex_coords: [0.0, 1.0],
//     }, // A
//     ShaderTextureVertex {
//         position: [1.0, 1.0, 0.0],
//         tex_coords: [1.0, 0.0],
//     }, // D
//     ShaderTextureVertex {
//         position: [1.0, -1.0, 0.0],
//         tex_coords: [1.0, 1.0],
//     }, // C
//     ShaderTextureVertex {
//         position: [-1.0, -1.0, 0.0],
//         tex_coords: [0.0, 0.0],
//     }, // B
// ];

// pub const VERTICES: &[ShaderTextureVertex] = &[
//     ShaderTextureVertex {
//         position: [-1.0, 1.0, 0.0],
//         tex_coords: [1.0, 0.0],
//     }, // A
//     ShaderTextureVertex {
//         position: [1.0, 1.0, 0.0],
//         tex_coords: [1.0, 1.0],
//     }, // D
//     ShaderTextureVertex {
//         position: [1.0, -1.0, 0.0],
//         tex_coords: [0.0, 1.0],
//     }, // C
//     ShaderTextureVertex {
//         position: [-1.0, -1.0, 0.0],
//         tex_coords: [0.0, 0.0],
//     }, // B
// ];

// pub const VERTICES: &[ShaderTextureVertex] = &[
//     ShaderTextureVertex {
//         position: [-1.0, 1.0, 0.0],
//         tex_coords: [0.0, 1.0],
//     }, // A
//     ShaderTextureVertex {
//         position: [-1.0, -1.0, 0.0],
//         tex_coords: [1.0, 1.0],
//     }, // B
//     ShaderTextureVertex {
//         position: [1.0, -1.0, 0.0],
//         tex_coords: [1.0, 0.0],
//     }, // C
//     ShaderTextureVertex {
//         position: [1.0, 1.0, 0.0],
//         tex_coords: [0.0, 0.0],
//     }, // D
// ];

pub struct RasterResources {
    pub sampler: Option<wgpu::Sampler>,
    pub msaa: Option<Msaa>,
    pub texture: Option<Texture>,
    pub pipeline: Option<wgpu::RenderPipeline>,
    pub bind_group: Option<wgpu::BindGroup>,
    pub index_buffer: Option<wgpu::Buffer>,
    pub vertex_buffer: Option<wgpu::Buffer>,
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
        tile_vertex: &ShaderTextureVertex,
        tile_fragment: &ShaderTextureVertex,
    ) {
        self.pipeline = Some(
            TilePipeline::new(
                settings.msaa,
                tile_vertex.describe_vertex(),
                tile_fragment.describe_fragment(),
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

    pub fn set_vertex_buffer(&mut self, device: &wgpu::Device) {
        self.vertex_buffer = Some(
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(VERTICES),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            }),
        );
    }

    pub fn set_index_buffer(&mut self, device: &wgpu::Device) {
        self.index_buffer = Some(
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(INDICES),
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            }),
        );
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
            index_buffer: None,
            vertex_buffer: None,
        }
    }
}
