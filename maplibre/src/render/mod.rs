//! This module implements the rendering algorithm of maplibre-rs. It manages the whole
//! communication with the GPU.
//!
//! The render in this module is largely based on the
//! [bevy_render](https://github.com/bevyengine/bevy/tree/aced6a/crates/bevy_render)
//! crate with commit `aced6a`.
//! It is dual-licensed under MIT and Apache:
//!
//! ```text
//! Bevy is dual-licensed under either
//!
//! * MIT License (docs/LICENSE-MIT or http://opensource.org/licenses/MIT)
//! * Apache License, Version 2.0 (docs/LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
//!
//! at your option.
//! ```
//!
//! We appreciate the design and implementation work which as gone into it.
//!

use std::{ops::Deref, rc::Rc, sync::Arc};

use crate::{
    environment::Environment,
    kernel::Kernel,
    plugin::Plugin,
    render::{
        error::RenderError,
        eventually::Eventually,
        graph::{EmptyNode, RenderGraph},
        main_pass::{MainPassDriverNode, MainPassNode},
        resource::{Head, Surface, Texture, TextureView},
        settings::{RendererSettings, WgpuSettings},
        systems::{
            cleanup_system::cleanup_system, resource_system::ResourceSystem,
            sort_phase_system::sort_phase_system,
            tile_view_pattern_system::tile_view_pattern_system,
        },
    },
    schedule::{Schedule, StageLabel},
    tcs::{
        system::{stage::SystemStage, SystemContainer},
        world::World,
    },
    window::{HeadedMapWindow, MapWindow},
};

pub mod graph;
pub mod resource;
mod systems;

// Rendering internals
mod graph_runner;
mod main_pass;
pub mod shaders; // TODO: Make private

// Public API
pub mod builder;
pub mod camera;
pub mod error;
pub mod eventually;
pub mod render_commands;
pub mod render_phase;
pub mod settings;
pub mod tile_view_pattern;

pub use shaders::ShaderVertex;

use crate::render::{
    render_phase::{LayerItem, RenderPhase, TileMaskItem},
    systems::{graph_runner_system::GraphRunnerSystem, upload_system::upload_system},
    tile_view_pattern::{ViewTileSources, WgpuTileViewPattern},
};

pub(crate) const INDEX_FORMAT: wgpu::IndexFormat = wgpu::IndexFormat::Uint32; // Must match IndexDataType

/// The labels of the default App rendering stages.
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum RenderStageLabel {
    /// Extract data from the world.
    Extract,

    /// Prepare render resources from the extracted data for the GPU.
    /// For example during this phase textures are created, buffers are allocated and written.
    Prepare,

    /// Queues [PhaseItems](render_phase::PhaseItem) that depend on
    /// [`Prepare`](RenderStageLabel::Prepare) data and queue up draw calls to run during the
    /// [`Render`](RenderStageLabel::Render) stage.
    /// For example data is uploaded to the GPU in this stage.
    Queue,

    /// Sort the [`RenderPhases`](crate::render_phase::RenderPhase) here.
    PhaseSort,

    /// Actual rendering happens here.
    /// In most cases, only the render backend should insert resources here.
    Render,

    /// Cleanup render resources here.
    Cleanup,
}

impl StageLabel for RenderStageLabel {
    fn dyn_clone(&self) -> Box<dyn StageLabel> {
        Box::new(self.clone())
    }
}

pub struct RenderResources {
    pub surface: Surface,
    pub render_target: Eventually<TextureView>,
    pub depth_texture: Eventually<Texture>,
    pub multisampling_texture: Eventually<Option<Texture>>,
}

impl RenderResources {
    pub fn new(surface: Surface) -> Self {
        Self {
            render_target: Default::default(),
            depth_texture: Default::default(),
            multisampling_texture: Default::default(),
            surface,
        }
    }

    pub fn recreate_surface<MW>(
        &mut self,
        window: &MW,
        instance: &wgpu::Instance,
    ) -> Result<(), RenderError>
    where
        MW: MapWindow + HeadedMapWindow,
    {
        self.surface.recreate::<MW>(window, instance)
    }

    pub fn surface(&self) -> &Surface {
        &self.surface
    }
}

pub struct Renderer {
    pub instance: wgpu::Instance,
    pub device: Arc<wgpu::Device>, // TODO: Arc is needed for headless rendering. Is there a simpler solution?
    pub queue: wgpu::Queue,
    pub adapter: wgpu::Adapter,

    pub wgpu_settings: WgpuSettings,
    pub settings: RendererSettings,

