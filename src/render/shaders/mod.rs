use wgpu::{
    ColorTargetState, Device, FragmentState, ShaderModule, VertexBufferLayout, VertexState,
};

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
    use crate::platform::COLOR_TEXTURE_FORMAT;
    use crate::render::shader_ffi::{GpuVertexUniform, TileUniform};

    use super::{FragmentShaderState, VertexShaderState};

    pub const VERTEX: VertexShaderState = VertexShaderState::new(
        include_str!("tile.vertex.wgsl"),
        &[
            wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<GpuVertexUniform>() as u64,
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
                    // tile_id
                    wgpu::VertexAttribute {
                        offset: 2 * wgpu::VertexFormat::Float32x2.size(),
                        format: wgpu::VertexFormat::Uint32,
                        shader_location: 2,
                    },
                ],
            },
            wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<TileUniform>() as u64,
                step_mode: wgpu::VertexStepMode::Instance,
                attributes: &[
                    // color
                    wgpu::VertexAttribute {
                        offset: 0,
                        format: wgpu::VertexFormat::Float32x4,
                        shader_location: 3,
                    },
                    // translate
                    wgpu::VertexAttribute {
                        offset: wgpu::VertexFormat::Float32x4.size(),
                        format: wgpu::VertexFormat::Float32x3,
                        shader_location: 4,
                    },
                ],
            },
        ],
    );

    pub const FRAGMENT: FragmentShaderState = FragmentShaderState::new(
        include_str!("tile.fragment.wgsl"),
        &[wgpu::ColorTargetState {
            format: COLOR_TEXTURE_FORMAT,
            blend: None,
            write_mask: wgpu::ColorWrites::ALL,
        }],
    );
}

pub mod tile_mask {
    use crate::platform::COLOR_TEXTURE_FORMAT;
    use crate::render::shader_ffi::{GpuVertexUniform, MaskInstanceUniform};

    use super::{FragmentShaderState, VertexShaderState};

    pub const VERTEX: VertexShaderState = VertexShaderState::new(
        include_str!("tile_mask.vertex.wgsl"),
        &[wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<MaskInstanceUniform>() as u64,
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
            write_mask: wgpu::ColorWrites::empty(),
        }],
    );
}
