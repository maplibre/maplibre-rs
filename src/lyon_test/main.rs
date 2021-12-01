use std::time::{Duration, Instant};

use futures::executor::block_on;
use lyon::tessellation::VertexBuffers;
use vector_tile::parse_tile;
use wgpu::util::DeviceExt;
use winit::event_loop::EventLoop;
use winit::window::Window;

use crate::piplines::create_map_render_pipeline_description;
use crate::scene::SceneParams;
use crate::shader::{
    create_fragment_module_descriptor, create_map_fragment_state, create_map_vertex_state,
    create_vertex_module_descriptor,
};
use crate::shader_ffi::*;
use crate::tesselation::{RustLogo, Tesselated};

mod piplines;
mod scene;
mod shader;
mod shader_ffi;
mod tesselation;
mod tile_downloader;

const PRIM_BUFFER_LEN: usize = 256;

const DEFAULT_WINDOW_WIDTH: f32 = 800.0;
const DEFAULT_WINDOW_HEIGHT: f32 = 800.0;

/// Creates a texture that uses MSAA and fits a given swap chain
fn create_multisampled_framebuffer(
    device: &wgpu::Device,
    desc: &wgpu::SurfaceConfiguration,
    sample_count: u32,
) -> wgpu::TextureView {
    let multisampled_frame_descriptor = &wgpu::TextureDescriptor {
        label: Some("Multisampled frame descriptor"),
        size: wgpu::Extent3d {
            width: desc.width,
            height: desc.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count,
        dimension: wgpu::TextureDimension::D2,
        format: desc.format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
    };

    device
        .create_texture(multisampled_frame_descriptor)
        .create_view(&wgpu::TextureViewDescriptor::default())
}

fn main() {
    env_logger::init();
    println!("== wgpu example ==");
    println!("Controls:");
    println!("  Arrow keys: scrolling");
    println!("  PgUp/PgDown: zoom in/out");
    println!("  a/z: increase/decrease the stroke width");

    // Number of samples for anti-aliasing
    // Set to 1 to disable
    let sample_count = 4;

    let num_instances: u32 = 1;

    let stroke_prim_id = 0;
    let fill_prim_id = 1;

    let mut geometry: VertexBuffers<GpuVertex, u16> = VertexBuffers::new();

    let (stroke_range, fill_range) = if true {
        let tile =
            parse_tile("test-data/12-2176-1425.pbf").expect("failed loading tile");
        (
            0..tile.tesselate_stroke(&mut geometry, stroke_prim_id),
            0..0,
        )
    } else {
        let logo = RustLogo();
        let max_index = logo.tesselate_stroke(&mut geometry, stroke_prim_id);
        (
            0..max_index,
            max_index..max_index + logo.tesselate_fill(&mut geometry, fill_prim_id),
        )
    };

    let mut cpu_primitives = [Primitive {
        color: [1.0, 0.0, 0.0, 1.0],
        z_index: 0,
        width: 0.0,
        translate: [0.0, 0.0],
        angle: 0.0,
        ..Primitive::DEFAULT
    }; PRIM_BUFFER_LEN];

    // Stroke primitive
    cpu_primitives[stroke_prim_id as usize] = Primitive {
        color: [0.0, 0.0, 0.0, 1.0],
        z_index: num_instances as i32 + 2,
        width: 1.0,
        ..Primitive::DEFAULT
    };
    // Main fill primitive
    cpu_primitives[fill_prim_id as usize] = Primitive {
        color: [1.0, 1.0, 1.0, 1.0],
        z_index: num_instances as i32 + 1,
        ..Primitive::DEFAULT
    };

    let mut scene = SceneParams::DEFAULT;

    let event_loop = EventLoop::new();
    let window = Window::new(&event_loop).unwrap();

    // create an instance
    let instance = wgpu::Instance::new(wgpu::Backends::all());

    // create an surface
    let surface = unsafe { instance.create_surface(&window) };

    // create an adapter
    let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::LowPower,
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
    }))
    .unwrap();
    // create a device and a queue
    let (device, queue) = block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: None,
            features: wgpu::Features::default(),
            limits: wgpu::Limits::default(),
        },
        None,
    ))
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

    let prim_buffer_byte_size = (PRIM_BUFFER_LEN * std::mem::size_of::<Primitive>()) as u64;
    let globals_buffer_byte_size = std::mem::size_of::<Globals>() as u64;

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
    let mut render_pipeline_descriptor = create_map_render_pipeline_description(
        &pipeline_layout,
        create_map_vertex_state(&vertex_module),
        create_map_fragment_state(&fragment_module),
        sample_count,
    );
    let render_pipeline = device.create_render_pipeline(&render_pipeline_descriptor);

    // TODO: this isn't what we want: we'd need the equivalent of VK_POLYGON_MODE_LINE,
    // but it doesn't seem to be exposed by wgpu?
    render_pipeline_descriptor.primitive.topology = wgpu::PrimitiveTopology::LineList;

    let size = window.inner_size();

    let mut surface_desc = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: wgpu::TextureFormat::Bgra8UnormSrgb,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Mailbox,
    };

    let mut multisampled_render_target = None;

    surface.configure(&device, &surface_desc);

    let mut depth_texture_view = None;

    let start = Instant::now();
    let mut next_report = start + Duration::from_secs(1);
    let mut frame_count: u32 = 0;
    let mut time_secs: f32 = 0.0;

    window.request_redraw();

    event_loop.run(move |event, _, control_flow| {
        if !scene.update_inputs(event, &window, control_flow) {
            // keep polling inputs.
            return;
        }

        if scene.size_changed {
            scene.size_changed = false;
            let physical = scene.window_size;
            surface_desc.width = physical.width;
            surface_desc.height = physical.height;
            surface.configure(&device, &surface_desc);

            let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Depth texture"),
                size: wgpu::Extent3d {
                    width: surface_desc.width,
                    height: surface_desc.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            });

            depth_texture_view =
                Some(depth_texture.create_view(&wgpu::TextureViewDescriptor::default()));

            multisampled_render_target = if sample_count > 1 {
                Some(create_multisampled_framebuffer(
                    &device,
                    &surface_desc,
                    sample_count,
                ))
            } else {
                None
            };
        }

        // TODO: Without this we are not able to close the window
        if !scene.render {
            return;
        }

        scene.render = false;

        let frame = match surface.get_current_texture() {
            Ok(texture) => texture,
            Err(e) => {
                println!("Swap-chain error: {:?}", e);
                return;
            }
        };

        let frame_view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Encoder"),
        });

        /*        cpu_primitives[fill_prim_id as usize].color = [
            (time_secs * 0.8 - 1.6).sin() * -0.1 + 0.1,
            (time_secs * 0.5 - 1.6).sin() * -0.1 + 0.1,
            (time_secs - 1.6).sin() * -0.1 + 0.1,
            1.0,
        ];*/

        cpu_primitives[stroke_prim_id as usize].width = scene.stroke_width;
        cpu_primitives[stroke_prim_id as usize].color = [
            (time_secs * 0.8 - 1.6).sin() * 0.1 + 0.1,
            (time_secs * 0.5 - 1.6).sin() * 0.1 + 0.1,
            (time_secs - 1.6).sin() * 0.1 + 0.1,
            1.0,
        ];

        for idx in 2..(num_instances + 1) {
            cpu_primitives[idx as usize].translate = [
                (time_secs * 0.05 * idx as f32).sin() * (100.0 + idx as f32 * 10.0),
                (time_secs * 0.1 * idx as f32).sin() * (100.0 + idx as f32 * 10.0),
            ];
        }

        queue.write_buffer(
            &globals_uniform_buffer,
            0,
            bytemuck::cast_slice(&[Globals {
                resolution: [
                    scene.window_size.width as f32,
                    scene.window_size.height as f32,
                ],
                zoom: scene.zoom,
                scroll_offset: scene.scroll.to_array(),
                _pad: 0.0,
            }]),
        );

        queue.write_buffer(
            &prims_uniform_buffer,
            0,
            bytemuck::cast_slice(&cpu_primitives),
        );

        {
            // A resolve target is only supported if the attachment actually uses anti-aliasing
            // So if sample_count == 1 then we must render directly to the surface's buffer
            let color_attachment = if let Some(msaa_target) = &multisampled_render_target {
                wgpu::RenderPassColorAttachment {
                    view: msaa_target,
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
                    view: depth_texture_view.as_ref().unwrap(),
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

            pass.set_pipeline(&render_pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.set_index_buffer(indices_uniform_buffer.slice(..), wgpu::IndexFormat::Uint16);
            pass.set_vertex_buffer(0, vertex_uniform_buffer.slice(..));

            pass.draw_indexed(fill_range.clone(), 0, 0..(num_instances as u32));
            pass.draw_indexed(stroke_range.clone(), 0, 0..1);
        }

        queue.submit(Some(encoder.finish()));
        frame.present();

        frame_count += 1;
        let now = Instant::now();
        time_secs = (now - start).as_secs_f32();
        if now >= next_report {
            println!("{} FPS", frame_count);
            frame_count = 0;
            next_report = now + Duration::from_secs(1);
        }
    });
}
