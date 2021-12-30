use std::cmp;
use std::default::Default;
use std::io::Cursor;
use std::ops::Range;

use log::{trace, warn};
use lyon::tessellation::VertexBuffers;
use wgpu::util::DeviceExt;
use wgpu::{Buffer, Limits, Queue};
use winit::dpi::PhysicalSize;
use winit::event::{
    DeviceEvent, ElementState, KeyboardInput, MouseButton, TouchPhase, WindowEvent,
};
use winit::window::Window;

use vector_tile::parse_tile_reader;

use crate::fps_meter::FPSMeter;
use crate::io::cache::Cache;
use crate::io::{static_database, TileCoords};
use crate::platform::{COLOR_TEXTURE_FORMAT, MIN_BUFFER_SIZE};
use crate::render::buffer_pool::{BackingBufferDescriptor, BufferPool};
use crate::render::{camera, shaders};
use crate::tesselation::{IndexDataType, Tesselated};
use crate::util::measure::Measure;

use super::piplines::*;
use super::shader_ffi::*;
use super::texture::Texture;

pub struct SceneParams {
    pub stroke_width: f32,
}

impl Default for SceneParams {
    fn default() -> Self {
        SceneParams { stroke_width: 1.0 }
    }
}

const INDEX_FORMAT: wgpu::IndexFormat = wgpu::IndexFormat::Uint32;

pub struct State {
    instance: wgpu::Instance,

    device: wgpu::Device,
    queue: wgpu::Queue,

    fps_meter: FPSMeter,

    surface: wgpu::Surface,
    surface_config: wgpu::SurfaceConfiguration,
    suspended: bool,

    size: winit::dpi::PhysicalSize<u32>,

    render_pipeline: wgpu::RenderPipeline,
    mask_pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,

    sample_count: u32,
    multisampling_texture: Option<Texture>,

    depth_texture: Texture,

    tiles_uniform_buffer: wgpu::Buffer,
    globals_uniform_buffer: wgpu::Buffer,

    buffer_pool: BufferPool<Queue, Buffer, GpuVertexUniform, IndexDataType>,

    tile_mask_instances: wgpu::Buffer,

    pub camera: camera::Camera,
    projection: camera::Projection,

    pub scene: SceneParams,
}

impl SceneParams {
    pub fn new() -> Self {
        Self {
            ..SceneParams::default()
        }
    }
}

