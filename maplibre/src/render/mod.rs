//! This module implements the rendering algorithm of maplibre-rs. It manages the whole
//! communication with the GPU.

use crate::render::render_phase::RenderPhase;
use crate::render::resource::{BufferPool, Globals, IndexEntry};
use crate::render::resource::{Head, Surface};
use crate::render::resource::{Texture, TextureView};
use crate::render::settings::{RendererSettings, SurfaceType, WgpuSettings};
use crate::render::shaders::{ShaderFeatureStyle, ShaderLayerMetadata};
use crate::render::tile_view_pattern::{TileInView, TileShape, TileViewPattern};
use crate::render::util::Eventually;
use crate::tessellation::IndexDataType;
use crate::MapWindow;
use log::info;

// Rendering internals
mod graph;
mod graph_runner;
mod main_pass;
mod render_commands;
mod render_phase;
mod resource;
mod shaders;
mod stages;
mod tile_pipeline;
mod tile_view_pattern;
mod util;

// Public API
pub mod camera;
pub mod settings;

pub use shaders::ShaderVertex;
pub use stages::register_render_stages;

pub const INDEX_FORMAT: wgpu::IndexFormat = wgpu::IndexFormat::Uint32; // Must match IndexDataType

#[derive(Default)]
pub struct RenderState {
    render_target: Eventually<TextureView>,

    buffer_pool: Eventually<
        BufferPool<
            wgpu::Queue,
            wgpu::Buffer,
            ShaderVertex,
            IndexDataType,
            ShaderLayerMetadata,
            ShaderFeatureStyle,
        >,
    >,
    tile_view_pattern: Eventually<TileViewPattern<wgpu::Queue, wgpu::Buffer>>,

    tile_pipeline: Eventually<wgpu::RenderPipeline>,
    mask_pipeline: Eventually<wgpu::RenderPipeline>,

    globals_bind_group: Eventually<Globals>,

    depth_texture: Eventually<Texture>,
    multisampling_texture: Eventually<Option<Texture>>,

    mask_phase: RenderPhase<TileInView>,
    tile_phase: RenderPhase<(IndexEntry, TileShape)>,
}

pub struct Renderer {
    pub instance: wgpu::Instance,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub adapter_info: wgpu::AdapterInfo,

    pub wgpu_settings: WgpuSettings,
    pub settings: RendererSettings,

    pub state: RenderState,
    pub surface: Surface,
}

impl Renderer {
    /// Initializes the renderer by retrieving and preparing the GPU instance, device and queue
    /// for the specified backend.
    pub async fn initialize<MW>(
        window: &MW,
        wgpu_settings: WgpuSettings,
        settings: RendererSettings,
    ) -> Result<Self, wgpu::RequestDeviceError>
    where
        MW: MapWindow,
    {
        let instance = wgpu::Instance::new(wgpu_settings.backends.unwrap_or(wgpu::Backends::all()));

        let maybe_surface = match &settings.surface_type {
            SurfaceType::Headless => None,
            SurfaceType::Headed => Some(Surface::from_window(&instance, window, &settings)),
        };

        let compatible_surface = if let Some(surface) = &maybe_surface {
            match &surface.head() {
                Head::Headed(window_head) => Some(window_head.surface()),
                Head::Headless(_) => None,
            }
        } else {
            None
        };

        let (device, queue, adapter_info) = Self::request_device(
            &instance,
            &wgpu_settings,
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu_settings.power_preference,
                force_fallback_adapter: false,
                compatible_surface,
            },
        )
        .await?;

        let surface = maybe_surface.unwrap_or_else(|| match &settings.surface_type {
            SurfaceType::Headless => Surface::from_image(&device, window, &settings),
            SurfaceType::Headed => Surface::from_window(&instance, window, &settings),
        });

        match surface.head() {
            Head::Headed(window) => window.configure(&device),
            Head::Headless(_) => {}
        }

