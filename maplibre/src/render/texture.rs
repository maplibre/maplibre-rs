pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
}

pub const DEPTH_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth24PlusStencil8;

impl Texture {
    pub fn create_depth_texture(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        sample_count: u32,
    ) -> Self {
        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count,
            dimension: wgpu::TextureDimension::D2,
            format: DEPTH_TEXTURE_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        });
        let view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            texture: depth_texture,
            view,
        }
    }

    /// Creates a texture that uses MSAA and fits a given swap chain
    pub fn create_multisampling_texture(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
        sample_count: u32,
    ) -> Texture {
        let multisampling_texture = &wgpu::TextureDescriptor {
            label: Some("Multisampled frame descriptor"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        };

        let texture = device.create_texture(multisampling_texture);
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        Self { texture, view }
    }
}
