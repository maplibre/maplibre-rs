use std::cmp;
use std::io::Cursor;
use std::ops::Range;

use lyon::math::Vector;
use lyon::tessellation::VertexBuffers;
use vector_tile::parse_tile_reader;
use wgpu::util::DeviceExt;
use winit::event::{DeviceEvent, ElementState, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::window::Window;

use crate::fps_meter::FPSMeter;
use crate::render::camera;
use crate::render::tesselation::TileMask;

use super::piplines::*;
use super::platform_constants::{COLOR_TEXTURE_FORMAT, MIN_BUFFER_SIZE};
use super::shader::*;
use super::shader_ffi::*;
use super::tesselation::Tesselated;
use super::texture::Texture;

pub struct SceneParams {
    pub target_zoom: f32,
    pub zoom: f32,
    pub target_scroll: Vector,
    pub scroll: Vector,
    pub stroke_width: f32,
    pub target_stroke_width: f32,
    cpu_primitives: Vec<Primitive>,
}

impl Default for SceneParams {
    fn default() -> Self {
        SceneParams {
            target_zoom: 5.0,
            zoom: 5.0,
            target_scroll: Vector::new(70.0, 70.0),
            scroll: Vector::new(70.0, 70.0),
            stroke_width: 1.0,
            target_stroke_width: 1.0,
            cpu_primitives: vec![],
        }
    }
}

const PRIM_BUFFER_LEN: usize = 256;
const STROKE_PRIM_ID: u32 = 0;
const FILL_PRIM_ID: u32 = 1;

pub struct State {
    device: wgpu::Device,
    queue: wgpu::Queue,

    fps_meter: FPSMeter,

    surface: wgpu::Surface,
    surface_config: wgpu::SurfaceConfiguration,

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

    tile_mask_vertex_uniform_buffer: wgpu::Buffer,
    tile_mask_indices_uniform_buffer: wgpu::Buffer,
    tile_mask_range: Range<u32>,

    camera: camera::Camera,                      // UPDATED!
    projection: camera::Projection,              // NEW!
    camera_controller: camera::CameraController, // UPDATED!
    camera_uniform: camera::CameraUniform,       // UPDATED!

    scene: SceneParams,
}

const TEST_TILES: &[u8] = include_bytes!("../../test-data/12-2176-1425.pbf");
impl SceneParams {
    pub fn new() -> Self {
        let mut cpu_primitives = Vec::with_capacity(PRIM_BUFFER_LEN);
        for _ in 0..PRIM_BUFFER_LEN {
            cpu_primitives.push(Primitive::new(
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
            Primitive::new([0.0, 0.0, 0.0, 1.0], [0.0, 0.0], 0, 1.0, 0.0, 1.0);
        // Main fill primitive
        cpu_primitives[FILL_PRIM_ID as usize] =
            Primitive::new([0.0, 0.0, 0.0, 1.0], [0.0, 0.0], 0, 0.0, 0.0, 1.0);

        Self {
            cpu_primitives,
            ..SceneParams::default()
        }
    }
}

impl State {
    pub async fn new(window: &Window) -> Self {
        let sample_count = 4;

        let size = window.inner_size();

        let mut geometry: VertexBuffers<GpuVertex, u16> = VertexBuffers::new();
        //let tile = parse_tile("test-data/12-2176-1425.pbf").expect("failed loading tile");

        let tile = parse_tile_reader(&mut Cursor::new(TEST_TILES));
        let (tile_stroke_range, tile_fill_range) = (
            tile.tesselate_stroke(&mut geometry, STROKE_PRIM_ID),
            tile.tesselate_fill(&mut geometry, FILL_PRIM_ID),
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
            wgpu::Limits::downlevel_webgl2_defaults()
        } else {
            wgpu::Limits::default()
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

        let mut tile_mask_geometry: VertexBuffers<GpuVertex, u16> = VertexBuffers::new();
        let tile_mask = TileMask();
        let tile_mask_range = tile_mask.tesselate_fill(&mut tile_mask_geometry, FILL_PRIM_ID);
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

        let prim_buffer_byte_size = cmp::max(
            MIN_BUFFER_SIZE,
            (PRIM_BUFFER_LEN * std::mem::size_of::<Primitive>()) as u64,
        );
        let globals_buffer_byte_size =
            cmp::max(MIN_BUFFER_SIZE, std::mem::size_of::<Globals>() as u64);

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

        let vertex_module = device.create_shader_module(&create_vertex_module_descriptor());
        let fragment_module = device.create_shader_module(&create_fragment_module_descriptor());
        let render_pipeline_descriptor = create_map_render_pipeline_description(
            &pipeline_layout,
            create_vertex_state(&vertex_module),
            create_fragment_state(&fragment_module, false),
            sample_count,
            false,
        );

        let mask_pipeline_descriptor = create_map_render_pipeline_description(
            &pipeline_layout,
            create_vertex_state(&vertex_module),
            create_fragment_state(&fragment_module, true),
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

        let camera = camera::Camera::new((0.0, 5.0, 10.0), cgmath::Deg(-90.0), cgmath::Deg(-20.0));
        let projection = camera::Projection::new(
            surface_config.width,
            surface_config.height,
            cgmath::Deg(45.0),
            0.1,
            100.0,
        );
        let camera_controller = camera::CameraController::new(4.0, 0.4);
        let mut camera_uniform = camera::CameraUniform::new();

        camera_uniform.update_view_proj(&camera, &projection);

        Self {
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
            tile_mask_indices_uniform_buffer,
            tile_mask_range,
            camera,
            projection,
            camera_controller,
            camera_uniform,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
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

    pub fn input(&mut self, event: &DeviceEvent) -> bool {
        let scene = &mut self.scene;
        match event {
            DeviceEvent::Key(KeyboardInput {
                virtual_keycode: Some(key),
                state,
                ..
            }) => match key {
                VirtualKeyCode::PageDown => {
                    println!("PageDown");
                    scene.target_zoom *= 0.8;
                    true
                }
                VirtualKeyCode::PageUp => {
                    println!("PageUp");
                    scene.target_zoom *= 1.25;
                    true
                }
                VirtualKeyCode::Left => {
                    scene.target_scroll.x -= 50.0 / scene.target_zoom;
                    true
                }
                VirtualKeyCode::Right => {
                    scene.target_scroll.x += 50.0 / scene.target_zoom;
                    true
                }
                VirtualKeyCode::Up => {
                    scene.target_scroll.y -= 50.0 / scene.target_zoom;
                    true
                }
                VirtualKeyCode::Down => {
                    scene.target_scroll.y += 50.0 / scene.target_zoom;
                    true
                }
                VirtualKeyCode::A => {
                    scene.target_stroke_width /= 0.8;
                    true
                }
                VirtualKeyCode::Z => {
                    scene.target_stroke_width *= 0.8;
                    true
                }
                _key => self.camera_controller.process_keyboard(*key, *state),
            },
            DeviceEvent::MouseWheel { delta, .. } => {
                self.camera_controller.process_scroll(delta);
                true
            }
            DeviceEvent::Button {
                button: 1, // Left Mouse Button
                state,
            } => {
                //self.mouse_pressed = *state == ElementState::Pressed;
                true
            }
            DeviceEvent::MouseMotion { delta } => {
                //if self.mouse_pressed {
                //    self.camera_controller.process_mouse(delta.0, delta.1);
                //}
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
                bytemuck::cast_slice(&[Globals::new(
                    self.camera_uniform.view_proj,
                    self.camera_uniform.view_position,
                    [self.size.width as f32, self.size.height as f32],
                    scene.scroll.to_array(),
                    scene.zoom,
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
                //pass.set_stencil_reference(0);
                pass.set_index_buffer(
                    self.tile_mask_indices_uniform_buffer.slice(..),
                    wgpu::IndexFormat::Uint16,
                );
                pass.set_vertex_buffer(0, self.tile_mask_vertex_uniform_buffer.slice(..));
                pass.draw_indexed(self.tile_mask_range.clone(), 0, 0..1);
            }
            {
                pass.set_pipeline(&self.render_pipeline);
                pass.set_stencil_reference(1);
                pass.set_index_buffer(
                    self.indices_uniform_buffer.slice(..),
                    wgpu::IndexFormat::Uint16,
                );
                pass.set_vertex_buffer(0, self.vertex_uniform_buffer.slice(..));
                //pass.draw_indexed(self.fill_range.clone(), 0, 0..(self.num_instances as u32));
                pass.draw_indexed(self.tile_stroke_range.clone(), 0, 0..1);
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
        self.camera_uniform
            .update_view_proj(&self.camera, &self.projection);

        // Animate the zoom to match target_zoom
        scene.zoom += (scene.target_zoom - scene.zoom) / 3.0;
        scene.scroll = scene.scroll + (scene.target_scroll - scene.scroll) / 3.0;
        scene.stroke_width =
            scene.stroke_width + (scene.target_stroke_width - scene.stroke_width) / 5.0;

        // Animate the strokes of primitive
        scene.cpu_primitives[STROKE_PRIM_ID as usize].width = scene.stroke_width;
        scene.cpu_primitives[STROKE_PRIM_ID as usize].color = [
            (time_secs * 0.8 - 1.6).sin() * 0.1 + 0.1,
            (time_secs * 0.5 - 1.6).sin() * 0.1 + 0.1,
            (time_secs - 1.6).sin() * 0.1 + 0.1,
            1.0,
        ];

        self.fps_meter.update_and_print()
    }
}
