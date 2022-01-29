use wgpu::{
    ColorTargetState, Device, FragmentState, ShaderModule, VertexBufferLayout, VertexState,
};

use crate::coords::WorldCoords;
use bytemuck_derive::{Pod, Zeroable};
use cgmath::SquareMatrix;

pub type Vec2f32 = [f32; 2];
pub type Vec3f32 = [f32; 3];
pub type Vec4f32 = [f32; 4];
pub type Mat4x4f32 = [Vec4f32; 4];

impl From<WorldCoords> for Vec3f32 {
    fn from(world_coords: WorldCoords) -> Self {
        [
            world_coords.x as f32,
            world_coords.y as f32,
            world_coords.z as f32,
        ]
    }
}

pub struct FragmentShaderState {
    source: &'static str,
    targets: &'static [ColorTargetState],
    module: Option<ShaderModule>,
}

pub struct VertexShaderState {
    source: &'static str,
    buffers: &'static [VertexBufferLayout<'static>],
    module: Option<ShaderModule>,
}

impl FragmentShaderState {
    pub const fn new(source: &'static str, targets: &'static [ColorTargetState]) -> Self {
        Self {
            source,
            targets,
            module: None,
        }
    }

    pub fn create_fragment_state(&mut self, device: &Device) -> FragmentState {
        self.module = Some(device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("fragment shader"),
            source: wgpu::ShaderSource::Wgsl(self.source.into()),
        }));

        wgpu::FragmentState {
            module: self.module.as_ref().unwrap(),
            entry_point: "main",
            targets: self.targets,
        }
    }
}

impl VertexShaderState {
    pub const fn new(
        source: &'static str,
        buffers: &'static [VertexBufferLayout<'static>],
    ) -> Self {
        Self {
            source,
            buffers,
            module: None,
        }
    }

    pub fn create_vertex_state(&mut self, device: &Device) -> VertexState {
        self.module = Some(device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("vertex shader"),
            source: wgpu::ShaderSource::Wgsl(self.source.into()),
        }));

        wgpu::VertexState {
            module: self.module.as_ref().unwrap(),
            entry_point: "main",
            buffers: self.buffers,
        }
    }
}

pub mod tile {
    use super::{ShaderTileMetadata, ShaderVertex};
    use crate::platform::COLOR_TEXTURE_FORMAT;
    use crate::render::shaders::ShaderFeatureStyle;

    use super::{FragmentShaderState, VertexShaderState};

    pub const VERTEX: VertexShaderState = VertexShaderState::new(
        include_str!("tile.vertex.wgsl"),
        &[
            // vertex data
            wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<ShaderVertex>() as u64,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[
                    // position
                    wgpu::VertexAttribute {
                        offset: 0,
                        format: wgpu::VertexFormat::Float32x2,
                        shader_location: 0,
                    },
                    // normal
                    wgpu::VertexAttribute {
                        offset: wgpu::VertexFormat::Float32x2.size(),
                        format: wgpu::VertexFormat::Float32x2,
                        shader_location: 1,
                    },
                ],
            },
            // tile metadata
            wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<ShaderTileMetadata>() as u64,
                step_mode: wgpu::VertexStepMode::Instance,
                attributes: &[
                    // translate
                    wgpu::VertexAttribute {
                        offset: 0,
                        format: wgpu::VertexFormat::Float32x4,
                        shader_location: 4,
                    },
                    wgpu::VertexAttribute {
                        offset: 1 * wgpu::VertexFormat::Float32x4.size(),
                        format: wgpu::VertexFormat::Float32x4,
                        shader_location: 5,
                    },
                    wgpu::VertexAttribute {
                        offset: 2 * wgpu::VertexFormat::Float32x4.size(),
                        format: wgpu::VertexFormat::Float32x4,
                        shader_location: 6,
                    },
                    wgpu::VertexAttribute {
                        offset: 3 * wgpu::VertexFormat::Float32x4.size(),
                        format: wgpu::VertexFormat::Float32x4,
                        shader_location: 7,
                    },
                ],
            },
            // vertex style
            wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<ShaderFeatureStyle>() as u64,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[
                    // color
                    wgpu::VertexAttribute {
                        offset: 0,
                        format: wgpu::VertexFormat::Float32x4,
                        shader_location: 8,
                    },
                ],
            },
        ],
    );

    pub const FRAGMENT: FragmentShaderState = FragmentShaderState::new(
        include_str!("tile.fragment.wgsl"),
        &[wgpu::ColorTargetState {
            format: COLOR_TEXTURE_FORMAT,
            /*blend: Some(wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::SrcAlpha,
                    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::SrcAlpha,
                    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                    operation: wgpu::BlendOperation::Add,
                },
            }),*/
            blend: None,
            write_mask: wgpu::ColorWrites::ALL,
        }],
    );
}

