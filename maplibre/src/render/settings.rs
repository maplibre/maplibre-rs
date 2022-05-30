//! Settings for the renderer

use crate::platform::COLOR_TEXTURE_FORMAT;
use std::borrow::Cow;

pub use wgpu::Backends;

/// Provides configuration for renderer initialization. Use [`Device::features`](crate::renderer::Device::features),
/// [`Device::limits`](crate::renderer::Device::limits), and the [`WgpuAdapterInfo`](crate::render_resource::WgpuAdapterInfo)
/// resource to get runtime information about the actual adapter, backend, features, and limits.
#[derive(Clone)]
pub struct WgpuSettings {
    pub device_label: Option<Cow<'static, str>>,
    pub backends: Option<wgpu::Backends>,
    pub power_preference: wgpu::PowerPreference,
    /// The features to ensure are enabled regardless of what the adapter/backend supports.
    /// Setting these explicitly may cause renderer initialization to fail.
    pub features: wgpu::Features,
    /// The features to ensure are disabled regardless of what the adapter/backend supports
    pub disabled_features: Option<wgpu::Features>,
    /// The imposed limits.
    pub limits: wgpu::Limits,
    /// The constraints on limits allowed regardless of what the adapter/backend supports
    pub constrained_limits: Option<wgpu::Limits>,

    /// Whether a trace is recorded an stored in the current working directory
    pub record_trace: bool,
}

impl Default for WgpuSettings {
    fn default() -> Self {
        let backends = Some(wgpu::util::backend_bits_from_env().unwrap_or(wgpu::Backends::all()));

        let limits = if cfg!(feature = "web-webgl") {
            wgpu::Limits {
                max_texture_dimension_2d: 4096,
                ..wgpu::Limits::downlevel_webgl2_defaults()
            }
        } else if cfg!(target_os = "android") {
            wgpu::Limits {
                max_storage_textures_per_shader_stage: 4,
                max_compute_workgroups_per_dimension: 0,
                max_compute_workgroup_size_z: 0,
                max_compute_workgroup_size_y: 0,
                max_compute_workgroup_size_x: 0,
                max_compute_workgroup_storage_size: 0,
                max_compute_invocations_per_workgroup: 0,
                ..wgpu::Limits::downlevel_defaults()
            }
        } else {
            wgpu::Limits {
                ..wgpu::Limits::default()
            }
        };

        Self {
            device_label: Default::default(),
            backends,
            power_preference: wgpu::PowerPreference::HighPerformance,
            features: wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
            disabled_features: None,
            limits,
            constrained_limits: None,
            record_trace: false,
        }
    }
}

#[derive(Clone)]
pub enum SurfaceType {
    Headless,
    Headed,
}

#[derive(Copy, Clone)]
/// Configuration resource for [Multi-Sample Anti-Aliasing](https://en.wikipedia.org/wiki/Multisample_anti-aliasing).
///
pub struct Msaa {
    /// The number of samples to run for Multi-Sample Anti-Aliasing. Higher numbers result in
    /// smoother edges.
    /// Defaults to 4.
    ///
    /// Note that WGPU currently only supports 1 or 4 samples.
    /// Ultimately we plan on supporting whatever is natively supported on a given device.
    /// Check out this issue for more info: <https://github.com/gfx-rs/wgpu/issues/1832>
    pub samples: u32,
}

impl Msaa {
    pub fn is_active(&self) -> bool {
        self.samples > 1
    }
}

impl Default for Msaa {
    fn default() -> Self {
        Self { samples: 4 }
    }
}

#[derive(Clone)]
pub struct RendererSettings {
    pub msaa: Msaa,
    pub texture_format: wgpu::TextureFormat,
}

impl Default for RendererSettings {
    fn default() -> Self {
        Self {
            msaa: Msaa::default(),
            texture_format: COLOR_TEXTURE_FORMAT,
        }
    }
}
