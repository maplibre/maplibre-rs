mod geom;
mod glyph_tesselation;
mod texture;

use geom::{Mesh, Meshable, Quad, Vertex};
use glyph_tesselation::GlyphBuilder;
use std::collections::HashMap;
use std::io::Error;
use std::mem;
use ttf_parser as ttf;
use wgpu::util::DeviceExt;

use crate::rendering::{Renderable, State};

// A piece of text in the scene, which is styled and moved as a unit
pub struct TextEntity {
    pub text: String,
    pub shaping_info: rustybuzz::GlyphBuffer,
    pub position: cgmath::Vector3<f32>,
    pub rotation: cgmath::Quaternion<f32>,
    pub color: cgmath::Vector3<f32>,
    pub size: f32,
}

pub type TextEntityID = u32;

pub enum TextSystemError {
    FontNotLoaded(&'static str),
    GlyphNotSupported(&'static str),
}

// The values for the instance buffer
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct GlyphInstanceAttributes {
    pub transform: [[f32; 4]; 4],
    pub color: [f32; 3],
}

// Struct of arrays to allow simple instance buffer generation from 'attributes'
// While at the same time allow removing and editing of glyphs that are part of
// a certain text entity
struct GlyphInstances {
    text_entity_ids: Vec<TextEntityID>,
    attributes: Vec<GlyphInstanceAttributes>,
    buffer: Option<wgpu::Buffer>,
}

impl GlyphInstances {
    fn new() -> Self {
        Self {
            text_entity_ids: Vec::new(),
            attributes: Vec::new(),
            buffer: None,
        }
    }

    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<GlyphInstanceAttributes>() as wgpu::BufferAddress,
            // We need to switch from using a step mode of Vertex to Instance
            // This means that our shaders will only change to use the next
            // instance when the shader starts processing a new instance
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // A mat4 takes up 4 vertex slots as it is technically 4 vec4s. We need to define a slot
                // for each vec4. We'll have to reassemble the mat4 in
                // the shader.
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }

    fn add(&mut self, entity_id: TextEntityID, glyph_attributes: GlyphInstanceAttributes) {
        self.text_entity_ids.push(entity_id);
        self.attributes.push(glyph_attributes);
    }

    fn compute_buffer(&mut self, device: &wgpu::Device) {
        self.buffer = Some(
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&self.attributes),
                usage: wgpu::BufferUsages::VERTEX,
            }),
        );
    }

    fn num_instances(&self) -> u32 {
        self.attributes.len() as u32
    }
}

impl TextEntity {
    fn new(
        text: &str,
        face: rustybuzz::Face,
        position: cgmath::Vector3<f32>,
        rotation: cgmath::Quaternion<f32>,
        color: cgmath::Vector3<f32>,
        size: f32,
    ) -> Self {
        let mut buffer = rustybuzz::UnicodeBuffer::new();
        buffer.push_str(text);

        TextEntity {
            text: String::from(text),
            shaping_info: rustybuzz::shape(&face, &[], buffer),
            position,
            rotation,
            color,
            size,
        }
    }
}

pub type FontID = String;

pub struct Font {
    pub font_name: FontID,
    pub font_file_path: String,
    font_data: Vec<u8>,
}

impl Font {
    pub fn new(font_name: FontID, font_path: &str) -> Result<Self, TextSystemError> {
        let font_file_path = String::from(font_path);

        let font_data_res = std::fs::read(&font_file_path);
        if font_data_res.is_err() {
            return Err(TextSystemError::FontNotLoaded("Could not read font file!"));
        }

        let font_data = font_data_res.unwrap();

        Ok(Self {
            font_name,
            font_file_path,
            font_data,
        })
    }

    pub fn get_face(&self) -> rustybuzz::Face {
        // TODO: try using "owningFace" instead and return ref, so we don't recompute the face every time!
        rustybuzz::Face::from_slice(&self.font_data, 0).unwrap()
    }
}

