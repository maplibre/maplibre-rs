use std::cmp;
use std::io::Cursor;
use std::ops::Range;

use log::warn;
use lyon::tessellation::VertexBuffers;
use wgpu::{Extent3d, Limits};
use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;
use winit::event::{DeviceEvent, ElementState, KeyboardInput, MouseButton, TouchPhase, WindowEvent};
use winit::window::Window;

use vector_tile::parse_tile_reader;

use crate::fps_meter::FPSMeter;
use crate::io::static_database;
use crate::render::{camera, shaders};
use crate::render::camera::CameraController;
use crate::render::tesselation::TileMask;

use super::piplines::*;
use super::platform_constants::{COLOR_TEXTURE_FORMAT, MIN_BUFFER_SIZE};
use super::shader_ffi::*;
use super::tesselation::Tesselated;
use super::texture::Texture;

pub struct SceneParams {
    stroke_width: f32,
    target_stroke_width: f32,

    last_touch: Option<(f64, f64)>,

    cpu_primitives: Vec<PrimitiveUniform>,
}

impl Default for SceneParams {
    fn default() -> Self {
        SceneParams {
            stroke_width: 1.0,
            target_stroke_width: 1.0,
            last_touch: None,
            cpu_primitives: vec![],
        }
    }
}

const INDEX_FORMAT: wgpu::IndexFormat = wgpu::IndexFormat::Uint32;

type IndexDataType = u32; // Must match INDEX_FORMAT

const PRIM_BUFFER_LEN: usize = 256;
const STROKE_PRIM_ID: u32 = 0;
const FILL_PRIM_ID: u32 = 1;
const SECOND_TILE_FILL_PRIM_ID: u32 = 2;
const MASK_FILL_PRIM_ID: u32 = 3;
const SECOND_TILE_STROKE_PRIM_ID: u32 = 5;

pub struct State {
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

    prims_uniform_buffer: wgpu::Buffer,
    globals_uniform_buffer: wgpu::Buffer,

    vertex_uniform_buffer: wgpu::Buffer,
    indices_uniform_buffer: wgpu::Buffer,
    tile_fill_range: Range<u32>,
    tile_stroke_range: Range<u32>,
    tile2_fill_range: Range<u32>,
    tile2_stroke_range: Range<u32>,

    tile_mask_vertex_uniform_buffer: wgpu::Buffer,
    tile_mask_indices_uniform_buffer: wgpu::Buffer,
    tile_mask_range: Range<u32>,
    tile_mask_instances: wgpu::Buffer,

    camera: camera::Camera,
    projection: camera::Projection,
    camera_controller: camera::CameraController,
    mouse_pressed: bool,

    scene: SceneParams,
}

impl SceneParams {
    pub fn new() -> Self {
        let mut cpu_primitives = Vec::with_capacity(PRIM_BUFFER_LEN);
        for _ in 0..PRIM_BUFFER_LEN {
            cpu_primitives.push(PrimitiveUniform::new(
                [1.0, 0.0, 0.0, 1.0],
                [0.0, 0.0],
                0,
                0.0,
                0.0,
                1.0,
            ));
        }

        // Stroke primitive
        cpu_primitives[STROKE_PRIM_ID as usize] =
            PrimitiveUniform::new([1.0, 0.0, 1.0, 1.0], [0.0, 0.0], 0, 1.0, 0.0, 1.0);
        cpu_primitives[SECOND_TILE_STROKE_PRIM_ID as usize] =
            PrimitiveUniform::new([0.5, 0.8, 0.1, 1.0], [4096.0, 0.0], 0, 1.0, 0.0, 1.0);
        // Main fill primitive
        cpu_primitives[FILL_PRIM_ID as usize] =
            PrimitiveUniform::new([0.0, 0.0, 1.0, 1.0], [0.0, 0.0], 0, 1.0, 0.0, 1.0);
        cpu_primitives[SECOND_TILE_FILL_PRIM_ID as usize] =
            PrimitiveUniform::new([0.0, 1.0, 1.0, 1.0], [4096.0, 0.0], 0, 1.0, 0.0, 1.0);

        cpu_primitives[MASK_FILL_PRIM_ID as usize] =
            PrimitiveUniform::new([0.0, 0.0, 1.0, 1.0], [0.0, 0.0], 0, 1.0, 0.0, 1.0);


        Self {
            cpu_primitives,
            ..SceneParams::default()
        }
    }
}

