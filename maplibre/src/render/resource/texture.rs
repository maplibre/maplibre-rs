//! Utility for a texture view which can either be created by a [`TextureView`](wgpu::TextureView)
//! or [`SurfaceTexture`](wgpu::SurfaceTexture)

use std::ops::Deref;

use crate::render::{eventually::HasChanged, settings::Msaa};

/// Describes a [`TextureView`].
///
/// May be converted from a [`TextureView`](wgpu::TextureView) or [`SurfaceTexture`](wgpu::SurfaceTexture)
/// or dereferences to a wgpu [`TextureView`](wgpu::TextureView).
#[derive(Debug)]
pub enum TextureView {
    /// The value is an actual wgpu [`TextureView`](wgpu::TextureView).
    TextureView(wgpu::TextureView),

    /// The value is a wgpu [`SurfaceTexture`](wgpu::SurfaceTexture), but dereferences to
    /// a [`TextureView`](wgpu::TextureView).
    SurfaceTexture {
        // NOTE: The order of these fields is important because the view must be dropped before the
        // frame is dropped
        view: wgpu::TextureView,
        texture: wgpu::SurfaceTexture,
    },
}

impl TextureView {
    /// Returns the [`SurfaceTexture`](wgpu::SurfaceTexture) of the texture view if it is of that type.
    #[inline]
    pub fn take_surface_texture(self) -> Option<wgpu::SurfaceTexture> {
        match self {
            TextureView::TextureView(_) => None,
            TextureView::SurfaceTexture { texture, .. } => Some(texture),
        }
    }
}

impl From<wgpu::TextureView> for TextureView {
    fn from(value: wgpu::TextureView) -> Self {
        TextureView::TextureView(value)
    }
}

impl From<wgpu::SurfaceTexture> for TextureView {
    fn from(surface_texture: wgpu::SurfaceTexture) -> Self {
        let view = surface_texture.texture.create_view(&Default::default());

        TextureView::SurfaceTexture {
            texture: surface_texture,
            view,
        }
    }
}

impl Deref for TextureView {
    type Target = wgpu::TextureView;

    #[inline]
    fn deref(&self) -> &Self::Target {
        match &self {
            TextureView::TextureView(value) => value,
            TextureView::SurfaceTexture { view, .. } => view,
        }
    }
}

pub struct Texture {
    pub size: wgpu::Extent3d,
    pub texture: wgpu::Texture,
    pub view: TextureView,
}

impl Texture {
    pub fn new(
        label: wgpu::Label,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        width: u32,
        height: u32,
        msaa: Msaa,
        usage: wgpu::TextureUsages,
    ) -> Texture {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: msaa.samples,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage,
            view_formats: &[format],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        Self {
            size,
            texture,
            view: TextureView::TextureView(view),
        }
    }
}

impl HasChanged for Texture {
    type Criteria = (u32, u32);

    fn has_changed(&self, criteria: &Self::Criteria) -> bool {
        let size = (self.size.width, self.size.height);
        !size.eq(criteria)
    }
}