struct GlyphRenderData {
    mesh: Mesh,
    instances: GlyphInstances,
}

// System that takes care of rendering the text
// Offers a simple interface to add / transform / remove bits of text and then render them to the screen
// TODO: cleaner separation of rendering algorithm and text system management (i.e. -> interface that supplies mesh representation and shaders + some factory)
// TODO: add prioritzed collision detection
// TODO: add way to handle duplicate text on different map zoom levels
// TODO: test performance with very dynamic text (i.e. lots of movement / appearing / vanishing of labels on the map, ...)
pub struct SceneTextSystem {
    // The loaded fonts
    fonts: HashMap<FontID, Font>,
    // Cache for triangulated glyphs (vertices are in font coordinate system -> relatively large numbers)
    glyph_mesh_cache: HashMap<ttf::GlyphId, GlyphRenderData>,
    // All texts in the scene
    text_entities: HashMap<TextEntityID, TextEntity>,
    // Internal counter to assign a unique ID to each text in the scene
    next_text_entity_id: TextEntityID,
    // ######### Rendering related stuff #########################################################
    prepass_target_texture: texture::Texture,
    prepass_target_texture_bind_group: wgpu::BindGroup,
    prepass_pipeline: wgpu::RenderPipeline,
    main_pipeline: wgpu::RenderPipeline,
    full_screen_quad: Mesh,
}