    pub resources: RenderResources,
    pub render_graph: RenderGraph,
}

impl Renderer {
    /// Initializes the renderer by retrieving and preparing the GPU instance, device and queue
    /// for the specified backend.
    pub async fn initialize<MW>(
        window: &MW,
        wgpu_settings: WgpuSettings,
        settings: RendererSettings,
    ) -> Result<Self, RenderError>
    where
        MW: MapWindow + HeadedMapWindow,
    {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu_settings.backends.unwrap_or(wgpu::Backends::all()),
            dx12_shader_compiler: Default::default(),
        });

        let surface: wgpu::Surface = unsafe { instance.create_surface(window.raw())? };

        let (adapter, device, queue) = Self::request_device(
            &instance,
            &wgpu_settings,
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu_settings.power_preference,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            },
        )
        .await?;

        let surface = Surface::from_surface(surface, &adapter, window, &settings);

        match surface.head() {
            Head::Headed(window) => window.configure(&device),
            Head::Headless(_) => {}
        }

        Ok(Self {
            instance,
            device: Arc::new(device),
            queue,
            adapter,
            wgpu_settings,
            settings,
            resources: RenderResources::new(surface),
            render_graph: Default::default(),
        })
    }

    pub async fn initialize_headless<MW>(
        window: &MW,
        wgpu_settings: WgpuSettings,
        settings: RendererSettings,
    ) -> Result<Self, RenderError>
    where
        MW: MapWindow,
    {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu_settings.backends.unwrap_or(wgpu::Backends::all()),
            dx12_shader_compiler: Default::default(),
        });

        let (adapter, device, queue) = Self::request_device(
            &instance,
            &wgpu_settings,
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu_settings.power_preference,
                force_fallback_adapter: false,
                compatible_surface: None,
            },
        )
        .await?;

        let surface = Surface::from_image(&device, window, &settings);

        Ok(Self {
            instance,
            device: Arc::new(device),
            queue,
            adapter,
            wgpu_settings,
            settings,
            resources: RenderResources::new(surface),
            render_graph: Default::default(),
        })
    }

    pub fn resize_surface(&mut self, width: u32, height: u32) {
        self.resources.surface.resize(width, height)
    }

    /// Requests a device
    async fn request_device(
        instance: &wgpu::Instance,
        settings: &WgpuSettings,
        request_adapter_options: &wgpu::RequestAdapterOptions<'_>,
    ) -> Result<(wgpu::Adapter, wgpu::Device, wgpu::Queue), wgpu::RequestDeviceError> {
        let adapter = instance
            .request_adapter(request_adapter_options)
            .await
            .ok_or(wgpu::RequestDeviceError)?;

        let adapter_info = adapter.get_info();

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

        let mut features =
            adapter.features() | wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES;
        if adapter_info.device_type == wgpu::DeviceType::DiscreteGpu {
            // `MAPPABLE_PRIMARY_BUFFERS` can have a significant, negative performance impact for
            // discrete GPUs due to having to transfer data across the PCI-E bus and so it
            // should not be automatically enabled in this case. It is however beneficial for
            // integrated GPUs.
            features -= wgpu::Features::MAPPABLE_PRIMARY_BUFFERS;
        }
        let mut limits = adapter.limits();

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
                max_bindings_per_bind_group: limits
                    .max_bindings_per_bind_group
                    .min(constrained_limits.max_bindings_per_bind_group),
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
                max_buffer_size: limits
                    .max_buffer_size
                    .min(constrained_limits.max_buffer_size),
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
        Ok((adapter, device, queue))
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
    pub fn state(&self) -> &RenderResources {
        &self.resources
    }
    pub fn surface(&self) -> &Surface {
        &self.resources.surface
    }
}

