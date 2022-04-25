use crate::geom::{Mesh, Meshable};
use std::collections::HashMap;
use ttf_parser as ttf;
use std::io::{Error};

pub enum TextSystemError {
    FontNotLoaded(&'static str),
    GlyphNotSupported(&'static str)
}

// The values for the instance buffer
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct GlyphInstanceAttributes {
    pub transform: [[f32; 4]; 4],
    pub color: [f32; 3],
}

// A piece of text in the scene, which is styled and moved as a unit
struct TextEntity {
    pub text: String,
    pub shaping_info: rustybuzz::GlyphBuffer,
    pub position: cgmath::Vector3<f32>,
    pub rotation: cgmath::Quaternion<f32>,
    pub color: cgmath::Vector3<f32>,
    pub size: f32,
}

pub type TextEntityID u32;

// Struct of arrays to allow simple instance buffer generation from 'attributes'
// While at the same time allow removing and editing of glyphs that are part of
// a certain text entity
struct GlyphInstances {
    text_entity_ids: std::Vec<TextEntityID>,
    attributes: std::Vec<GlyphInstanceAttributes>,
}

impl GlyphInstances {
    fn new() -> Self {
        Self {
            text_entity_ids: Vec::new(),
            attributes: HashMap::new()
        }
    }

    fn add(&mut self, entity_id: TextEntityID, glyph_attributes: GlyphInstanceAttributes) {
        self.text_entity_ids.push(entity_id);
        self.attributes.push(glyph_attributes);
    }
}

impl TextEntity {
    fn new(
        text: &str,
        face: &rustybuzz::Face,
        position: cgmath::Vector3<f32>,
        rotation: cgmath::Quaternion<f32>,
        color: cgmath::Vector3<f32>,
        size: f32
    ) -> Self {
        let mut buffer = rustybuzz::UnicodeBuffer::new();
        buffer.push_str(text);

        TextEntity {
            text: String::from(text),
            shaping_info: rustybuzz::shape(face, &[], buffer),
            position,
            color,
            size
        }
    }
}

pub type FontID String;

pub struct Font {
    pub font_name: FontID,
    pub font_file_path: String,
    font_data: Vec<u8>,
    font_face: rustybuzz::Face,
}

impl Font {
    pub fn new(font_path: &str) -> Self {
        let font_file_path = String::from(font_path);
        let font_data = std::fs::read(&font_file_path)?;
        let font_face = rustybuzz::Face::from_slice(&font_data, 0)?;

        Self {
            font_file_path,
            font_data,
            font_face,
        }
    }
}


// System that takes care of rendering the text
// Offers a simple interface to add / transform / remove bits of text and then render them to the screen
// TODO: cleaner separation of rendering algorithm and text system management (i.e. -> interface that supplies mesh representation and shaders + some factory)
// TODO: This will likely not perform well with very dynamic text (i.e. lots of movement / appearing / vanishing of labels on the map, ...)
pub struct SceneTextSystem {
    device: &wgpu::Device,

    // The loaded fonts
    fonts: HashMap<FontID, Font>,

    // Cache for triangulated glyphs (vertices are in font coordinate system -> relatively large numbers)
    glyph_mesh_cache: HashMap<ttf::GlyphId, Mesh>,

    // Information (transform, color, ...) for each instance of a glyph in the scene
    glyph_instance_map: HashMap<ttf::GlyphId, GlyphInstances>,

    // All texts in the scene
    text_entities: HashMap<TextEntityID, TextEntity>,

    // Internal counter to assign a unique ID to each text in the scene
    next_text_entity_id: TextEntityID

    // ######### Rendering related stuff #########################################################
    // TODO: abstract into separate class
    size: winit::dpi::PhysicalSize<u32>,
    camera_bind_group_layout: &wgpu::BindGroupLayout,
    config: &wgpu::SurfaceConfiguration,

    prepass_target_texture: texture::Texture,
    prepass_target_texture_bind_group: wgpu::BindGroup,
    prepass_pipeline: wgpu::RenderPipeline,
    main_pipeline: wgpu::RenderPipeline,
}

impl SceneTextSystem {
    pub fn new(
        device: &wgpu::Device
        config: &wgpu::SurfaceConfiguration,
        camera_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Result<Self, Error> {

        let prepass_target_texture = texture::Texture::empty(
            &device,
            config.width,
            config.height,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            Some("textPrepassTarget"),
        );
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&prepass_target_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&prepass_target_texture.sampler),
                    },
                ],
                label: Some("text_prepass_texture_bind_group"),
            });

        let shaders = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("Text Shaders"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders.wgsl").into()),
        });

        // PREPASS PIPELINE

        let pre_pass_render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Prepass Pipeline Layout"),
                bind_group_layouts: &[camera_bind_group_layout],
                push_constant_ranges: &[],
            });

        let prepass_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Prepass Pipeline"),
            layout: Some(&pre_pass_render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shaders,
                entry_point: "prepass_vs",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shaders,
                entry_point: "prepass_fs",
                targets: &[wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                }],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None, // no culling because glyph tesselation yields cw and ccw triangles
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

        // MAIN PASS PIPELINE

        let main_pass_render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Text main pass Pipeline Layout"),
                bind_group_layouts: &[camera_bind_group_layout, &texture_bind_group_layout],
                push_constant_ranges: &[],
            });

        let main_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
                targets: &[wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                }],
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

        Self {
            device,
            fonts: HashMap::new(),
            glyph_mesh_cache: HashMap::new(),
            glyph_instance_map: HashMap::new(),
            text_entities: HashMap::new(),
            next_text_entity_id: 0u
        }
    }

    pub fn add_text_to_scene(
        &mut self,
        text: &str,
        base_position: &cgmath::Vector3<f32>,
        rotation: &cgmath::Quaternion<f32>,
        color: &cgmath::Vector3<f32>,
        size: f32,
        font_id: FontID,
    ) -> Result<(), TextSystemError> {

        let new_text_id = self.next_text_entity_id;

        let font_opt = self.fonts.get(font_id);

        if font_opt.is_none() {
            return Err(TextSystemError::FontNotLoaded(format!("Font {} must be loaded before adding text that uses it!", font_id)));
        }

        let font : Font = font_opt.unwrap();
        
        self.text_entities.insert(new_entity_id, TextEntity::new(text, &font.font_face, base_position, color, size));

        // Construct instances for each glyph of the text (optionally create mesh for glyph if needed)
        let infos = glyph_buffer.glyph_infos();
        let posistions = glyph_buffer.glyph_positions();
        let mut glyph_offset = base_position;

        // For each glyph in the layed out word
        for (info, pos) in infos.iter().zip(posistions) {
            let glyph_id = ttf::GlyphId(info.glyph_id.try_into().unwrap()); // ttfparser for some reason wants a u16 ?!

            // Create and add glyph mesh to cache if not present
            if !self.glyph_mesh_cache.contains_key(glyph_id) {
                let mut glyph_builder = GlyphBuilder::new();
                if let Some(bbox) = font.font_face.outline_glyph(glyph_id, glyph_builder) {
                    glyph_builder.finalize(&bbox);
                    self.glyph_mesh_cache.insert(glyph_id, glyph_builder.as_mesh(self.device));
                } else {
                    return Err(TextSystemError::GlyphNotSupported("Glyph not supported!"));
                }
            }

            // Construct instance by passing the attributes. Currently only the position changes for each letter.
            // TODO: support different styles for each letter in a text entity
            let glyph_instances : &mut GlyphInstances = self.glyph_instance_map.entry(glyph_id).or_insert(GlyphInstances::new());

            let scale = [size, size, 0.0], // we don't scale text in the z-direction as it is a 2-D flat object positioned in 3-D
            glyph_instances.add(new_text_id, GlyphInstanceAttributes {
                transform: (cgmath::Matrix4::from_translation(glyph_offset) * cgmath::Matrix4::from(scale) * cgmath::Matrix4::from(rotation)).into()
                color: color.into(),
            })

            // Move offset to position of next letter in the text
            let x_advance = pos.x_advance as f32;
            let y_advance = pos.y_advance as f32;
            glyph_offset += cgmath::Vector3::new(x_advance, y_advance, 0.0);
        }
    
        self.next_text_entity_id += 1;
        Ok(())
    }

    fn render_prepass(&mut self, encoder: &mut wgpu::CommandEncoder) {
        // Draw all the meshes into the target texture

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("PrePass"),
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: &self.prepass_target_texture.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 1.0,
                        g: 1.0,
                        b: 1.0,
                        a: 0.0, // important, otherwise all pixels will pass the uneven winding number test in the main pass...
                    }),
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });

        render_pass.set_pipeline(&self.prepass_pipeline);
        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);

        // TODO: render each glyph instanced (create instance buffers anew, because text in the scene in its positions might have changed!)
        /*
        for mesh in &self.meshes {
            render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(0..4));
            render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            let num_indices = mesh.num_indices as u32;
            render_pass.draw_indexed(0..num_indices, 0, 0..1);
        }
        */
    }

    fn render_mainpass(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        surface: &wgpu::Surface
    ) -> Result<wgpu::SurfaceTexture, wgpu::SurfaceError> {
        let output = surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("MainPass"),
            color_attachments: &[wgpu::RenderPassColorAttachment {
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
            }],
            depth_stencil_attachment: None,
        });

        render_pass.set_pipeline(&self.main_pipeline);
        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
        render_pass.set_bind_group(1, &self.prepass_target_texture_bind_group, &[]);

        let mesh = &self.full_screen_quad;
        render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(0..4));
        render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        let num_indices = mesh.num_indices as u32;
        render_pass.draw_indexed(0..num_indices, 0, 0..1);

        Ok(output)
    }

    pub fn render(&mut self, surface: &wgpu::Surface) -> Result<wgpu::SurfaceTexture, wgpu::SurfaceError> {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // Prepass to fill in the glyph meshes into the texture
        self.render_prepass(&mut encoder, surface);

        // Actual pass to flip the pixels (and compute anti-aliasing?)
        let output = self.render_mainpass(&mut encoder)?;

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        
        output
    }
}