impl SceneTextSystem {
    pub fn new(rendering_state: &State) -> Result<Self, Error> {
        let prepass_target_texture = texture::Texture::empty(
            &rendering_state.device,
            rendering_state.config.width,
            rendering_state.config.height,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            Some("textPrepassTarget"),
        );
        let texture_bind_group_layout =
            rendering_state
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            // This should match the filterable field of the
                            // corresponding Texture entry above.
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                    label: Some("text_prepass_texture_bind_group_layout"),
                });

        let prepass_target_texture_bind_group =
            rendering_state
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &texture_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(
                                &prepass_target_texture.view,
                            ),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(
                                &prepass_target_texture.sampler,
                            ),
                        },
                    ],
                    label: Some("text_prepass_texture_bind_group"),
                });

        let shaders = rendering_state
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Text Shaders"),
                source: wgpu::ShaderSource::Wgsl(include_str!("textsystem_shaders.wgsl").into()),
            });

        // PREPASS PIPELINE

        let pre_pass_render_pipeline_layout =
            rendering_state
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Prepass Pipeline Layout"),
                    bind_group_layouts: &[&rendering_state.camera_bind_group_layout],
                    push_constant_ranges: &[],
                });

        let prepass_pipeline =
            rendering_state
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Prepass Pipeline"),
                    layout: Some(&pre_pass_render_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shaders,
                        entry_point: "prepass_vs",
                        buffers: &[Vertex::desc(), GlyphInstances::desc()],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shaders,
                        entry_point: "prepass_fs",
                        targets: &[Some(wgpu::ColorTargetState {
                            format: rendering_state.config.format,
                            blend: Some(wgpu::BlendState {
                                // Overwrite color without alpha blending applied
                                // -> A glyph instance's color will be transferred unmodified through the prepass
                                color: wgpu::BlendComponent {
                                    src_factor: wgpu::BlendFactor::One,
                                    dst_factor: wgpu::BlendFactor::Zero,
                                    operation: wgpu::BlendOperation::Add,
                                },
                                alpha: wgpu::BlendComponent::OVER,
                            }),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: None, // no culling because glyph tesselation yields cw and ccw triangles
                        polygon_mode: wgpu::PolygonMode::Fill,
                        unclipped_depth: false,
                        conservative: false,
                    },
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState {
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    multiview: None,
                });

        // MAIN PASS PIPELINE

        let main_pass_render_pipeline_layout =
            rendering_state
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Text main pass Pipeline Layout"),
                    bind_group_layouts: &[
                        &rendering_state.camera_bind_group_layout,
                        &texture_bind_group_layout,
                    ],
                    push_constant_ranges: &[],
                });

        let main_pipeline =
            rendering_state
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Text main pipeline"),
                    layout: Some(&main_pass_render_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shaders,
                        entry_point: "mainpass_vs",
                        buffers: &[Vertex::desc()],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shaders,
                        entry_point: "mainpass_fs",
                        targets: &[Some(wgpu::ColorTargetState {
                            format: rendering_state.config.format,
                            blend: Some(wgpu::BlendState::REPLACE),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: Some(wgpu::Face::Back),
                        // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                        polygon_mode: wgpu::PolygonMode::Fill,
                        // Requires Features::DEPTH_CLIP_CONTROL
                        unclipped_depth: false,
                        // Requires Features::CONSERVATIVE_RASTERIZATION
                        conservative: false,
                    },
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState {
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    multiview: None,
                });

        let quad = Quad {
            center: (0.0, 0.0, 0.0).into(),
            width: 1.0,
            height: 1.0,
        };
        let full_screen_quad = quad.as_mesh(&rendering_state.device);

        Ok(Self {
            fonts: HashMap::new(),
            glyph_mesh_cache: HashMap::new(),
            text_entities: HashMap::new(),
            next_text_entity_id: 0,
            prepass_target_texture,
            prepass_target_texture_bind_group,
            prepass_pipeline,
            main_pipeline,
            full_screen_quad,
        })
    }

    pub fn load_font(&mut self, id: &FontID, font_path: &str) -> Result<(), TextSystemError> {
        self.fonts
            .insert(id.clone(), Font::new(id.clone(), font_path)?);
        Ok(())
    }

    pub fn add_text_to_scene(
        &mut self,
        rendering_state: &State,
        text: &str,
        base_position: cgmath::Vector3<f32>,
        rotation: cgmath::Quaternion<f32>,
        color: cgmath::Vector3<f32>,
        size: f32,
        font_id: &FontID,
    ) -> Result<(), TextSystemError> {
        let new_text_id = self.next_text_entity_id;

        let font_opt = self.fonts.get(font_id);

        if font_opt.is_none() {
            return Err(TextSystemError::FontNotLoaded(
                "Font must be loaded before adding text that uses it!",
            ));
        }

        let font: &Font = font_opt.unwrap();

        self.text_entities.insert(
            new_text_id,
            TextEntity::new(text, font.get_face(), base_position, rotation, color, size),
        );
        let text_entity = self.text_entities.get(&new_text_id).unwrap(); // This MUST be present, as it was added in the line above.

        // Construct instances for each glyph of the text (optionally create mesh for glyph if needed)
        let infos = text_entity.shaping_info.glyph_infos();
        let posistions = text_entity.shaping_info.glyph_positions();
        let mut glyph_offset = base_position;

        // For each glyph in the layed out word
        for (info, pos) in infos.iter().zip(posistions) {
            let glyph_id = ttf::GlyphId(info.glyph_id.try_into().unwrap()); // ttfparser for some reason wants a u16 ?!

            let mut inserted = true;

            // Create and add glyph mesh to cache if not present
            if !self.glyph_mesh_cache.contains_key(&glyph_id) {
                let mut glyph_builder = GlyphBuilder::new();
                if let Some(bbox) = font.get_face().outline_glyph(glyph_id, &mut glyph_builder) {
                    glyph_builder.finalize(&bbox);
                    self.glyph_mesh_cache.insert(
                        glyph_id,
                        GlyphRenderData {
                            mesh: glyph_builder.as_mesh(&rendering_state.device),
                            instances: GlyphInstances::new(),
                        },
                    );
                } else {
                    // now new mesh -> we don't need and instance infos for it!
                    inserted = false;
                    // TODO: most likely: white space? -> detect that so we can detect actual errors here
                    // return Err(TextSystemError::GlyphNotSupported("Glyph not supported!"));
                }
            }

            if inserted {
                // Construct instance by passing the attributes. Currently only the position changes for each letter.
                // TODO: support different styles for each letter in a text entity
                let glyph_render_data: &mut GlyphRenderData =
                    self.glyph_mesh_cache.get_mut(&glyph_id).unwrap();
                // we don't scale text in the z-direction as it is a 2-D flat object positioned in 3-D
                let glyph_instances: &mut GlyphInstances = &mut glyph_render_data.instances;
                glyph_instances.add(
                    new_text_id,
                    GlyphInstanceAttributes {
                        transform: (cgmath::Matrix4::from_translation(glyph_offset)
                            * cgmath::Matrix4::from_nonuniform_scale(size, size, 1.0)
                            * cgmath::Matrix4::from(rotation))
                        .into(),
                        color: color.into(),
                    },
                );
            }

            // Move offset to position of next letter in the text
            // Apply scaling because advances are in the fonts local coordinate system
            let x_advance = pos.x_advance as f32 * size;
            let y_advance = pos.y_advance as f32 * size;
            glyph_offset += cgmath::Vector3::new(x_advance, y_advance, 0.0);
        }
        self.next_text_entity_id += 1;
        Ok(())
    }

    fn render_prepass(&mut self, encoder: &mut wgpu::CommandEncoder, rendering_state: &State) {
        // Draw all the meshes into the target texture

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("TextSystemPrePass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.prepass_target_texture.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 0.0, // important, otherwise all pixels will pass the uneven winding number test in the main pass...
                    }),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        render_pass.set_pipeline(&self.prepass_pipeline);
        render_pass.set_bind_group(0, &rendering_state.camera_bind_group, &[]);

        for (_, glyph_render_data) in &mut self.glyph_mesh_cache {
            render_pass.set_vertex_buffer(0, glyph_render_data.mesh.vertex_buffer.slice(..));
            glyph_render_data
                .instances
                .compute_buffer(&rendering_state.device);
            render_pass.set_vertex_buffer(
                1,
                glyph_render_data
                    .instances
                    .buffer
                    .as_ref()
                    .unwrap()
                    .slice(..),
            );

            render_pass.set_index_buffer(
                glyph_render_data.mesh.index_buffer.slice(..),
                wgpu::IndexFormat::Uint16,
            );
            render_pass.draw_indexed(
                0..(glyph_render_data.mesh.num_indices as u32),
                0,
                0..glyph_render_data.instances.num_instances(),
            );
        }
    }

    fn render_mainpass(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        rendering_state: &State,
        output_texture: wgpu::SurfaceTexture,
    ) -> wgpu::SurfaceTexture {
        let view = output_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("TextSystemMainPass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 1.0,
                        g: 1.0,
                        b: 1.0,
                        a: 1.0,
                    }),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        render_pass.set_pipeline(&self.main_pipeline);
        render_pass.set_bind_group(0, &rendering_state.camera_bind_group, &[]);
        render_pass.set_bind_group(1, &self.prepass_target_texture_bind_group, &[]);

        let mesh = &self.full_screen_quad;
        render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(0..4));
        render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        let num_indices = mesh.num_indices as u32;
        render_pass.draw_indexed(0..num_indices, 0, 0..1);

        output_texture
    }
}

impl Renderable for SceneTextSystem {
    fn render(
        &mut self,
        rendering_state: &State,
        output_texture: wgpu::SurfaceTexture,
    ) -> wgpu::SurfaceTexture {
        let mut encoder =
            rendering_state
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        // Prepass to fill in the glyph meshes into the texture
        self.render_prepass(&mut encoder, rendering_state);

        // Actual pass to flip the pixels (and compute anti-aliasing?)
        let output = self.render_mainpass(&mut encoder, rendering_state, output_texture);

        rendering_state
            .queue
            .submit(std::iter::once(encoder.finish()));
        output
    }
}
