//! Background, globe surface, and atmospheric shader descriptors.

use bytemuck_derive::{Pod, Zeroable};
use thiserror::Error;

use super::{Mat4x4f32, Shader, ShaderTileMetadata};
use crate::{
    projection::globe::camera::GlobeCameraState,
    render::resource::{FragmentState, VertexBufferLayout, VertexState},
    style::light::{LightError, LightSpecification},
};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct BackgroundLayerMetadata {
    pub color: [f32; 4],
    pub z_index: f32,
}

pub struct BackgroundShader {
    pub format: wgpu::TextureFormat,
}

/// Shader drawing background paint on the projected globe surface.
pub struct GlobeBackgroundShader {
    /// Render-target format.
    pub format: wgpu::TextureFormat,
}

/// Per-draw inputs for GL JS-compatible atmospheric scattering.
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct AtmosphereLayerMetadata {
    inverse_projection: Mat4x4f32,
    globe_position: [f32; 4],
    sun_direction: [f32; 4],
    radius_blend_padding: [f32; 4],
}

impl AtmosphereLayerMetadata {
    /// Creates inert metadata for frames where no atmosphere draw is queued.
    pub fn disabled() -> Self {
        Self {
            inverse_projection: cgmath::Matrix4::from_scale(1.0).into(),
            globe_position: [0.0; 4],
            sun_direction: [0.0, 0.0, 1.0, 0.0],
            radius_blend_padding: [1.0, 0.0, 0.0, 0.0],
        }
    }

    /// Creates atmosphere inputs from the current globe camera and root light.
    pub fn from_view(
        camera: &GlobeCameraState,
        light: &LightSpecification,
        zoom: f64,
        blend: f32,
    ) -> Result<Self, AtmosphereMetadataError> {
        let inverse_projection = camera
            .inverse_projection()
            .cast::<f32>()
            .ok_or(AtmosphereMetadataError::FloatConversion)?;
        let globe_position = camera
            .globe_center_in_view()
            .cast::<f32>()
            .ok_or(AtmosphereMetadataError::FloatConversion)?;
        let sun_direction = light
            .sun_direction_in_view(camera, zoom)
            .map_err(|source| AtmosphereMetadataError::Light { source })?
            .cast::<f32>()
            .ok_or(AtmosphereMetadataError::FloatConversion)?;
        let radius = camera.globe_radius_pixels() as f32;
        if !radius.is_finite() || radius <= 0.0 || !blend.is_finite() {
            return Err(AtmosphereMetadataError::InvalidScalar { radius, blend });
        }
        Ok(Self {
            inverse_projection: inverse_projection.into(),
            globe_position: [globe_position.x, globe_position.y, globe_position.z, 0.0],
            sun_direction: [sun_direction.x, sun_direction.y, sun_direction.z, 0.0],
            radius_blend_padding: [radius, blend.clamp(0.0, 1.0), 0.0, 0.0],
        })
    }
}

/// Invalid CPU inputs for the atmosphere shader.
#[derive(Clone, Copy, Debug, Error, PartialEq)]
pub enum AtmosphereMetadataError {
    /// The root light cannot be evaluated for this view.
    #[error("failed to evaluate atmosphere light")]
    Light {
        /// Underlying light error.
        #[source]
        source: LightError,
    },
    /// Camera values cannot be represented by the shader's f32 interface.
    #[error("atmosphere camera data cannot be represented as f32")]
    FloatConversion,
    /// Radius and blend must be finite and radius must be positive.
    #[error("invalid atmosphere radius {radius} or blend {blend}")]
    InvalidScalar {
        /// Globe radius in view-space pixels.
        radius: f32,
        /// Evaluated atmosphere blend.
        blend: f32,
    },
}

/// Shader drawing a translucent atmospheric shell.
pub struct AtmosphereShader {
    /// Render-target format.
    pub format: wgpu::TextureFormat,
}

#[cfg(test)]
mod tests;

impl Shader for AtmosphereShader {
    fn describe_vertex(&self) -> VertexState {
        VertexState {
            source: include_str!("atmosphere.vertex.wgsl"),
            entry_point: "main",
            buffers: atmosphere_buffers(),
        }
    }

