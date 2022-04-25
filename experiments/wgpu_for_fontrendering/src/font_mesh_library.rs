use crate::geom::{Mesh, Meshable};
use std::collections::HashMap;
use ttf_parser as ttf;
use std::io::{Error};

pub enum TextSystemError {
    GlyphNotSupported(&'static str)
}

pub struct GlyphMeshCache {
    data: std::HashMap<ttf::GlyphId, Mesh>,
    device: &wgpu::Device,
}

impl GlyphMeshCache {
    fn new(device: &wgpu::Device) -> Self {
        Self {
            data: std::HashMap::new(),
            device,
        }
    }

    // Constructs a mesh and cashes it under the key
    // Precondition: the key must not have been cashed before
    pub fn insert(&mut self, key: ttf::GlyphId, mesh_factory: &Meshable, device: &wgpu::Device) {
        assert_eq!(self.data.insert(key, mesh_factory.as_mesh(device)));
    }

    pub fn contains_glyph(&self, glyph: ttf::GlyphId) -> bool {
        self.data.contains_key(glyph)
    }

    pub fn get_mesh(&self, glyph: ttf::GlyphId) -> Option<&Mesh> {
        self.data.get(glyph)
    }
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
            shaping_info: rustybuzz::shape(&face, &[], buffer),
            position,
            color,
            size
        }
    }
}

// System that takes care of rendering the text
// TODO: move render passes for the text in the scene into this system class, as that makes more sense!
pub struct SceneTextSystem {
    device: &wgpu::Device,
    font_file_path: String,
    font_data: Vec<u8>,
    font_face: rustybuzz::Face,
    glyph_meshes: GlyphMeshCache,
    glyph_instance_map: HashMap<ttf::GlyphId, GlyphInstances>,
    text_entities: HashMap<TextEntityID, TextEntity>
    next_text_entity_id: TextEntityID
}

impl SceneTextSystem {
    pub fn new(device: &wgpu::Device, font_path: &str) -> Result<Self, Error> {
        let font_file_path = String::from(font_path);
        let font_data = std::fs::read(&font_file_path)?;
        let font_face = rustybuzz::Face::from_slice(&font_data, 0)?;

        Self {
            device,
            font_file_path,
            font_data,
            font_face,
            glyph_meshes: GlyphMeshCache::new(device),
            glyph_instance_map: HashMap::new(),
            text_entities: HashMap::new(),
            next_text_entity_id: 0u
        }
    }

    pub fn add_text_to_scene(
        &mut self,
        text: &str,
        base_position: cgmath::Vector3<f32>,
        rotation: cgmath::Quaternion<f32>,
        color: cgmath::Vector3<f32>,
        size: f32,
    ) -> Result<(), TextSystemError> {

        let new_text_id = self.next_text_entity_id;
        self.text_entities.insert(new_entity_id, TextEntity::new(text, self.font_face, base_position, color, size));

        // Construct instances for each glyph of the text (optionally create mesh for glyph if needed)
        let infos = glyph_buffer.glyph_infos();
        let posistions = glyph_buffer.glyph_positions();
        let mut glyph_offset = base_position;

        // For each glyph in the layed out word
        for (info, pos) in infos.iter().zip(posistions) {
            let glyph_id = ttf::GlyphId(info.glyph_id.try_into().unwrap()); // ttfparser for some reason wants a u16 ?!

            // Cerate and add glyph mesh to cache if not present
            if !self.glyph_meshes.contains_glyph(glyph_id) {
                let mut glyph_builder = GlyphBuilder::new();
                if let Some(glyph_mesh) = self.font_face.outline_glyph(glyph_id, glyph_builder) {
                    self.glyph_meshes.insert(glyph_id, glyph_mesh, self.device);
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

    pub fn render(render_pass: &wgpu::RenderPass) {
        // TODO: render each glyph instanced (create instance buffers anew, because text in the scene in its positions might have changed!)
    }
}
