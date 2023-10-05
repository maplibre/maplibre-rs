#![allow(clippy::identity_op)]

use bytemuck_derive::{Pod, Zeroable};
use cgmath::SquareMatrix;

use crate::{
    coords::WorldCoords,
    render::resource::{FragmentState, VertexBufferLayout, VertexState},
};

pub type Vec2f32 = [f32; 2];
pub type Vec3f32 = [f32; 3];
pub type Vec4f32 = [f32; 4];
pub type Mat4x4f32 = [Vec4f32; 4];

impl From<WorldCoords> for Vec3f32 {
    fn from(world_coords: WorldCoords) -> Self {
        [world_coords.x as f32, world_coords.y as f32, 0.0]
    }
}

pub trait Shader {
    fn describe_vertex(&self) -> VertexState;
    fn describe_fragment(&self) -> FragmentState;
}

pub struct TileMaskShader {
    pub format: wgpu::TextureFormat,
    pub draw_colors: bool,
    pub debug_lines: bool,
}

impl Shader for TileMaskShader {
    fn describe_vertex(&self) -> VertexState {
        VertexState {
            source: if self.debug_lines {
                include_str!("tile_debug.vertex.wgsl")
            } else {
                include_str!("tile_mask.vertex.wgsl")
            },
            entry_point: "main",
            buffers: vec![VertexBufferLayout {
                array_stride: std::mem::size_of::<ShaderTileMetadata>() as u64,
                step_mode: wgpu::VertexStepMode::Instance,
                attributes: vec![
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
                    // zoom factor
                    wgpu::VertexAttribute {
                        offset: 4 * wgpu::VertexFormat::Float32x4.size(),
                        format: wgpu::VertexFormat::Float32,
                        shader_location: 9,
                    },
                ],
            }],
        }
    }

    fn describe_fragment(&self) -> FragmentState {
        FragmentState {
            source: include_str!("basic.fragment.wgsl"),
            entry_point: "main",
            targets: vec![Some(wgpu::ColorTargetState {
                format: self.format,
                blend: None,
                write_mask: if self.draw_colors {
                    wgpu::ColorWrites::ALL
                } else {
                    wgpu::ColorWrites::empty()
                },
            })],
        }
    }
}

pub struct VectorTileShader {
    pub format: wgpu::TextureFormat,
}

impl Shader for VectorTileShader {
    fn describe_vertex(&self) -> VertexState {
        VertexState {
            source: include_str!("tile.vertex.wgsl"),
            entry_point: "main",
            buffers: vec![
                // vertex data
                VertexBufferLayout {
                    array_stride: std::mem::size_of::<ShaderVertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: vec![
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
                VertexBufferLayout {
                    array_stride: std::mem::size_of::<ShaderTileMetadata>() as u64,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: vec![
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
                        // zoom_factor
                        wgpu::VertexAttribute {
                            offset: 4 * wgpu::VertexFormat::Float32x4.size(),
                            format: wgpu::VertexFormat::Float32,
                            shader_location: 9,
                        },
                    ],
                },
                // layer metadata
                VertexBufferLayout {
                    array_stride: std::mem::size_of::<ShaderLayerMetadata>() as u64,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: vec![
                        // z_index
                        wgpu::VertexAttribute {
                            offset: 0,
                            format: wgpu::VertexFormat::Float32,
                            shader_location: 10,
                        },
                    ],
                },
                // features
                VertexBufferLayout {
                    array_stride: std::mem::size_of::<ShaderFeatureStyle>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: vec![
                        // color
                        wgpu::VertexAttribute {
                            offset: 0,
                            format: wgpu::VertexFormat::Float32x4,
                            shader_location: 8,
                        },
                    ],
                },
            ],
        }
    }

    fn describe_fragment(&self) -> FragmentState {
        FragmentState {
            source: include_str!("basic.fragment.wgsl"),
            entry_point: "main",
            targets: vec![Some(wgpu::ColorTargetState {
                format: self.format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
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
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct ShaderFeatureStyle {
    pub color: Vec4f32,
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct ShaderLayerMetadata {
    pub z_index: f32,
}

impl ShaderLayerMetadata {
    pub fn new(z_index: f32) -> Self {
        Self { z_index }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct ShaderTileMetadata {
    pub transform: Mat4x4f32,
    pub zoom_factor: f32,
}

impl ShaderTileMetadata {
    pub fn new(transform: Mat4x4f32, zoom_factor: f32) -> Self {
        Self {
            transform,
            zoom_factor,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct ShaderTextureVertex {
    pub position: Vec2f32,
    pub tex_coords: Vec2f32,
}

impl ShaderTextureVertex {
    pub fn new(position: Vec2f32, tex_coords: Vec2f32) -> Self {
        Self {
            position,
            tex_coords,
        }
    }
}

impl Default for ShaderTextureVertex {
    fn default() -> Self {
        ShaderTextureVertex::new([0.0, 0.0], [0.0, 0.0])
    }
}

pub struct RasterTileShader {
    pub format: wgpu::TextureFormat,
}

impl Shader for RasterTileShader {
    fn describe_vertex(&self) -> VertexState {
        VertexState {
            source: include_str!("tile_raster.vertex.wgsl"),
            entry_point: "main",
            buffers: vec![
                // tile metadata
                VertexBufferLayout {
                    array_stride: std::mem::size_of::<ShaderTileMetadata>() as u64,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: vec![
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
                        // zoom_factor
                        wgpu::VertexAttribute {
                            offset: 4 * wgpu::VertexFormat::Float32x4.size(),
                            format: wgpu::VertexFormat::Float32,
                            shader_location: 9,
                        },
                    ],
                },
                // layer metadata
                VertexBufferLayout {
                    array_stride: std::mem::size_of::<ShaderLayerMetadata>() as u64,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: vec![
                        // z_index
                        wgpu::VertexAttribute {
                            offset: 0,
                            format: wgpu::VertexFormat::Float32,
                            shader_location: 10,
                        },
                    ],
                },
            ],
        }
    }

    fn describe_fragment(&self) -> FragmentState {
        FragmentState {
            source: include_str!("tile_raster.fragment.wgsl"),
            entry_point: "main",
            targets: vec![Some(wgpu::ColorTargetState {
                format: self.format,
                blend: Some(wgpu::BlendState {
                    color: wgpu::BlendComponent::REPLACE,
                    alpha: wgpu::BlendComponent::REPLACE,
                }),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }
    }
}