    fn describe_fragment(&self) -> FragmentState {
        FragmentState {
            source: include_str!("atmosphere.fragment.wgsl"),
            entry_point: "main",
            targets: vec![Some(wgpu::ColorTargetState {
                format: self.format,
                // The scattering shader emits premultiplied RGB, matching GL JS's
                // ColorMode.alphaBlended (`ONE`, `ONE_MINUS_SRC_ALPHA`).
                blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }
    }
}

fn atmosphere_buffers() -> Vec<VertexBufferLayout> {
    vec![VertexBufferLayout {
        array_stride: std::mem::size_of::<AtmosphereLayerMetadata>() as u64,
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: vec![
            matrix_attribute(0, 8),
            matrix_attribute(1, 9),
            matrix_attribute(2, 10),
            matrix_attribute(3, 11),
            wgpu::VertexAttribute {
                offset: 4 * wgpu::VertexFormat::Float32x4.size(),
                format: wgpu::VertexFormat::Float32x4,
                shader_location: 12,
            },
            wgpu::VertexAttribute {
                offset: 5 * wgpu::VertexFormat::Float32x4.size(),
                format: wgpu::VertexFormat::Float32x4,
                shader_location: 13,
            },
            wgpu::VertexAttribute {
                offset: 6 * wgpu::VertexFormat::Float32x4.size(),
                format: wgpu::VertexFormat::Float32x4,
                shader_location: 14,
            },
        ],
    }]
}

impl Shader for GlobeBackgroundShader {
    fn describe_vertex(&self) -> VertexState {
        VertexState {
            source: concat!(
                include_str!("projection.vertex.wgsl"),
                include_str!("globe_background.vertex.wgsl")
            ),
            entry_point: "main",
            buffers: globe_background_buffers(),
        }
    }

    fn describe_fragment(&self) -> FragmentState {
        FragmentState {
            source: include_str!("globe_background.fragment.wgsl"),
            entry_point: "main",
            targets: vec![Some(wgpu::ColorTargetState {
                format: self.format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }
    }
}

fn globe_background_buffers() -> Vec<VertexBufferLayout> {
    vec![
        tile_mesh_layout(),
        VertexBufferLayout {
            array_stride: std::mem::size_of::<ShaderTileMetadata>() as u64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: vec![
                matrix_attribute(0, 4),
                matrix_attribute(1, 5),
                matrix_attribute(2, 6),
                matrix_attribute(3, 7),
                wgpu::VertexAttribute {
                    offset: 4 * wgpu::VertexFormat::Float32x4.size()
                        + 3 * wgpu::VertexFormat::Float32.size(),
                    format: wgpu::VertexFormat::Float32x4,
                    shader_location: 2,
                },
            ],
        },
        VertexBufferLayout {
            array_stride: std::mem::size_of::<BackgroundLayerMetadata>() as u64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: vec![
                wgpu::VertexAttribute {
                    offset: 0,
                    format: wgpu::VertexFormat::Float32x4,
                    shader_location: 8,
                },
                wgpu::VertexAttribute {
                    offset: wgpu::VertexFormat::Float32x4.size(),
                    format: wgpu::VertexFormat::Float32,
                    shader_location: 10,
                },
            ],
        },
    ]
}

fn tile_mesh_layout() -> VertexBufferLayout {
    VertexBufferLayout {
        array_stride: std::mem::size_of::<crate::projection::globe::tile_mesh::TileMeshVertex>()
            as u64,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: vec![wgpu::VertexAttribute {
            offset: 0,
            format: wgpu::VertexFormat::Sint16x2,
            shader_location: 0,
        }],
    }
}

fn matrix_attribute(column: u64, shader_location: u32) -> wgpu::VertexAttribute {
    wgpu::VertexAttribute {
        offset: column * wgpu::VertexFormat::Float32x4.size(),
        format: wgpu::VertexFormat::Float32x4,
        shader_location,
    }
}

impl Shader for BackgroundShader {
    fn describe_vertex(&self) -> VertexState {
        VertexState {
            source: include_str!("background.vertex.wgsl"),
            entry_point: "main",
            buffers: vec![VertexBufferLayout {
                array_stride: std::mem::size_of::<BackgroundLayerMetadata>() as u64,
                step_mode: wgpu::VertexStepMode::Instance,
                attributes: vec![
                    wgpu::VertexAttribute {
                        offset: 0,
                        format: wgpu::VertexFormat::Float32x4,
                        shader_location: 0,
                    },
                    wgpu::VertexAttribute {
                        offset: 16,
                        format: wgpu::VertexFormat::Float32,
                        shader_location: 1,
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
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }
    }
}
