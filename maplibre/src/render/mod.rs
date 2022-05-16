//! This module implements the rendering algorithm of maplibre-rs. It manages the whole
//! communication with the GPU.

use crate::render::buffer_pool::{BackingBufferDescriptor, BufferPool};
use crate::render::graph::{EmptyNode, RenderGraph};
use crate::render::graph_runner::RenderGraphRunner;
use crate::render::main_pass::{draw_graph, node, MainPassDriverNode, MainPassNode};
use crate::render::resource::surface::{Head, Surface};
use crate::render::resource::texture::{Texture, TextureView};
use crate::render::settings::{RendererSettings, SurfaceType, WgpuSettings};
use crate::render::shaders::{Shader, ShaderFeatureStyle, ShaderLayerMetadata, ShaderTileMetadata};
use crate::render::stage::{RenderStage, Schedule, Stage};
use crate::render::tile_pipeline::TilePipeline;
use crate::render::tile_view_pattern::TileViewPattern;
use crate::render::Eventually::{Initialized, Uninitialized};
use crate::tessellation::IndexDataType;
use crate::MapWindow;
use log::{error, info};
use std::mem::size_of;
use tokio::io::AsyncReadExt;

mod buffer_pool;
mod shaders;
mod tile_view_pattern;

pub mod camera;
pub mod graph;
pub mod graph_runner;
pub mod main_pass;
pub mod render_phase;
//pub mod render_state;
mod render_commands;
pub mod resource;
pub mod settings;
pub mod stage;

pub mod tile_pipeline;
pub mod util;

use crate::render::resource::pipeline::RenderPipeline;
pub use shaders::ShaderVertex;

pub trait FromDevice {
    fn from_device(device: &wgpu::Device) -> Self;
}

pub const INDEX_FORMAT: wgpu::IndexFormat = wgpu::IndexFormat::Uint32; // Must match IndexDataType

pub enum Eventually<T> {
    Initialized(T),
    Uninitialized,
}

impl<T> Eventually<T> {
    pub fn take(mut self) -> Option<T> {
        match self {
            Initialized(value) => {
                self = Uninitialized;
                Some(value)
            }
            Uninitialized => None,
        }
    }
}

impl<T> Default for Eventually<T> {
    fn default() -> Self {
        Uninitialized
    }
}

#[derive(Default)]
pub struct RenderState {
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

    render_target: Eventually<TextureView>,

    depth_texture: Eventually<Texture>,
    multisampling_texture: Eventually<Option<Texture>>,
}

pub const TILE_VIEW_SIZE: wgpu::BufferAddress = 4096;

struct ResourceStage;

impl Stage for ResourceStage {
    fn run(
        &mut self,
        Renderer {
            settings,
            device,
            surface,
            ..
        }: &Renderer,
        state: &mut RenderState,
    ) {
        state.render_target = Initialized(surface.create_view().unwrap());

        state.depth_texture = Initialized(Texture::new(
            device,
            wgpu::TextureFormat::Depth24PlusStencil8,
            100,
            100,
            settings.sample_count,
        ));

        state.multisampling_texture = Initialized(if settings.sample_count > 1 {
            Some(Texture::new(
                &device,
                settings.texture_format,
                100,
                100,
                settings.sample_count,
            ))
        } else {
            None
        });

        state.buffer_pool = Initialized(BufferPool::from_device(device));

        let tile_view_buffer_desc = wgpu::BufferDescriptor {
            label: None,
            size: size_of::<ShaderTileMetadata> as wgpu::BufferAddress * TILE_VIEW_SIZE,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        };

        state.tile_view_pattern = Initialized(TileViewPattern::new(BackingBufferDescriptor::new(
            device.create_buffer(&tile_view_buffer_desc),
            tile_view_buffer_desc.size,
        )));

        let tile_shader = shaders::TileShader {
            format: settings.texture_format,
        };
        let mask_shader = shaders::TileShader {
            format: settings.texture_format,
        };

        state.tile_pipeline = Initialized(
            TilePipeline::new(
                settings.sample_count,
                tile_shader.describe_vertex(),
                tile_shader.describe_fragment(),
            )
            .describe_render_pipeline()
            .initialize(device),
        );

        state.mask_pipeline = Initialized(
            TilePipeline::new(
                settings.sample_count,
                mask_shader.describe_vertex(),
                mask_shader.describe_fragment(),
            )
            .describe_render_pipeline()
            .initialize(device),
        );
    }
}

/// Updates the [`RenderGraph`] with all of its nodes and then runs it to render the entire frame.

struct GraphRunnerStage {
    graph: RenderGraph,
}

impl Default for GraphRunnerStage {
    fn default() -> Self {
        let pass_node = MainPassNode::new();
        let mut graph = RenderGraph::default();

        let mut draw_graph = RenderGraph::default();
        draw_graph.add_node(draw_graph::node::MAIN_PASS, pass_node);
        let input_node_id = draw_graph.set_input(vec![]);
        draw_graph
            .add_node_edge(input_node_id, draw_graph::node::MAIN_PASS)
            .unwrap();
        graph.add_sub_graph(draw_graph::NAME, draw_graph);

        graph.add_node(node::MAIN_PASS_DEPENDENCIES, EmptyNode);
        graph.add_node(node::MAIN_PASS_DRIVER, MainPassDriverNode);
        graph
            .add_node_edge(node::MAIN_PASS_DEPENDENCIES, node::MAIN_PASS_DRIVER)
            .unwrap();
        Self { graph }
    }
}

impl Stage for GraphRunnerStage {
    fn run(&mut self, renderer: &Renderer, state: &mut RenderState) {
        self.graph.update(state);

        if let Err(e) =
            RenderGraphRunner::run(&self.graph, &renderer.device, &renderer.queue, state)
        {
            error!("Error running render graph:");
            {
                let mut src: &dyn std::error::Error = &e;
                loop {
                    error!("> {}", src);
                    match src.source() {
                        Some(s) => src = s,
                        None => break,
                    }
                }
            }

            panic!("Error running render graph: {:?}", e);
        }

        {
            let _span = tracing::info_span!("present_frames").entered();

            if let Some(render_target) = state.render_target.take() {
                if let Some(surface_texture) = render_target.take_surface_texture() {
                    surface_texture.present();
                }

                #[cfg(feature = "tracing-tracy")]
                tracing::event!(
                    tracing::Level::INFO,
                    message = "finished frame",
                    tracy.frame_mark = true
                );
            }
        }
    }
}

pub struct Renderer {
    instance: wgpu::Instance,
    device: wgpu::Device,
    queue: wgpu::Queue,
    adapter_info: wgpu::AdapterInfo,

    wgpu_settings: WgpuSettings,
    settings: RendererSettings,

    pub state: RenderState,
    surface: Surface,
}

impl Renderer {
    pub fn register_in_schedule(&self, schedule: &mut Schedule) {
        schedule.add_stage(RenderStage::Prepare, ResourceStage);
        schedule.add_stage(RenderStage::Render, GraphRunnerStage::default());
    }

    /// Initializes the renderer by retrieving and preparing the GPU instance, device and queue
    /// for the specified backend.
    pub async fn initialize<MW>(window: &MW) -> Result<Self, wgpu::RequestDeviceError>
    where
        MW: MapWindow,
    {
        let wgpu_settings = WgpuSettings::default(); // FIXME: make configurable
        let settings = RendererSettings::default();

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