#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod tests {
    use crate::{
        tcs::world::World,
        window::{MapWindow, MapWindowConfig, WindowSize},
    };

    pub struct HeadlessMapWindowConfig {
        size: WindowSize,
    }

    impl MapWindowConfig for HeadlessMapWindowConfig {
        type MapWindow = HeadlessMapWindow;

        fn create(&self) -> Self::MapWindow {
            Self::MapWindow { size: self.size }
        }
    }

    pub struct HeadlessMapWindow {
        size: WindowSize,
    }

    impl MapWindow for HeadlessMapWindow {
        fn size(&self) -> WindowSize {
            self.size
        }
    }

    #[tokio::test]
    async fn test_render() {
        use log::LevelFilter;

        use crate::render::{
            graph::RenderGraph, graph_runner::RenderGraphRunner, resource::Surface,
            RenderResources, RendererSettings,
        };

        let _ = env_logger::builder()
            .filter_level(LevelFilter::Trace)
            .is_test(true)
            .try_init();
        let graph = RenderGraph::default();

        let backends = wgpu::util::backend_bits_from_env().unwrap_or(wgpu::Backends::all());
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends,
            dx12_shader_compiler: Default::default(),
        });
        let adapter = wgpu::util::initialize_adapter_from_env_or_default(&instance, backends, None)
            .await
            .expect("Unable to initialize adapter");

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
            .expect("Unable to request device");

        let render_state = RenderResources::new(Surface::from_image(
            &device,
            &HeadlessMapWindow {
                size: WindowSize::new(100, 100).expect("invalid headless map size"),
            },
            &RendererSettings::default(),
        ));

        let world = World::default();
        RenderGraphRunner::run(&graph, &device, &queue, &render_state, &world)
            .expect("failed to run graph runner");
    }
}

// Contributors to the RenderGraph should use the following label conventions:
// 1. Graph modules should have a NAME, input module, and node module (where relevant)
// 2. The "main_graph" graph is the root.
// 3. "sub graph" modules should be nested beneath their parent graph module
pub mod main_graph {
    // Labels for input nodes
    pub mod input {}
    // Labels for non-input nodes
    pub mod node {
        pub const MAIN_PASS_DEPENDENCIES: &str = "main_pass_dependencies";
        pub const MAIN_PASS_DRIVER: &str = "main_pass_driver";
    }
}

/// Labels for the "draw" graph
mod draw_graph {
    pub const NAME: &str = "draw";
    // Labels for input nodes
    pub mod input {}
    // Labels for non-input nodes
    pub mod node {
        pub const MAIN_PASS: &str = "main_pass";
    }
}

pub struct MaskPipeline(pub wgpu::RenderPipeline);
impl Deref for MaskPipeline {
    type Target = wgpu::RenderPipeline;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// TODO: Do we really want a render plugin or do we want to statically do this setup?
#[derive(Default)]
pub struct RenderPlugin;

impl<E: Environment> Plugin<E> for RenderPlugin {
    fn build(
        &self,
        schedule: &mut Schedule,
        _kernel: Rc<Kernel<E>>,
        world: &mut World,
        graph: &mut RenderGraph,
    ) {
        let resources = &mut world.resources;

        let mut draw_graph = RenderGraph::default();
        // Draw nodes
        draw_graph.add_node(draw_graph::node::MAIN_PASS, MainPassNode::new());
        // Input node
        let input_node_id = draw_graph.set_input(vec![]);
        // Edges
        draw_graph
            .add_node_edge(input_node_id, draw_graph::node::MAIN_PASS)
            .expect("main pass or draw node does not exist");

        graph.add_sub_graph(draw_graph::NAME, draw_graph);
        graph.add_node(main_graph::node::MAIN_PASS_DEPENDENCIES, EmptyNode);
        graph.add_node(main_graph::node::MAIN_PASS_DRIVER, MainPassDriverNode);
        graph
            .add_node_edge(
                main_graph::node::MAIN_PASS_DEPENDENCIES,
                main_graph::node::MAIN_PASS_DRIVER,
            )
            .expect("main pass driver or dependencies do not exist");

        // render graph dependency
        resources.init::<RenderPhase<LayerItem>>();
        resources.init::<RenderPhase<TileMaskItem>>();
        // tile_view_pattern:
        resources.insert(Eventually::<WgpuTileViewPattern>::Uninitialized);
        resources.init::<ViewTileSources>();
        // masks
        resources.insert(Eventually::<MaskPipeline>::Uninitialized);

        schedule.add_stage(RenderStageLabel::Extract, SystemStage::default());
        schedule.add_stage(
            RenderStageLabel::Prepare,
            SystemStage::default().with_system(SystemContainer::new(ResourceSystem)),
        );
        schedule.add_stage(
            RenderStageLabel::Queue,
            SystemStage::default()
                .with_system(tile_view_pattern_system)
                .with_system(upload_system),
        );
        schedule.add_stage(
            RenderStageLabel::PhaseSort,
            SystemStage::default().with_system(sort_phase_system),
        );
        schedule.add_stage(
            RenderStageLabel::Render,
            SystemStage::default().with_system(SystemContainer::new(GraphRunnerSystem)),
        );
        schedule.add_stage(
            RenderStageLabel::Cleanup,
            SystemStage::default().with_system(cleanup_system),
        );
    }
}
