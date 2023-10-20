//! Settings for the renderer

use std::borrow::Cow;

use wgpu::PresentMode;
pub use wgpu::{Backends, Features, Limits, PowerPreference, TextureFormat};

/// Provides configuration for renderer initialization. Use [`Device::features`](crate::renderer::Device::features),
/// [`Device::limits`](crate::renderer::Device::limits), and the [`WgpuAdapterInfo`](crate::render_resource::WgpuAdapterInfo)
/// resource to get runtime information about the actual adapter, backend, features, and limits.
#[derive(Clone)]
pub struct WgpuSettings {
    pub device_label: Option<Cow<'static, str>>,
    pub backends: Option<Backends>,
    pub power_preference: PowerPreference,
    /// The features to ensure are enabled regardless of what the adapter/backend supports.
    /// Setting these explicitly may cause renderer initialization to fail.
    pub features: Features,
    /// The features to ensure are disabled regardless of what the adapter/backend supports
    pub disabled_features: Option<Features>,
    /// The imposed limits.
    pub limits: Limits,
    /// The constraints on limits allowed regardless of what the adapter/backend supports
    pub constrained_limits: Option<Limits>,

    /// Whether a trace is recorded an stored in the current working directory
    pub record_trace: bool,
}

impl Default for WgpuSettings {
    fn default() -> Self {
        let backends = Some(wgpu::util::backend_bits_from_env().unwrap_or(Backends::all()));

        let limits = if cfg!(feature = "web-webgl") {
            Limits {
                max_texture_dimension_2d: 4096,
                ..Limits::downlevel_webgl2_defaults()
            }
        } else if cfg!(target_os = "android") {
            Limits {
                max_storage_textures_per_shader_stage: 4,
                max_compute_workgroups_per_dimension: 0,
                max_compute_workgroup_size_z: 0,
                max_compute_workgroup_size_y: 0,
                max_compute_workgroup_size_x: 0,
                max_compute_workgroup_storage_size: 0,
                max_compute_invocations_per_workgroup: 0,
                ..Limits::downlevel_defaults()
            }
        } else {
            Limits {
                ..Limits::default()
            }
        };

        let features = Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES;

        Self {
            device_label: Default::default(),
            backends,
            power_preference: PowerPreference::HighPerformance,
            features,
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
    pub fn is_multisampling(&self) -> bool {
        self.samples > 1
    }
}

impl Default for Msaa {
    fn default() -> Self {
        // By default we are trying to multisample
        Self { samples: 4 }
    }
}

#[derive(Clone, Copy)]
pub struct RendererSettings {
    pub msaa: Msaa,
    /// Explicitly set a texture format or let the renderer automatically choose one
    pub texture_format: Option<TextureFormat>,
    pub depth_texture_format: TextureFormat,
    /// Present mode for surfaces if a surface is used.
    pub present_mode: PresentMode,
}

impl Default for RendererSettings {
    fn default() -> Self {
        Self {
            msaa: Msaa::default(),
            texture_format: None,

            depth_texture_format: TextureFormat::Depth24PlusStencil8,
            present_mode: PresentMode::AutoVsync,
        }
    }
}