impl State {
    pub async fn new(window: &Window) -> Self {
        let sample_count = 4;

        let size = if cfg!(target_os = "android") {
            // FIXME: inner_size() is only working AFTER Event::Resumed on Android
            PhysicalSize::new(500, 500)
        } else {
            window.inner_size()
        };

        let mut geometry: VertexBuffers<GpuVertexUniform, IndexDataType> = VertexBuffers::new();

        println!("Using static database from {}", static_database::get_source_path());

        let tile = parse_tile_reader(&mut Cursor::new(static_database::get_tile(2179, 1421, 12).unwrap().contents())).expect("failed to load tile");
        let (tile_stroke_range, tile_fill_range) = (
            tile.tesselate_stroke(&mut geometry, STROKE_PRIM_ID),
            //tile.empty_range(&mut geometry, STROKE_PRIM_ID),
            tile.tesselate_fill(&mut geometry, FILL_PRIM_ID),
        );

        // tile right to it
        let tile = parse_tile_reader(&mut Cursor::new(static_database::get_tile(2180, 1421, 12).unwrap().contents())).expect("failed to load tile");
        let (tile2_stroke_range, tile2_fill_range) = (
            tile.tesselate_stroke(&mut geometry, SECOND_TILE_STROKE_PRIM_ID),
            //tile.empty_range(&mut geometry, STROKE_PRIM_ID),
            tile.tesselate_fill(&mut geometry, SECOND_TILE_FILL_PRIM_ID),
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

        let vertex_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&geometry.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let indices_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&geometry.indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let mut tile_mask_geometry: VertexBuffers<GpuVertexUniform, IndexDataType> = VertexBuffers::new();
        let tile_mask = TileMask();
        let tile_mask_range = tile_mask.tesselate_fill(&mut tile_mask_geometry, MASK_FILL_PRIM_ID);
        let tile_mask_vertex_uniform_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&tile_mask_geometry.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let tile_mask_indices_uniform_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&tile_mask_geometry.indices),
                usage: wgpu::BufferUsages::INDEX,
            });

        let instances = [
            GpuVertexUniform::new([0.0, 0.0], [0.0, 0.0], 0),
            GpuVertexUniform::new([4096.0, 0.0], [0.0, 0.0], 0),
        ];

        let tile_mask_instances =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&instances),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let prim_buffer_byte_size = cmp::max(
            MIN_BUFFER_SIZE,
            (PRIM_BUFFER_LEN * std::mem::size_of::<PrimitiveUniform>()) as u64,
        );
        let globals_buffer_byte_size = cmp::max(
            MIN_BUFFER_SIZE,
            std::mem::size_of::<GlobalsUniform>() as u64,
        );

        let prims_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Prims ubo"),
            size: prim_buffer_byte_size,
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
                        min_binding_size: wgpu::BufferSize::new(prim_buffer_byte_size),
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
                        prims_uniform_buffer.as_entire_buffer_binding(),
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

        // TODO: this isn't what we want: we'd need the equivalent of VK_POLYGON_MODE_LINE,
        // but it doesn't seem to be exposed by wgpu?
        //render_pipeline_descriptor.primitive.topology = wgpu::PrimitiveTopology::LineList;

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
        /*
        let data = [1; 512 * 512] ;

        queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::StencilOnly,
                texture: &depth_texture.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(512),
                rows_per_image: None,
            },
            Extent3d {
                width: 10,
                height: 10,
                depth_or_array_layers: 1,
            }
        );*/

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
        let camera_controller = camera::CameraController::new(3000.0, 0.2);

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
            tile_fill_range,
            scene: SceneParams::new(),
            vertex_uniform_buffer,
            globals_uniform_buffer,
            prims_uniform_buffer,
            indices_uniform_buffer,
            tile_mask_vertex_uniform_buffer,
            fps_meter: FPSMeter::new(),
            tile_stroke_range,
            tile2_fill_range,
            tile_mask_indices_uniform_buffer,
            tile_mask_range,
            tile_mask_instances,
            camera,
            projection,
            camera_controller,
            mouse_pressed: false,
            tile2_stroke_range,
            suspended: false, // Initially the app is not suspended
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

    pub fn device_input(&mut self, event: &DeviceEvent, window: &Window) -> bool {
        match event {
            DeviceEvent::MouseMotion { delta } => {
                if self.mouse_pressed {
                    warn!("mouse {}", delta.0);
                    self.camera_controller.process_mouse(delta.0 / window.scale_factor(), delta.1 / window.scale_factor());
                }
                true
            }
            _ => false,
        }
    }

    pub fn window_input(&mut self, event: &WindowEvent, window: &Window) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input:
                KeyboardInput {
                    state,
                    virtual_keycode: Some(key),
                    ..
                },
                ..
            } => match key {
                winit::event::VirtualKeyCode::Z => {
                    self.scene.target_stroke_width *= 1.2;
                    true
                }
                winit::event::VirtualKeyCode::H => {
                    self.scene.target_stroke_width *= 0.8;
                    true
                }
                _ => self.camera_controller.process_keyboard(*key, *state),
            },
            WindowEvent::Touch(touch) => {
                match touch.phase {
                    TouchPhase::Started => {
                        self.scene.last_touch = Some((touch.location.x, touch.location.y))
                    }
                    TouchPhase::Moved | TouchPhase::Ended => {
                        if let Some(start) = self.scene.last_touch {
                            let delta_x = start.0 - touch.location.x;
                            let delta_y = start.1 - touch.location.y;
                            warn!("touch {} {} {}", delta_x, delta_y, window.scale_factor());
                            self.camera_controller.process_touch(delta_x / window.scale_factor(), delta_y / window.scale_factor());
                        }

                        self.scene.last_touch = Some((touch.location.x, touch.location.y))
                    }
                    TouchPhase::Cancelled => {}
                }


                true
            }
            WindowEvent::MouseWheel { delta, .. } => {
                self.camera_controller.process_scroll(delta);
                true
            }
            WindowEvent::MouseInput {
                button: MouseButton::Left, // Left Mouse Button
                state,
                ..
            } => {
                self.mouse_pressed = *state == ElementState::Pressed;
                true
            }
            _ => false,
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
                    CameraController::create_camera_uniform(&self.camera, &self.projection),
                )]),
            );

            self.queue.write_buffer(
                &self.prims_uniform_buffer,
                0,
                bytemuck::cast_slice(&scene.cpu_primitives),
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
                // Increment stencil
                pass.set_pipeline(&self.mask_pipeline);
                pass.set_index_buffer(
                    self.tile_mask_indices_uniform_buffer.slice(..),
                    INDEX_FORMAT,
                );
                pass.set_vertex_buffer(0, self.tile_mask_vertex_uniform_buffer.slice(..));
                pass.set_vertex_buffer(1, self.tile_mask_instances.slice(..));
                pass.draw_indexed(self.tile_mask_range.clone(), 0, 0..2);
            }
            {
                pass.set_pipeline(&self.render_pipeline);
                pass.set_stencil_reference(2);
                pass.set_index_buffer(
                    self.indices_uniform_buffer.slice(..),
                    INDEX_FORMAT,
                );
                pass.set_vertex_buffer(0, self.vertex_uniform_buffer.slice(..));
                if (self.tile_fill_range.len() > 0) {
                    pass.draw_indexed(self.tile_fill_range.clone(), 0, 0..1);
                }
                pass.draw_indexed(self.tile_stroke_range.clone(), 0, 0..1);
            }
            {
                pass.set_pipeline(&self.render_pipeline);
                pass.set_stencil_reference(1);
                pass.set_index_buffer(
                    self.indices_uniform_buffer.slice(..),
                    INDEX_FORMAT,
                );
                pass.set_vertex_buffer(0, self.vertex_uniform_buffer.slice(..));
                if (self.tile2_fill_range.len() > 0) {
                    pass.draw_indexed(self.tile2_fill_range.clone(), 0, 0..1);
                }
                pass.draw_indexed(self.tile2_stroke_range.clone(), 0, 0..1);
            }
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();

        Ok(())
    }

    pub fn update(&mut self, dt: std::time::Duration) {
        let scene = &mut self.scene;
        let time_secs = self.fps_meter.time_secs as f32;

        self.camera_controller.update_camera(&mut self.camera, dt);

        // Animate the stroke_width to match target_stroke_width
        scene.stroke_width =
            scene.stroke_width + (scene.target_stroke_width - scene.stroke_width) / 5.0;

        // Animate the strokes of primitive
        scene.cpu_primitives[STROKE_PRIM_ID as usize].width = scene.stroke_width;
        /*        scene.cpu_primitives[STROKE_PRIM_ID as usize].color = [
                    (time_secs * 0.8 - 1.6).sin() * 0.1 + 0.1,
                    (time_secs * 0.5 - 1.6).sin() * 0.1 + 0.1,
                    (time_secs - 1.6).sin() * 0.1 + 0.1,
                    1.0,
                ];
        */
        self.fps_meter.update_and_print()
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