pub mod tile_mask {
    use super::ShaderTileMaskInstance;
    use crate::platform::COLOR_TEXTURE_FORMAT;
    use crate::render::options::DEBUG_STENCIL_PATTERN;
    use wgpu::ColorWrites;

    use super::{FragmentShaderState, VertexShaderState};

    pub const VERTEX: VertexShaderState = VertexShaderState::new(
        include_str!("tile_mask.vertex.wgsl"),
        &[wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<ShaderTileMaskInstance>() as u64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // offset position
                wgpu::VertexAttribute {
                    offset: 0,
                    format: wgpu::VertexFormat::Float32x2,
                    shader_location: 4,
                },
                // target_width
                wgpu::VertexAttribute {
                    offset: 1 * wgpu::VertexFormat::Float32x2.size(),
                    format: wgpu::VertexFormat::Float32,
                    shader_location: 5,
                },
                // target_height
                wgpu::VertexAttribute {
                    offset: 1 * wgpu::VertexFormat::Float32x2.size()
                        + wgpu::VertexFormat::Float32.size(),
                    format: wgpu::VertexFormat::Float32,
                    shader_location: 6,
                },
                // debug_color
                wgpu::VertexAttribute {
                    offset: 1 * wgpu::VertexFormat::Float32x2.size()
                        + 2 * wgpu::VertexFormat::Float32.size(),
                    format: wgpu::VertexFormat::Float32x4,
                    shader_location: 7,
                },
            ],
        }],
    );

    pub const FRAGMENT: FragmentShaderState = FragmentShaderState::new(
        include_str!("tile_mask.fragment.wgsl"),
        &[wgpu::ColorTargetState {
            format: COLOR_TEXTURE_FORMAT,
            blend: None,
            write_mask: mask_write_mask(),
        }],
    );

    pub const fn mask_write_mask() -> ColorWrites {
        if DEBUG_STENCIL_PATTERN {
            wgpu::ColorWrites::ALL
        } else {
            wgpu::ColorWrites::empty()
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct ShaderCamera {
    view_proj: Mat4x4f32,   // 64 bytes
    view_position: Vec4f32, // 16 bytes
}

impl ShaderCamera {
    pub fn new(view_proj: Mat4x4f32, view_position: Vec4f32) -> Self {
        Self {
            view_position,
            view_proj,
        }
    }
}

impl Default for ShaderCamera {
    fn default() -> Self {
        Self {
            view_position: [0.0; 4],
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct ShaderGlobals {
    camera: ShaderCamera,
}

impl ShaderGlobals {
    pub fn new(camera_uniform: ShaderCamera) -> Self {
        Self {
            camera: camera_uniform,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct ShaderVertex {
    pub position: Vec2f32,
    pub normal: Vec2f32,
}

impl ShaderVertex {
    pub fn new(position: Vec2f32, normal: Vec2f32) -> Self {
        Self { position, normal }
    }
}

impl Default for ShaderVertex {
    fn default() -> Self {
        ShaderVertex::new([0.0, 0.0], [0.0, 0.0])
    }
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct ShaderTileMaskInstance {
    pub position: Vec2f32,
    pub target_width: f32,
    pub target_height: f32,
    pub debug_color: Vec4f32,
}

impl ShaderTileMaskInstance {
    pub fn new(
        position: Vec2f32,
        target_width: f32,
        target_height: f32,
        debug_color: Vec4f32,
    ) -> Self {
        Self {
            position,
            target_width,
            target_height,
            debug_color,
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct ShaderFeatureStyle {
    pub color: Vec4f32,
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct ShaderTileMetadata {
    pub transform: Mat4x4f32,
}

impl ShaderTileMetadata {
    pub fn new(transform: Mat4x4f32) -> Self {
        Self { transform }
    }
}