impl State {
    pub async fn new(window: &Window) -> Self {
        let mut measure = Measure::time();

        let sample_count = 4;

        let size = if cfg!(target_os = "android") {
            // FIXME: inner_size() is only working AFTER Event::Resumed on Android
            PhysicalSize::new(500, 500)
        } else {
            window.inner_size()
        };

        println!(
            "Using static database from {}",
            static_database::get_source_path()
        );

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
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::default(),
                    limits,
                },
                None,
            )
            .await
            .unwrap();

        let vertex_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 1024 * 1024 * 16,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let indices_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 1024 * 1024 * 16,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let instances = [
            // Step 1
            MaskInstanceUniform::new([0.0, 0.0], 4.0, 4.0, [1.0, 0.0, 0.0, 1.0]), // horizontal
            //MaskInstanceUniform::new([0.0, 2.0 * 4096.0], 4.0, 1.0, [1.0, 0.0, 0.0, 1.0]), // vertical
            // Step 2
            MaskInstanceUniform::new([1.0 * 4096.0, 0.0], 1.0, 4.0, [0.0, 1.0, 0.0, 1.0]), // vertical
            MaskInstanceUniform::new([3.0 * 4096.0, 0.0], 1.0, 4.0, [0.0, 1.0, 0.0, 1.0]), // vertical
            // Step 3
            MaskInstanceUniform::new([0.0, 1.0 * 4096.0], 4.0, 1.0, [0.0, 0.0, 1.0, 1.0]), // horizontal
            MaskInstanceUniform::new([0.0, 3.0 * 4096.0], 4.0, 1.0, [0.0, 0.0, 1.0, 1.0]), // horizontal
            // Step 4
            MaskInstanceUniform::new([1.0 * 4096.0, 0.0], 1.0, 4.0, [0.5, 0.25, 0.5, 1.0]), // vertical
            MaskInstanceUniform::new([3.0 * 4096.0, 0.0], 1.0, 4.0, [0.5, 0.25, 0.5, 1.0]), // vertical
        ];

        let tile_mask_instances = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&instances),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let globals_buffer_byte_size = cmp::max(
            MIN_BUFFER_SIZE,
            std::mem::size_of::<GlobalsUniform>() as u64,
        );

        let tiles_uniform_buffer_size = std::mem::size_of::<TileUniform>() as u64 * 128; // FIXME: Tile count?
        let tiles_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Tiles ubo"),
            size: tiles_uniform_buffer_size,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
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
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(globals_buffer_byte_size),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(tiles_uniform_buffer_size),
                    },
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Bind group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(
                        globals_uniform_buffer.as_entire_buffer_binding(),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(
                        tiles_uniform_buffer.as_entire_buffer_binding(),
                    ),
                },
            ],
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

        let camera = camera::Camera::new((0.0, 5.0, 5000.0), cgmath::Deg(-90.0), cgmath::Deg(-0.0));
        let projection = camera::Projection::new(
            surface_config.width,
            surface_config.height,
            cgmath::Deg(45.0),
            0.1,
            100000.0,
        );

        measure.breadcrumb("initialized");

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
            scene: SceneParams::new(),
            globals_uniform_buffer,
            tiles_uniform_buffer: tiles_uniform_buffer,
            fps_meter: FPSMeter::new(),
            tile_mask_instances,
            camera,
            projection,
            suspended: false, // Initially the app is not suspended
            buffer_pool: BufferPool::new(
                BackingBufferDescriptor(vertex_uniform_buffer, 1024 * 1024 * 16),
                BackingBufferDescriptor(indices_uniform_buffer, 1024 * 1024 * 16),
            ),
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

            self.projection.resize(new_size.width, new_size.height);

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

    pub fn upload_tile_geometry(&mut self, cache: &Cache) {
        const OFFSET_X: u32 = 2179;
        const OFFSET_Y: u32 = 1421;

        let upload = cache.pop_all();

        for tile in upload.iter() {
            let new_coords = TileCoords {
                x: tile.coords.x - OFFSET_X,
                y: tile.coords.y - OFFSET_Y,
                z: tile.coords.z,
            };

            self.buffer_pool
                .allocate_geometry(&self.queue, tile.id, new_coords, &tile.geometry);

            let uniform = TileUniform::new(
                [0.0, 0.0, 0.0, 1.0],
                [new_coords.x as f32 * 4096.0, new_coords.y as f32 * 4096.0],
            );
            self.queue.write_buffer(
                &self.tiles_uniform_buffer,
                std::mem::size_of::<TileUniform>() as u64 * tile.id as u64,
                bytemuck::cast_slice(&[uniform]),
            );
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let frame = self.surface.get_current_texture()?;
        let scene = &mut self.scene;
        let frame_view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        {
            self.queue.write_buffer(
                &self.globals_uniform_buffer,
                0,
                bytemuck::cast_slice(&[GlobalsUniform::new(
                    self.camera.create_camera_uniform(&self.projection),
                )]),
            );
        }

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
                // Draw masks
                pass.set_pipeline(&self.mask_pipeline);
                pass.set_vertex_buffer(0, self.tile_mask_instances.slice(..));
                // Draw 11 squares each out of 6 vertices
                pass.draw(0..6, 0..7);
            }
            {
                for entry in self.buffer_pool.available_vertices() {
                    let TileCoords { x, y, .. } = entry.coords;

                    pass.set_pipeline(&self.render_pipeline);
                    let reference = match (x, y) {
                        (x, y) if x % 2 == 0 && y % 2 == 0 => 1,
                        (x, y) if x % 2 == 0 && y % 2 != 0 => 2,
                        (x, y) if x % 2 != 0 && y % 2 == 0 => 3,
                        (x, y) if x % 2 != 0 && y % 2 != 0 => 4,
                        _ => 0,
                    };
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
                    /* if !self.tile_fill_range.is_empty() {
                        pass.draw_indexed(self.tile_fill_range.clone(), 0, 0..1);
                    }*/
                    trace!("current buffer_pool index {:?}", self.buffer_pool.index);
                    // FIXME: Custom Instance index possibly breaks on Metal
                    pass.draw_indexed(
                        entry.indices_range(),
                        0,
                        entry.id as u32..(entry.id + 1) as u32,
                    );
                }
            }
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();

        self.fps_meter.update_and_print();
        Ok(())
    }

    pub fn is_suspended(&self) -> bool {
        self.suspended
    }

    pub fn suspend(&mut self) {
        self.suspended = true;
    }

    pub fn resume(&mut self) {
        self.suspended = false;
    }
}
