use cgmath::{Matrix4, Vector4};
use std::collections::HashSet;
use std::default::Default;
use std::fmt::Formatter;
use std::io::sink;
use std::time::{Instant, SystemTime};
use std::{cmp, fmt, iter};

use crate::coords::{ViewRegion, TILE_SIZE};

use crate::io::scheduler::IOScheduler;
use crate::io::LayerTessellateResult;
use style_spec::layer::{LayerPaint, StyleLayer};
use style_spec::{EncodedSrgb, Style};
use wgpu::{Buffer, Limits, Queue};
use winit::dpi::PhysicalSize;
use winit::window::Window;

use crate::platform::{COLOR_TEXTURE_FORMAT, MIN_BUFFER_SIZE};
use crate::render::buffer_pool::{BackingBufferDescriptor, BufferPool, IndexEntry};
use crate::render::camera;
use crate::render::camera::ViewProjection;
use crate::render::options::{
    DEBUG_WIREFRAME, FEATURE_METADATA_BUFFER_SIZE, INDEX_FORMAT, INDICES_BUFFER_SIZE,
    TILE_META_COUNT, VERTEX_BUFFER_SIZE,
};
use crate::render::tile_mask_pattern::TileMaskPattern;
use crate::tessellation::{IndexDataType, OverAlignedVertexBuffer};

use crate::util::FPSMeter;

use super::piplines::*;
use super::shaders;
use super::shaders::*;
use super::texture::Texture;

use tracing;

pub struct RenderState {
    instance: wgpu::Instance,

    device: wgpu::Device,
    queue: wgpu::Queue,

    fps_meter: FPSMeter,

    surface: wgpu::Surface,
    surface_config: wgpu::SurfaceConfiguration,
    suspended: bool,

    pub size: winit::dpi::PhysicalSize<u32>,

    render_pipeline: wgpu::RenderPipeline,
    mask_pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,

    sample_count: u32,
    multisampling_texture: Option<Texture>,

    depth_texture: Texture,

    globals_uniform_buffer: wgpu::Buffer,

    buffer_pool: BufferPool<
        Queue,
        Buffer,
        ShaderVertex,
        IndexDataType,
        ShaderLayerMetadata,
        ShaderFeatureStyle,
    >,

    tile_mask_pattern: TileMaskPattern,

    pub camera: camera::Camera,
    pub perspective: camera::Perspective,
    pub zoom: f64,

    style: Box<Style>,
}

