//! Utilities for handling surfaces which can be either headless or headed. A headed surface has
//! a handle to a window. A headless surface renders to a texture.

use std::{mem::size_of, sync::Arc};

use wgpu::CompositeAlphaMode;

use crate::{
    render::{eventually::HasChanged, resource::texture::TextureView, settings::RendererSettings},
    window::{HeadedMapWindow, MapWindow, WindowSize},
};

pub struct BufferDimensions {
    pub width: usize,
    pub height: usize,
    pub unpadded_bytes_per_row: usize,
    pub padded_bytes_per_row: usize,
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
    pub fn resize_and_configure(&mut self, width: u32, height: u32, device: &wgpu::Device) {
        self.surface_config.height = width;
        self.surface_config.width = height;
        self.surface.configure(device, &self.surface_config);
    }
    pub fn configure(&self, device: &wgpu::Device) {
        self.surface.configure(device, &self.surface_config);
    }

    pub fn recreate_surface<MW>(&mut self, window: &MW, instance: &wgpu::Instance)
    where
        MW: MapWindow + HeadedMapWindow,
    {
        self.surface = unsafe { instance.create_surface(window.raw()) };
    }
    pub fn surface(&self) -> &wgpu::Surface {
        &self.surface
    }
}

pub struct BufferedTextureHead {
    pub texture: wgpu::Texture,
    pub output_buffer: wgpu::Buffer,
    pub buffer_dimensions: BufferDimensions,
}

#[cfg(feature = "headless")]
impl BufferedTextureHead {
    pub async fn create_png<'a>(
        &self,
        device: &wgpu::Device,
        png_output_path: &str,
        // device: &wgpu::Device,
    ) {
        use std::{fs::File, io::Write};
        // Note that we're not calling `.await` here.
        let buffer_slice = self.output_buffer.slice(..);
        buffer_slice.map_async(wgpu::MapMode::Read, |_| ());

        // Poll the device in a blocking manner so that our future resolves.
        // In an actual application, `device.poll(...)` should
        // be called in an event loop or on another thread.
        device.poll(wgpu::Maintain::Wait);
        let padded_buffer = buffer_slice.get_mapped_range();

        let mut png_encoder = png::Encoder::new(
            File::create(png_output_path).unwrap(), // TODO: Remove unwrap
            self.buffer_dimensions.width as u32,
            self.buffer_dimensions.height as u32,
        );
        png_encoder.set_depth(png::BitDepth::Eight);
        png_encoder.set_color(png::ColorType::Rgba);
        let mut png_writer = png_encoder
            .write_header()
            .unwrap() // TODO: Remove unwrap
            .into_stream_writer_with_size(self.buffer_dimensions.unpadded_bytes_per_row)
            .unwrap(); // TODO: Remove unwrap

        // from the padded_buffer we write just the unpadded bytes into the image
        for chunk in padded_buffer.chunks(self.buffer_dimensions.padded_bytes_per_row) {
            png_writer
                .write_all(&chunk[..self.buffer_dimensions.unpadded_bytes_per_row])
                .unwrap(); // TODO: Remove unwrap
        }
        png_writer.finish().unwrap(); // TODO: Remove unwrap

        // With the current interface, we have to make sure all mapped views are
        // dropped before we unmap the buffer.
        drop(padded_buffer);

        self.output_buffer.unmap();
    }
}

pub enum Head {
    Headed(WindowHead),
    Headless(Arc<BufferedTextureHead>),
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
        MW: MapWindow + HeadedMapWindow,
    {
        let size = window.size();
        let surface_config = wgpu::SurfaceConfiguration {
            alpha_mode: CompositeAlphaMode::Auto,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: settings.texture_format,
            width: size.width(),
            height: size.height(),
            present_mode: settings.present_mode,
        };

        let surface = unsafe { instance.create_surface(window.raw()) };

        Self {
            size,
            head: Head::Headed(WindowHead {
                surface,
                surface_config,
            }),
        }
    }

    // TODO: Give better name
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
            label: Some("BufferedTextureHead buffer"),
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
            head: Head::Headless(Arc::new(BufferedTextureHead {
                texture,
                output_buffer,
                buffer_dimensions,
            })),
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
                        log::warn!("surface outdated");
                        window.configure(device);
                        surface
                            .get_current_texture()
                            .expect("Error reconfiguring surface")
                    }
                    err => err.expect("Failed to acquire next swap chain texture!"),
                };
                frame.into()
            }
            Head::Headless(arc) => arc
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default())
                .into(),
        }
    }

    pub fn size(&self) -> WindowSize {
        self.size
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.size = WindowSize::new(width, height).expect("Invalid size for resizing the surface.");
    }

    pub fn reconfigure(&mut self, device: &wgpu::Device) {
        match &mut self.head {
            Head::Headed(window) => {
                if window.has_changed(&(self.size.width(), self.size.height())) {
                    window.resize_and_configure(self.size.height(), self.size.width(), device);
                }
            }
            Head::Headless(_) => {}
        }
    }

    pub fn recreate<MW>(&mut self, window: &MW, instance: &wgpu::Instance)
    where
        MW: MapWindow + HeadedMapWindow,
    {
        match &mut self.head {
            Head::Headed(window_head) => {
                if window_head.has_changed(&(self.size.width(), self.size.height())) {
                    window_head.recreate_surface(window, instance);
                }
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
        self.surface_config.width != criteria.0 || self.surface_config.height != criteria.1
    }
}
