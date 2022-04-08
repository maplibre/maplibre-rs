use std::default::Default;

use std::{cmp, iter};

use tracing;
use wgpu::{Buffer, Limits, Queue, Surface, SurfaceConfiguration};

use style_spec::Style;

use crate::coords::{ViewRegion, Zoom};

use crate::io::tile_cache::TileCache;
use crate::io::LayerTessellateMessage;
use crate::platform::MIN_BUFFER_SIZE;
use crate::render::buffer_pool::{BackingBufferDescriptor, BufferPool, IndexEntry};

use crate::render::camera::{Camera, ViewProjection};
use crate::render::options::{
    DEBUG_WIREFRAME, FEATURE_METADATA_BUFFER_SIZE, INDEX_FORMAT, INDICES_BUFFER_SIZE,
    LAYER_METADATA_BUFFER_SIZE, TILE_VIEW_BUFFER_SIZE, VERTEX_BUFFER_SIZE,
};
use crate::render::tile_view_pattern::{TileInView, TileViewPattern};
use crate::tessellation::IndexDataType;
use crate::util::FPSMeter;
use crate::WindowSize;

use super::piplines::*;
use super::shaders;
use super::shaders::*;
use super::texture::Texture;

pub struct RenderState {
    instance: wgpu::Instance,

    device: wgpu::Device,
    queue: wgpu::Queue,

    fps_meter: FPSMeter,

    surface: wgpu::Surface,
    surface_config: wgpu::SurfaceConfiguration,
    suspended: bool,

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

    tile_view_pattern: TileViewPattern<Queue, Buffer>,
}

pub type SurfaceFactory = dyn FnOnce(&wgpu::Instance) -> (Surface, SurfaceConfiguration);

impl RenderState {
    pub async fn initialize<W: raw_window_handle::HasRawWindowHandle>(
        window: &W,
        window_size: WindowSize,
    ) -> Self {
        let sample_count = 4;

        //let instance = wgpu::Instance::new(wgpu::Backends::GL);
        let instance = wgpu::Instance::new(wgpu::Backends::all());

        let surface = unsafe { instance.create_surface(&window) };
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: crate::platform::COLOR_TEXTURE_FORMAT,
            width: window_size.width,
            height: window_size.height,
            // present_mode: wgpu::PresentMode::Mailbox,
            present_mode: wgpu::PresentMode::Fifo, // VSync
        };

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

        surface.configure(&device, &surface_config);

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

        let tile_view_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: TILE_VIEW_BUFFER_SIZE,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let globals_buffer_byte_size =
            cmp::max(MIN_BUFFER_SIZE, std::mem::size_of::<ShaderGlobals>() as u64);

        let layer_metadata_buffer_size =
            std::mem::size_of::<ShaderLayerMetadata>() as u64 * LAYER_METADATA_BUFFER_SIZE;
        let layer_metadata_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Layer Metadata ubo"),
            size: layer_metadata_buffer_size,
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

        let depth_texture = Texture::create_depth_texture(&device, &surface_config, sample_count);

        let multisampling_texture = if sample_count > 1 {
            Some(Texture::create_multisampling_texture(
                &device,
                &surface_config,
                sample_count,
            ))
        } else {
            None
        };