impl RenderState {
    pub async fn new(window: &Window, style: Box<Style>) -> Self {
        let sample_count = 4;

        let size = if cfg!(target_os = "android") {
            // FIXME: inner_size() is only working AFTER Event::Resumed on Android
            PhysicalSize::new(500, 500)
        } else {
            window.inner_size()
        };

        // create an instance
        let instance = wgpu::Instance::new(wgpu::Backends::all());

        // create an surface
        let surface = unsafe { instance.create_surface(window) };

        // create an adapter
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let limits = if cfg!(feature = "web-webgl") {
            Limits {
                max_texture_dimension_2d: 4096,
                ..wgpu::Limits::downlevel_webgl2_defaults()
            }
        } else if cfg!(target_os = "android") {
            Limits {
                max_storage_textures_per_shader_stage: 4,
                ..wgpu::Limits::default()
            }
        } else {
            Limits {
                ..wgpu::Limits::default()
            }
        };

        // create a device and a queue
        let features = if DEBUG_WIREFRAME {
            wgpu::Features::default() | wgpu::Features::POLYGON_MODE_LINE
        } else {
            wgpu::Features::default()
        };

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features,
                    limits,
                },
                None,
            )
            .await
            .unwrap();

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: VERTEX_BUFFER_SIZE,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let feature_metadata_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: FEATURE_METADATA_BUFFER_SIZE,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let indices_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: INDICES_BUFFER_SIZE,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let globals_buffer_byte_size =
            cmp::max(MIN_BUFFER_SIZE, std::mem::size_of::<ShaderGlobals>() as u64);

        let metadata_buffer_size =
            std::mem::size_of::<ShaderLayerMetadata>() as u64 * TILE_META_COUNT;
        let metadata_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Tiles ubo"),
            size: metadata_buffer_size,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let globals_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Globals ubo"),
            size: globals_buffer_byte_size,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Bind group layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(globals_buffer_byte_size),
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Bind group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(
                    globals_uniform_buffer.as_entire_buffer_binding(),
                ),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
            label: None,
        });

        let mut vertex_shader = shaders::tile::VERTEX;
        let mut fragment_shader = shaders::tile::FRAGMENT;

        let render_pipeline_descriptor = create_map_render_pipeline_description(
            &pipeline_layout,
            vertex_shader.create_vertex_state(&device),
            fragment_shader.create_fragment_state(&device),
            sample_count,
            false,
        );

        let mut vertex_shader = shaders::tile_mask::VERTEX;
        let mut fragment_shader = shaders::tile_mask::FRAGMENT;

        let mask_pipeline_descriptor = create_map_render_pipeline_description(
            &pipeline_layout,
            vertex_shader.create_vertex_state(&device),
            fragment_shader.create_fragment_state(&device),
            sample_count,
            true,
        );

        let render_pipeline = device.create_render_pipeline(&render_pipeline_descriptor);
        let mask_pipeline = device.create_render_pipeline(&mask_pipeline_descriptor);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: COLOR_TEXTURE_FORMAT,
            width: size.width,
            height: size.height,
            // present_mode: wgpu::PresentMode::Mailbox,
            present_mode: wgpu::PresentMode::Fifo, // VSync
        };

        surface.configure(&device, &surface_config);

        let depth_texture =
            Texture::create_depth_texture(&device, &surface_config, "depth_texture", sample_count);

        let multisampling_texture = if sample_count > 1 {
            Some(Texture::create_multisampling_texture(
                &device,
                &surface_config,
                sample_count,
            ))
        } else {
            None
        };

        let camera = camera::Camera::new(
            (TILE_SIZE / 2.0, TILE_SIZE / 2.0, 150.0),
            cgmath::Deg(-90.0),
            cgmath::Deg(0.0),
            size.width,
            size.height,
        );
        let projection = camera::Perspective::new(
            surface_config.width,
            surface_config.height,
            cgmath::Deg(110.0),
            100.0,
            2000.0,
        );

        Self {
            instance,
            surface,
            device,
            queue,
            size,
            surface_config,
            render_pipeline,
            mask_pipeline,
            bind_group,
            multisampling_texture,
            depth_texture,
            sample_count,
            globals_uniform_buffer,
            fps_meter: FPSMeter::new(),
            camera,
            perspective: projection,
            suspended: false, // Initially the app is not suspended
            buffer_pool: BufferPool::new(
                BackingBufferDescriptor::new(vertex_buffer, VERTEX_BUFFER_SIZE),
                BackingBufferDescriptor::new(indices_buffer, INDICES_BUFFER_SIZE),
                BackingBufferDescriptor::new(metadata_buffer, metadata_buffer_size),
                BackingBufferDescriptor::new(feature_metadata_buffer, FEATURE_METADATA_BUFFER_SIZE),
            ),
            tile_mask_pattern: TileMaskPattern::new(),
            zoom: 0.0,
            style,
        }
    }

    pub fn recreate_surface(&mut self, window: &winit::window::Window) {
        // We only create a new surface if we are currently suspended. On Android (and probably iOS)
        // the surface gets invalid after the app has been suspended.
        if self.suspended {
            let surface = unsafe { self.instance.create_surface(window) };
            surface.configure(&self.device, &self.surface_config);
            self.surface = surface;
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        // While the app is suspended we can not re-configure a surface
        if self.suspended {
            return;
        }

        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.surface_config.width = new_size.width;
            self.surface_config.height = new_size.height;
            self.surface.configure(&self.device, &self.surface_config);

            self.perspective.resize(new_size.width, new_size.height);
            self.camera.resize(new_size.width, new_size.height);

            // Re-configure depth buffer
            self.depth_texture = Texture::create_depth_texture(
                &self.device,
                &self.surface_config,
                "depth_texture",
                self.sample_count,
            );

            // Re-configure multi-sampling buffer
            self.multisampling_texture = if self.sample_count > 1 {
                Some(Texture::create_multisampling_texture(
                    &self.device,
                    &self.surface_config,
                    self.sample_count,
                ))
            } else {
                None
            };
        }
    }

    pub fn visible_z(&self) -> u8 {
        self.zoom.floor() as u8
    }

    /// Request tiles which are currently in view
    #[tracing::instrument]
    fn request_tiles_in_view(&self, view_region: &ViewRegion, scheduler: &mut IOScheduler) {
        let source_layers: HashSet<String> = self
            .style
            .layers
            .iter()
            .filter_map(|layer| layer.source_layer.clone())
            .collect();

        for coords in view_region.iter() {
            if let Some(_) = coords.build_quad_key() {
                // TODO: Make tesselation depend on style?
                scheduler.try_request_tile(&coords, &source_layers).unwrap();
            }
        }
    }

    /// Update tile metadata for all required tiles on the GPU according to current zoom, camera and perspective
    /// We perform the update before uploading new tessellated tiles, such that each
    /// tile metadata in the the `buffer_pool` gets updated exactly once and not twice.
    #[tracing::instrument]
    fn update_metadata(
        &self,
        _scheduler: &mut IOScheduler,
        view_region: &ViewRegion,
        view_proj: &ViewProjection,
    ) {
        /*let animated_one = 0.5
        * (1.0
            + ((SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64()
                * 10.0)
                .sin()));*/

        // Factor which determines how much we need to adjust the width of lines for example.
        // If zoom == z -> zoom_factor == 1

        for entries in self.buffer_pool.index().iter() {
            for entry in entries {
                let world_coords = entry.coords;

                // FIXME: Does not take into account rendering tiles with different z
                if !view_region.is_in_view(&entry.coords) {
                    continue;
                }

                let zoom_factor = 2.0_f64.powf(world_coords.z as f64 - self.zoom) as f32;

                let transform: Matrix4<f32> = (view_proj
                    .to_model_view_projection(world_coords.transform_for_zoom(self.zoom)))
                .downcast();

                self.buffer_pool.update_layer_metadata(
                    &self.queue,
                    entry,
                    ShaderLayerMetadata::new(
                        transform.into(),
                        zoom_factor,
                        entry.style_layer.index as f32,
                    ),
                );

                // TODO: Update features
                /*let source_layer = entry.style_layer.source_layer.as_ref().unwrap();

                if let Some(result) = scheduler
                    .get_tile_cache()
                    .iter_tessellated_layers_at(&world_coords)
                    .unwrap()
                    .find(|layer| source_layer.as_str() == layer.layer_name())
                {
                    let color: Option<Vec4f32> = entry
                        .style_layer
                        .paint
                        .as_ref()
                        .and_then(|paint| paint.get_color())
                        .map(|mut color| {
                            color.color.b = animated_one as f32;
                            color.into()
                        });

                    match result {
                        LayerTessellateResult::UnavailableLayer { .. } => {}
                        LayerTessellateResult::TessellatedLayer {
                            layer_data,
                            feature_indices,
                            ..
                        } => {

                            let feature_metadata = layer_data
                                .features()
                                .iter()
                                .enumerate()
                                .flat_map(|(i, _feature)| {
                                    iter::repeat(ShaderFeatureStyle {
                                        color: color.unwrap(),
                                    })
                                    .take(feature_indices[i] as usize)
                                })
                                .collect::<Vec<_>>();

                            self.buffer_pool.update_feature_metadata(
                                &self.queue,
                                entry,
                                &feature_metadata,
                            );
                        }
                    }
                }*/
            }
        }
    }

    #[tracing::instrument]
    fn upload_tile_geometry(
        &mut self,
        view_proj: &ViewProjection,
        view_region: &ViewRegion,
        scheduler: &mut IOScheduler,
    ) {
        let visible_z = self.visible_z();

        // Factor which determines how much we need to adjust the width of lines for example.
        // If zoom == z -> zoom_factor == 1
        let zoom_factor = 2.0_f64.powf(visible_z as f64 - self.zoom) as f32; // TODO deduplicate

        // Upload all tessellated layers which are in view
        for world_coords in view_region.iter() {
            let loaded_layers = self
                .buffer_pool
                .get_loaded_layers_at(&world_coords)
                .unwrap_or_default();
            if let Some(available_layers) = scheduler
                .get_tile_cache()
                .iter_tessellated_layers_at(&world_coords)
                .map(|layers| {
                    layers
                        .filter(|result| !loaded_layers.contains(&result.layer_name()))
                        .collect::<Vec<_>>()
                })
            {
                for style_layer in &self.style.layers {
                    let source_layer = style_layer.source_layer.as_ref().unwrap();

                    if let Some(result) = available_layers
                        .iter()
                        .find(|layer| source_layer.as_str() == layer.layer_name())
                    {
                        let color: Option<Vec4f32> = style_layer
                            .paint
                            .as_ref()
                            .and_then(|paint| paint.get_color())
                            .map(|color| color.into());

                        match result {
                            LayerTessellateResult::UnavailableLayer { coords, .. } => {
                                /*self.buffer_pool.mark_layer_unavailable(*coords);*/
                            }
                            LayerTessellateResult::TessellatedLayer {
                                coords,
                                feature_indices,
                                layer_data,
                                buffer,
                                ..
                            } => {
                                let world_coords = coords;

                                let feature_metadata = layer_data
                                    .features()
                                    .iter()
                                    .enumerate()
                                    .flat_map(|(i, _feature)| {
                                        iter::repeat(ShaderFeatureStyle {
                                            color: color.unwrap(),
                                        })
                                        .take(feature_indices[i] as usize)
                                    })
                                    .collect::<Vec<_>>();

                                // We are casting here from 64bit to 32bit, because 32bit is more performant and is
                                // better supported.
                                let transform: Matrix4<f32> = (view_proj.to_model_view_projection(
                                    world_coords.transform_for_zoom(self.zoom),
                                ))
                                .downcast();

                                self.buffer_pool.allocate_layer_geometry(
                                    &self.queue,
                                    *coords,
                                    style_layer.clone(),
                                    buffer,
                                    ShaderLayerMetadata::new(
                                        transform.into(),
                                        zoom_factor,
                                        style_layer.index as f32,
                                    ),
                                    &feature_metadata,
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    #[tracing::instrument]
    pub fn prepare_render_data(&mut self, scheduler: &mut IOScheduler) {
        let visible_z = self.visible_z();

        let view_region = self
            .camera
            .view_region_bounding_box(&self.camera.calc_view_proj(&self.perspective).invert())
            .map(|bounding_box| ViewRegion::new(bounding_box, 1, self.zoom, visible_z));

        let view_proj = self.camera.calc_view_proj(&self.perspective);

        if let Some(view_region) = &view_region {
            self.update_metadata(scheduler, &view_region, &view_proj);
            self.upload_tile_geometry(&view_proj, &view_region, scheduler);
            self.request_tiles_in_view(view_region, scheduler);
        }

        // TODO: Could we draw inspiration from StagingBelt (https://docs.rs/wgpu/latest/wgpu/util/struct.StagingBelt.html)?
        // TODO: What is StagingBelt for?

        // Update globals
        self.queue.write_buffer(
            &self.globals_uniform_buffer,
            0,
            bytemuck::cast_slice(&[ShaderGlobals::new(
                self.camera.create_camera_uniform(&self.perspective),
            )]),
        );
    }

    #[tracing::instrument]
    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let frame = self.surface.get_current_texture()?;
        let frame_view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Encoder"),
            });

        {
            let color_attachment = if let Some(multisampling_target) = &self.multisampling_texture {
                wgpu::RenderPassColorAttachment {
                    view: &multisampling_target.view,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                        store: true,
                    },
                    resolve_target: Some(&frame_view),
                }
            } else {
                wgpu::RenderPassColorAttachment {
                    view: &frame_view,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                        store: true,
                    },
                    resolve_target: None,
                }
            };

            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[color_attachment],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0.0),
                        store: true,
                    }),
                    stencil_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0),
                        store: true,
                    }),
                }),
            });

            pass.set_bind_group(0, &self.bind_group, &[]);

            {
                let _span_ = tracing::span!(tracing::Level::TRACE, "render pass").entered();

                let visible_z = self.visible_z();
                let inverted_view_proj = self.camera.calc_view_proj(&self.perspective).invert();
                let view_region = self
                    .camera
                    .view_region_bounding_box(&inverted_view_proj)
                    .map(|bounding_box| ViewRegion::new(bounding_box, 1, self.zoom, visible_z));

                let index = self.buffer_pool.index();

                if let Some(view_region) = &view_region {
                    for world_coords in view_region.iter() {
                        tracing::trace!("Drawing tile at {world_coords}");

                        if let Some(entries) = index.get_layers_fallback(&world_coords) {
                            let mut to_render: Vec<&IndexEntry> = Vec::from_iter(entries);
                            to_render.sort_by_key(|entry| entry.style_layer.index);

                            let reference = self
                                .tile_mask_pattern
                                .stencil_reference_value(&world_coords)
                                as u32;

                            // Draw mask
                            if let Some(mask_entry) = entries.front() {
                                {
                                    tracing::trace!("Drawing mask {}", &mask_entry.coords);

                                    pass.set_pipeline(&self.mask_pipeline);
                                    pass.set_stencil_reference(reference);
                                    pass.set_vertex_buffer(
                                        0,
                                        self.buffer_pool
                                            .metadata()
                                            .slice(mask_entry.layer_metadata_buffer_range()),
                                    );
                                    pass.draw(0..6, 0..1);
                                }
                            }

                            for entry in to_render {
                                // Draw tile
                                {
                                    tracing::trace!(
                                        "Drawing layer {:?} at {}",
                                        entry.style_layer.source_layer,
                                        &entry.coords
                                    );

                                    pass.set_pipeline(&self.render_pipeline);
                                    pass.set_stencil_reference(reference);
                                    pass.set_index_buffer(
                                        self.buffer_pool
                                            .indices()
                                            .slice(entry.indices_buffer_range()),
                                        INDEX_FORMAT,
                                    );
                                    pass.set_vertex_buffer(
                                        0,
                                        self.buffer_pool
                                            .vertices()
                                            .slice(entry.vertices_buffer_range()),
                                    );
                                    pass.set_vertex_buffer(
                                        1,
                                        self.buffer_pool
                                            .metadata()
                                            .slice(entry.layer_metadata_buffer_range()),
                                    );
                                    pass.set_vertex_buffer(
                                        2,
                                        self.buffer_pool
                                            .feature_metadata()
                                            .slice(entry.feature_metadata_buffer_range()),
                                    );
                                    pass.draw_indexed(entry.indices_range(), 0, 0..1);
                                }
                            }
                        }
                    }
                }
            }
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();

        self.fps_meter.update_and_print();
        Ok(())
    }

    pub fn suspend(&mut self) {
        self.suspended = true;
    }

    pub fn resume(&mut self) {
        self.suspended = false;
    }
}

impl fmt::Debug for RenderState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RenderState")
    }
}