        Ok(Self {
            instance,
            device,
            queue,
            adapter_info,
            wgpu_settings,
            settings,
            state: Default::default(),
            surface,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.surface.resize(width, height)
    }

    /// Requests a device
    async fn request_device(
        instance: &wgpu::Instance,
        settings: &WgpuSettings,
        request_adapter_options: &wgpu::RequestAdapterOptions<'_>,
    ) -> Result<(wgpu::Device, wgpu::Queue, wgpu::AdapterInfo), wgpu::RequestDeviceError> {
        let adapter = instance
            .request_adapter(request_adapter_options)
            .await
            .expect("Unable to find a GPU! Make sure you have installed required drivers!");

        let adapter_info = adapter.get_info();
        info!("{:?}", adapter_info);

        #[cfg(not(target_arch = "wasm32"))]
        let trace_path = if settings.record_trace {
            let path = std::path::Path::new("wgpu_trace");
            // ignore potential error, wgpu will log it
            let _ = std::fs::create_dir(path);
            Some(path)
        } else {
            None
        };

        #[cfg(target_arch = "wasm32")]
        let trace_path = None;

        // Maybe get features and limits based on what is supported by the adapter/backend
        let mut features = wgpu::Features::empty();
        let mut limits = settings.limits.clone();

        features = adapter.features() | wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES;
        if adapter_info.device_type == wgpu::DeviceType::DiscreteGpu {
            // `MAPPABLE_PRIMARY_BUFFERS` can have a significant, negative performance impact for
            // discrete GPUs due to having to transfer data across the PCI-E bus and so it
            // should not be automatically enabled in this case. It is however beneficial for
            // integrated GPUs.
            features -= wgpu::Features::MAPPABLE_PRIMARY_BUFFERS;
        }
        limits = adapter.limits();

        // Enforce the disabled features
        if let Some(disabled_features) = settings.disabled_features {
            features -= disabled_features;
        }
        // NOTE: |= is used here to ensure that any explicitly-enabled features are respected.
        features |= settings.features;

        // Enforce the limit constraints
        if let Some(constrained_limits) = settings.constrained_limits.as_ref() {
            // NOTE: Respect the configured limits as an 'upper bound'. This means for 'max' limits, we
            // take the minimum of the calculated limits according to the adapter/backend and the
            // specified max_limits. For 'min' limits, take the maximum instead. This is intended to
            // err on the side of being conservative. We can't claim 'higher' limits that are supported
            // but we can constrain to 'lower' limits.
            limits = wgpu::Limits {
                max_texture_dimension_1d: limits
                    .max_texture_dimension_1d
                    .min(constrained_limits.max_texture_dimension_1d),
                max_texture_dimension_2d: limits
                    .max_texture_dimension_2d
                    .min(constrained_limits.max_texture_dimension_2d),
                max_texture_dimension_3d: limits
                    .max_texture_dimension_3d
                    .min(constrained_limits.max_texture_dimension_3d),
                max_texture_array_layers: limits
                    .max_texture_array_layers
                    .min(constrained_limits.max_texture_array_layers),
                max_bind_groups: limits
                    .max_bind_groups
                    .min(constrained_limits.max_bind_groups),
                max_dynamic_uniform_buffers_per_pipeline_layout: limits
                    .max_dynamic_uniform_buffers_per_pipeline_layout
                    .min(constrained_limits.max_dynamic_uniform_buffers_per_pipeline_layout),
                max_dynamic_storage_buffers_per_pipeline_layout: limits
                    .max_dynamic_storage_buffers_per_pipeline_layout
                    .min(constrained_limits.max_dynamic_storage_buffers_per_pipeline_layout),
                max_sampled_textures_per_shader_stage: limits
                    .max_sampled_textures_per_shader_stage
                    .min(constrained_limits.max_sampled_textures_per_shader_stage),
                max_samplers_per_shader_stage: limits
                    .max_samplers_per_shader_stage
                    .min(constrained_limits.max_samplers_per_shader_stage),
                max_storage_buffers_per_shader_stage: limits
                    .max_storage_buffers_per_shader_stage
                    .min(constrained_limits.max_storage_buffers_per_shader_stage),
                max_storage_textures_per_shader_stage: limits
                    .max_storage_textures_per_shader_stage
                    .min(constrained_limits.max_storage_textures_per_shader_stage),
                max_uniform_buffers_per_shader_stage: limits
                    .max_uniform_buffers_per_shader_stage
                    .min(constrained_limits.max_uniform_buffers_per_shader_stage),
                max_uniform_buffer_binding_size: limits
                    .max_uniform_buffer_binding_size
                    .min(constrained_limits.max_uniform_buffer_binding_size),
                max_storage_buffer_binding_size: limits
                    .max_storage_buffer_binding_size
                    .min(constrained_limits.max_storage_buffer_binding_size),
                max_vertex_buffers: limits
                    .max_vertex_buffers
                    .min(constrained_limits.max_vertex_buffers),
                max_vertex_attributes: limits
                    .max_vertex_attributes
                    .min(constrained_limits.max_vertex_attributes),
                max_vertex_buffer_array_stride: limits
                    .max_vertex_buffer_array_stride
                    .min(constrained_limits.max_vertex_buffer_array_stride),
                max_push_constant_size: limits
                    .max_push_constant_size
                    .min(constrained_limits.max_push_constant_size),
                min_uniform_buffer_offset_alignment: limits
                    .min_uniform_buffer_offset_alignment
                    .max(constrained_limits.min_uniform_buffer_offset_alignment),
                min_storage_buffer_offset_alignment: limits
                    .min_storage_buffer_offset_alignment
                    .max(constrained_limits.min_storage_buffer_offset_alignment),
                max_inter_stage_shader_components: limits
                    .max_inter_stage_shader_components
                    .min(constrained_limits.max_inter_stage_shader_components),
                max_compute_workgroup_storage_size: limits
                    .max_compute_workgroup_storage_size
                    .min(constrained_limits.max_compute_workgroup_storage_size),
                max_compute_invocations_per_workgroup: limits
                    .max_compute_invocations_per_workgroup
                    .min(constrained_limits.max_compute_invocations_per_workgroup),
                max_compute_workgroup_size_x: limits
                    .max_compute_workgroup_size_x
                    .min(constrained_limits.max_compute_workgroup_size_x),
                max_compute_workgroup_size_y: limits
                    .max_compute_workgroup_size_y
                    .min(constrained_limits.max_compute_workgroup_size_y),
                max_compute_workgroup_size_z: limits
                    .max_compute_workgroup_size_z
                    .min(constrained_limits.max_compute_workgroup_size_z),
                max_compute_workgroups_per_dimension: limits
                    .max_compute_workgroups_per_dimension
                    .min(constrained_limits.max_compute_workgroups_per_dimension),
            };
        }

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: settings.device_label.as_ref().map(|a| a.as_ref()),
                    features,
                    limits,
                },
                trace_path,
            )
            .await?;
        Ok((device, queue, adapter_info))
    }

    pub fn instance(&self) -> &wgpu::Instance {
        &self.instance
    }
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }
    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }
    pub fn state(&self) -> &RenderState {
        &self.state
    }
    pub fn surface(&self) -> &Surface {
        &self.surface
    }
}

#[cfg(test)]
mod tests {
    use crate::render::graph::RenderGraph;
    use crate::render::graph_runner::RenderGraphRunner;
    use crate::render::pass_pipeline::build_graph;
    use crate::render::World;

    #[tokio::test]
    async fn test_render() {
        let graph = build_graph();

        let instance = wgpu::Instance::new(wgpu::Backends::all());

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: Default::default(),
                force_fallback_adapter: false,
                compatible_surface: None,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::default(),
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .ok()
            .unwrap();

        RenderGraphRunner::run(&graph, &device, &queue, &World {});
    }
}