        Self {
            instance,
            surface,
            device,
            queue,
            surface_config,
            render_pipeline,
            mask_pipeline,
            bind_group,
            multisampling_texture,
            depth_texture,
            sample_count,
            globals_uniform_buffer,
            fps_meter: FPSMeter::new(),
            suspended: false, // Initially rendering is not suspended
            buffer_pool: BufferPool::new(
                BackingBufferDescriptor::new(vertex_buffer, VERTEX_BUFFER_SIZE),
                BackingBufferDescriptor::new(indices_buffer, INDICES_BUFFER_SIZE),
                BackingBufferDescriptor::new(layer_metadata_buffer, layer_metadata_buffer_size),
                BackingBufferDescriptor::new(feature_metadata_buffer, FEATURE_METADATA_BUFFER_SIZE),
            ),
            tile_view_pattern: TileViewPattern::new(BackingBufferDescriptor::new(
                tile_view_buffer,
                TILE_VIEW_BUFFER_SIZE,
            )),
        }
    }

    pub fn recreate_surface<W: raw_window_handle::HasRawWindowHandle>(&mut self, window: &W) {
        // We only create a new surface if we are currently suspended. On Android (and probably iOS)
        // the surface gets invalid after the app has been suspended.
        if self.suspended {
            let surface = unsafe { self.instance.create_surface(window) };
            surface.configure(&self.device, &self.surface_config);
            self.surface = surface;
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        // While the app is suspended we can not re-configure a surface
        if self.suspended {
            return;
        }

        self.surface_config.width = width;
        self.surface_config.height = height;

        self.surface.configure(&self.device, &self.surface_config);

        // Re-configure depth buffer
        self.depth_texture =
            Texture::create_depth_texture(&self.device, &self.surface_config, self.sample_count);

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

    pub fn update_globals(&self, view_proj: &ViewProjection, camera: &Camera) {
        // Update globals
        self.queue.write_buffer(
            &self.globals_uniform_buffer,
            0,
            bytemuck::cast_slice(&[ShaderGlobals::new(ShaderCamera::new(
                view_proj.downcast().into(),
                camera
                    .position
                    .to_homogeneous()
                    .cast::<f32>()
                    .unwrap()
                    .into(),
            ))]),
        );
    }

    #[tracing::instrument(skip_all)]
    pub(crate) fn update_metadata(&mut self) {
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

        /*  for entries in self.buffer_pool.index().iter() {
        for entry in entries {
            let world_coords = entry.coords;*/

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
        /*            }
        }*/
    }

    #[tracing::instrument(skip_all)]
    pub fn update_tile_view_pattern(
        &mut self,
        view_region: &ViewRegion,
        view_proj: &ViewProjection,
        zoom: Zoom,
    ) {
        self.tile_view_pattern
            .update_pattern(view_region, &self.buffer_pool, zoom);
        self.tile_view_pattern
            .upload_pattern(&self.queue, view_proj);
    }

    #[tracing::instrument(skip_all)]
    pub fn upload_tile_geometry(
        &mut self,
        view_region: &ViewRegion,
        style: &Style,
        tile_cache: &TileCache,
    ) {
        // Upload all tessellated layers which are in view
        for world_coords in view_region.iter() {
            let loaded_layers = self
                .buffer_pool
                .get_loaded_layers_at(&world_coords)
                .unwrap_or_default();
            if let Some(available_layers) = tile_cache
                .iter_tessellated_layers_at(&world_coords)
                .map(|layers| {
                    layers
                        .filter(|result| !loaded_layers.contains(&result.layer_name()))
                        .collect::<Vec<_>>()
                })
            {
                for style_layer in &style.layers {
                    let source_layer = style_layer.source_layer.as_ref().unwrap();

                    if let Some(message) = available_layers
                        .iter()
                        .find(|layer| source_layer.as_str() == layer.layer_name())
                    {
                        let color: Option<Vec4f32> = style_layer
                            .paint
                            .as_ref()
                            .and_then(|paint| paint.get_color())
                            .map(|color| color.into());

                        match message {
                            LayerTessellateMessage::UnavailableLayer { coords: _, .. } => {
                                /*self.buffer_pool.mark_layer_unavailable(*coords);*/
                            }
                            LayerTessellateMessage::TessellatedLayer {
                                coords,
                                feature_indices,
                                layer_data,
                                buffer,
                                ..
                            } => {
                                let allocate_feature_metadata = tracing::span!(
                                    tracing::Level::TRACE,
                                    "allocate_feature_metadata"
                                );

                                let guard = allocate_feature_metadata.enter();
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
                                drop(guard);

                                tracing::trace!("Allocating geometry at {}", &coords);
                                self.buffer_pool.allocate_layer_geometry(
                                    &self.queue,
                                    *coords,
                                    style_layer.clone(),
                                    buffer,
                                    ShaderLayerMetadata::new(style_layer.index as f32),
                                    &feature_metadata,
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    #[tracing::instrument(skip_all)]
    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let render_setup_span = tracing::span!(tracing::Level::TRACE, "render prepare");
        let _guard = render_setup_span.enter();

        let frame = self.surface.get_current_texture()?;
        let frame_view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Encoder"),
            });

        drop(_guard);

        {
            let _span_ = tracing::span!(tracing::Level::TRACE, "render pass").entered();
            {
                let color_attachment =
                    if let Some(multisampling_target) = &self.multisampling_texture {
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
                    let index = self.buffer_pool.index();

                    for TileInView { shape, fallback } in self.tile_view_pattern.iter() {
                        let coords = shape.coords;
                        tracing::trace!("Drawing tile at {coords}");

                        let shape_to_render = fallback.as_ref().unwrap_or(shape);

                        let reference = self
                            .tile_view_pattern
                            .stencil_reference_value(&shape_to_render.coords)
                            as u32;

                        // Draw mask
                        {
                            tracing::trace!("Drawing mask {}", &coords);

                            pass.set_pipeline(&self.mask_pipeline);
                            pass.set_stencil_reference(reference);
                            pass.set_vertex_buffer(
                                0,
                                self.tile_view_pattern
                                    .buffer()
                                    .slice(shape.buffer_range.clone()),
                            );
                            pass.draw(0..6, 0..1);
                        }

                        if let Some(entries) = index.get_layers(&shape_to_render.coords) {
                            let mut layers_to_render: Vec<&IndexEntry> = Vec::from_iter(entries);
                            layers_to_render.sort_by_key(|entry| entry.style_layer.index);

                            for entry in layers_to_render {
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
                                        self.tile_view_pattern
                                            .buffer()
                                            .slice(shape_to_render.buffer_range.clone()),
                                    );
                                    pass.set_vertex_buffer(
                                        2,
                                        self.buffer_pool
                                            .metadata()
                                            .slice(entry.layer_metadata_buffer_range()),
                                    );
                                    pass.set_vertex_buffer(
                                        3,
                                        self.buffer_pool
                                            .feature_metadata()
                                            .slice(entry.feature_metadata_buffer_range()),
                                    );
                                    pass.draw_indexed(entry.indices_range(), 0, 0..1);
                                }
                            }
                        } else {
                            tracing::trace!("No layers found at {}", &shape_to_render.coords);
                        }
                    }
                }
            }
        }

        {
            let _span = tracing::span!(tracing::Level::TRACE, "render finish").entered();
            tracing::trace!("Finished drawing");

            self.queue.submit(Some(encoder.finish()));
            tracing::trace!("Submitted queue");

            frame.present();
            tracing::trace!("Presented frame");
        }

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
