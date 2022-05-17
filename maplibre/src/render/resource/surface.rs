//! Utilities for handling surfaces which can be either headless or headed. A headed surface has
//! a handle to a window. A headless surface renders to a texture.

use crate::render::resource::texture::TextureView;
use crate::render::settings::RendererSettings;
use crate::render::util::HasChanged;
use crate::{MapWindow, WindowSize};
use std::mem::size_of;

struct BufferDimensions {
    width: usize,
    height: usize,
    unpadded_bytes_per_row: usize,
    padded_bytes_per_row: usize,
}

impl BufferDimensions {
    fn new(width: usize, height: usize) -> Self {
        let bytes_per_pixel = size_of::<u32>();
        let unpadded_bytes_per_row = width * bytes_per_pixel;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize;
        let padded_bytes_per_row_padding = (align - unpadded_bytes_per_row % align) % align;
        let padded_bytes_per_row = unpadded_bytes_per_row + padded_bytes_per_row_padding;
        Self {
            width,
            height,
            unpadded_bytes_per_row,
            padded_bytes_per_row,
        }
    }
}

pub struct WindowHead {
    surface: wgpu::Surface,
    surface_config: wgpu::SurfaceConfiguration,
}

impl WindowHead {
    pub fn configure(&self, device: &wgpu::Device) {
        self.surface.configure(device, &self.surface_config);
    }

    pub fn recreate_surface<MW>(&mut self, window: &MW, instance: &wgpu::Instance)
    where
        MW: MapWindow,
    {
        self.surface = unsafe { instance.create_surface(window.inner()) };
    }
    pub fn surface(&self) -> &wgpu::Surface {
        &self.surface
    }
}

pub struct BufferedTextureHead {
    texture: wgpu::Texture,
    output_buffer: wgpu::Buffer,
    buffer_dimensions: BufferDimensions,
}

pub enum Head {
    Headed(WindowHead),
    Headless(BufferedTextureHead),
}

pub struct Surface {
    size: WindowSize,
    head: Head,
}

impl Surface {
    pub fn from_window<MW>(
        instance: &wgpu::Instance,
        window: &MW,
        settings: &RendererSettings,
    ) -> Self
    where
        MW: MapWindow,
    {
        let size = window.size();
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: settings.texture_format,
            width: size.width(),
            height: size.height(),
            //present_mode: wgpu::PresentMode::Immediate,
            present_mode: wgpu::PresentMode::Fifo, // VSync
        };

        let surface = unsafe { instance.create_surface(window.inner()) };

        Self {
            size,
            head: Head::Headed(WindowHead {
                surface,
                surface_config,
            }),
        }
    }

    pub fn from_image<MW>(device: &wgpu::Device, window: &MW, settings: &RendererSettings) -> Self
    where
        MW: MapWindow,
    {
        let size = window.size();

        // It is a WebGPU requirement that ImageCopyBuffer.layout.bytes_per_row % wgpu::COPY_BYTES_PER_ROW_ALIGNMENT == 0
        // So we calculate padded_bytes_per_row by rounding unpadded_bytes_per_row
        // up to the next multiple of wgpu::COPY_BYTES_PER_ROW_ALIGNMENT.
        // https://en.wikipedia.org/wiki/Data_structure_alignment#Computing_padding
        let buffer_dimensions =
            BufferDimensions::new(size.width() as usize, size.height() as usize);
        // The output buffer lets us retrieve the data as an array
        let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (buffer_dimensions.padded_bytes_per_row * buffer_dimensions.height) as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Surface texture"),
            size: wgpu::Extent3d {
                width: size.width(),
                height: size.height(),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: settings.texture_format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        });

        Self {
            size,
            head: Head::Headless(BufferedTextureHead {
                texture,
                output_buffer,
                buffer_dimensions,
            }),
        }
    }

    #[tracing::instrument(name = "create_view", skip_all)]
    pub fn create_view(&self, device: &wgpu::Device) -> TextureView {
        match &self.head {
            Head::Headed(window) => {
                let WindowHead { surface, .. } = window;
                let frame = match surface.get_current_texture() {
                    Ok(view) => view,
                    Err(wgpu::SurfaceError::Outdated) => {
                        tracing::trace!("surface outdated");
                        window.configure(device);
                        surface
                            .get_current_texture()
                            .expect("Error reconfiguring surface")
                    }
                    err => err.expect("Failed to acquire next swap chain texture!"),
                };
                frame.into()
            }
            Head::Headless(BufferedTextureHead { texture, .. }) => texture
                .create_view(&wgpu::TextureViewDescriptor::default())
                .into(),
        }
    }

    pub fn size(&self) -> WindowSize {
        self.size
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.size = WindowSize::new(width, height).expect("Invalid size for resizing the surface.");
        match &mut self.head {
            Head::Headed(window) => {
                window.surface_config.height = height;
                window.surface_config.width = width;
            }
            Head::Headless(_) => {}
        }
    }

    pub fn reconfigure(&mut self, device: &wgpu::Device) {
        match &mut self.head {
            Head::Headed(window) => {
                if window.has_changed(&(self.size.width(), self.size.height())) {
                    window.configure(device);
                }
            }
            Head::Headless(_) => {}
        }
    }

    pub fn recreate<MW>(&mut self, window: &MW, instance: &wgpu::Instance)
    where
        MW: MapWindow,
    {
        match &mut self.head {
            Head::Headed(head) => {
                head.recreate_surface(window, instance);
            }
            Head::Headless(_) => {}
        }
    }

    pub fn head(&self) -> &Head {
        &self.head
    }

    pub fn head_mut(&mut self) -> &mut Head {
        &mut self.head
    }
}

impl HasChanged for WindowHead {
    type Criteria = (u32, u32);

    fn has_changed(&self, criteria: &Self::Criteria) -> bool {
        self.surface_config.height != criteria.0 || self.surface_config.width != criteria.1
    }
}
