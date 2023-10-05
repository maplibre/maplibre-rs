//! Utilities for handling surfaces which can be either headless or headed. A headed surface has
//! a handle to a window. A headless surface renders to a texture.

use std::{mem::size_of, sync::Arc};

use wgpu::TextureFormatFeatures;

use crate::{
    render::{
        error::RenderError,
        eventually::HasChanged,
        resource::texture::TextureView,
        settings::{Msaa, RendererSettings},
    },
    window::{HeadedMapWindow, MapWindow, WindowSize},
};

pub struct BufferDimensions {
    pub width: u32,
    pub height: u32,
    pub unpadded_bytes_per_row: u32,
    pub padded_bytes_per_row: u32,
}

impl BufferDimensions {
    fn new(size: WindowSize) -> Self {
        let bytes_per_pixel = size_of::<u32>() as u32;
        let unpadded_bytes_per_row = size.width() * bytes_per_pixel;

        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padded_bytes_per_row_padding = (align - unpadded_bytes_per_row % align) % align;
        let padded_bytes_per_row = unpadded_bytes_per_row + padded_bytes_per_row_padding;
        Self {
            width: size.width(),
            height: size.height(),
            unpadded_bytes_per_row,
            padded_bytes_per_row,
        }
    }
}

pub struct WindowHead {
    surface: wgpu::Surface,
    size: WindowSize,

    texture_format: wgpu::TextureFormat,
    present_mode: wgpu::PresentMode,
    texture_format_features: TextureFormatFeatures,
}

impl WindowHead {
    pub fn resize_and_configure(&mut self, width: u32, height: u32, device: &wgpu::Device) {
        self.size = WindowSize::new(width, height).unwrap();
        self.configure(device);
    }

    pub fn configure(&self, device: &wgpu::Device) {
        let surface_config = wgpu::SurfaceConfiguration {
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: self.texture_format,
            width: self.size.width(),
            height: self.size.height(),
            present_mode: self.present_mode,
            view_formats: vec![self.texture_format],
        };

        self.surface.configure(device, &surface_config);
    }

    pub fn recreate_surface<MW>(
        &mut self,
        window: &MW,
        instance: &wgpu::Instance,
    ) -> Result<(), RenderError>
    where
        MW: MapWindow + HeadedMapWindow,
    {
        self.surface = unsafe { instance.create_surface(window.raw())? };
        Ok(())
    }

    pub fn surface(&self) -> &wgpu::Surface {
        &self.surface
    }
}

pub struct BufferedTextureHead {
    texture: wgpu::Texture,
    texture_format: wgpu::TextureFormat,
    output_buffer: wgpu::Buffer,
    buffer_dimensions: BufferDimensions,
}

#[cfg(feature = "headless")]
#[derive(thiserror::Error, Debug)]
pub enum WriteImageError {
    #[error("error while rendering to image")]
    WriteImage(#[from] png::EncodingError),
    #[error("could not create file to save as an image")]
    CreateImageFileFailed(#[from] std::io::Error),
}

#[cfg(feature = "headless")]
impl BufferedTextureHead {
    pub fn map_async(&self, device: &wgpu::Device) -> wgpu::BufferSlice {
        // Note that we're not calling `.await` here.
        let buffer_slice = self.output_buffer.slice(..);
        buffer_slice.map_async(wgpu::MapMode::Read, |_| ());

        // Poll the device in a blocking manner so that our future resolves.
        // In an actual application, `device.poll(...)` should
        // be called in an event loop or on another thread.
        device.poll(wgpu::Maintain::Wait);
        buffer_slice
    }

    pub fn unmap(&self) {
        self.output_buffer.unmap();
    }

    pub fn write_png<'a>(
        &self,
        padded_buffer: &wgpu::BufferView<'a>,
        png_output_path: &str,
    ) -> Result<(), WriteImageError> {
        use std::{fs::File, io::Write};
        let mut png_encoder = png::Encoder::new(
            File::create(png_output_path)?,
            self.buffer_dimensions.width as u32,
            self.buffer_dimensions.height as u32,
        );
        png_encoder.set_depth(png::BitDepth::Eight);
        png_encoder.set_color(png::ColorType::Rgba);
        let mut png_writer = png_encoder
            .write_header()?
            .into_stream_writer_with_size(self.buffer_dimensions.unpadded_bytes_per_row as usize)?;

        // from the padded_buffer we write just the unpadded bytes into the image
        for chunk in padded_buffer.chunks(self.buffer_dimensions.padded_bytes_per_row as usize) {
            png_writer
                .write_all(&chunk[..self.buffer_dimensions.unpadded_bytes_per_row as usize])?
        }
        png_writer.finish()?;
        Ok(())
    }

    pub fn copy_texture(&self) -> wgpu::ImageCopyTexture<'_> {
        self.texture.as_image_copy()
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.output_buffer
    }

    pub fn bytes_per_row(&self) -> u32 {
        self.buffer_dimensions.padded_bytes_per_row
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
    pub fn from_surface<MW>(
        surface: wgpu::Surface,
        adapter: &wgpu::Adapter,
        window: &MW,
        settings: &RendererSettings,
    ) -> Self
    where
        MW: MapWindow + HeadedMapWindow,
    {
        let size = window.size();

        let capabilities = surface.get_capabilities(adapter);
        log::info!("adapter capabilities on surface: {capabilities:?}");

        let texture_format = settings
            .texture_format
            .or_else(|| capabilities.formats.first().cloned())
            .unwrap_or(wgpu::TextureFormat::Rgba8Unorm);
        log::info!("format description: {texture_format:?}");

        let texture_format_features = adapter.get_texture_format_features(texture_format);
        log::info!("format features: {texture_format_features:?}");

        Self {
            size,
            head: Head::Headed(WindowHead {
                surface,
                size,
                texture_format,
                texture_format_features,
                present_mode: settings.present_mode,
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
        let buffer_dimensions = BufferDimensions::new(size);

        // The output buffer lets us retrieve the data as an array
        let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("BufferedTextureHead buffer"),
            size: (buffer_dimensions.padded_bytes_per_row * buffer_dimensions.height) as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // TODO: Is this a sane default?
        let format = settings
            .texture_format
            .unwrap_or(wgpu::TextureFormat::Rgba8Unorm);

        let texture_descriptor = wgpu::TextureDescriptor {
            label: Some("Surface texture"),
            size: wgpu::Extent3d {
                width: size.width(),
                height: size.height(),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[format],
        };
        let texture = device.create_texture(&texture_descriptor);

        Self {
            size,
            head: Head::Headless(Arc::new(BufferedTextureHead {
                texture,
                texture_format: format,
                output_buffer,
                buffer_dimensions,
            })),
        }
    }

    pub fn surface_format(&self) -> wgpu::TextureFormat {
        match &self.head {
            Head::Headed(headed) => headed.texture_format,
            Head::Headless(headless) => headless.texture_format,
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
                    window.resize_and_configure(self.size.width(), self.size.height(), device);
                }
            }
            Head::Headless(_) => {}
        }
    }

    pub fn recreate<MW>(
        &mut self,
        window: &MW,
        instance: &wgpu::Instance,
    ) -> Result<(), RenderError>
    where
        MW: MapWindow + HeadedMapWindow,
    {
        match &mut self.head {
            Head::Headed(window_head) => {
                if window_head.has_changed(&(self.size.width(), self.size.height())) {
                    window_head.recreate_surface(window, instance)?;
                }
            }
            Head::Headless(_) => {}
        }
        Ok(())
    }

    pub fn head(&self) -> &Head {
        &self.head
    }

    pub fn head_mut(&mut self) -> &mut Head {
        &mut self.head
    }

    pub fn is_multisampling_supported(&self, msaa: Msaa) -> bool {
        match &self.head {
            Head::Headed(headed) => {
                let max_sample_count = {
                    let flags = headed.texture_format_features.flags;
                    if flags.contains(wgpu::TextureFormatFeatureFlags::MULTISAMPLE_X8) {
                        8
                    } else if flags.contains(wgpu::TextureFormatFeatureFlags::MULTISAMPLE_X4) {
                        4
                    } else if flags.contains(wgpu::TextureFormatFeatureFlags::MULTISAMPLE_X2) {
                        2
                    } else {
                        1
                    }
                };
                let is_supported = msaa.samples <= max_sample_count;
                if !is_supported {
                    log::debug!("Multisampling is not supported on surface");
                }
                is_supported
            }
            Head::Headless(_) => false, // TODO: support multisampling on headless
        }
    }
}

impl HasChanged for WindowHead {
    /// Tuple of width and height
    type Criteria = (u32, u32);

    fn has_changed(&self, criteria: &Self::Criteria) -> bool {
        self.size.width() != criteria.0 || self.size.height() != criteria.1
    }
}
