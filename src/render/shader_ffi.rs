use bytemuck_derive::{Pod, Zeroable};
use cgmath::SquareMatrix;

type Vec2f32 = [f32; 2];
type Vec4f32 = [f32; 4];
type Mat4f32 = [Vec4f32; 4];

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct CameraUniform {
    view_proj: Mat4f32,     // 64 bytes
    view_position: Vec4f32, // 16 bytes
}

impl CameraUniform {
    pub fn new(view_proj: Mat4f32, view_position: Vec4f32) -> Self {
        Self {
            view_position,
            view_proj,
        }
    }
}

impl Default for CameraUniform {
    fn default() -> Self {
        Self {
            view_position: [0.0; 4],
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct GlobalsUniform {
    camera: CameraUniform,
}

impl GlobalsUniform {
    pub fn new(camera_uniform: CameraUniform) -> Self {
        Self { camera: camera_uniform }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct GpuVertexUniform {
    pub position: Vec2f32,
    pub normal: Vec2f32,
    pub prim_id: u32,
    _pad1: i32, // _padX aligns it to 8 bytes = AlignOf(Vec2f32=vec2<f32>):
                // https://gpuweb.github.io/gpuweb/wgsl/#alignment-and-size
}

impl GpuVertexUniform {
    pub fn new(position: Vec2f32, normal: Vec2f32, prim_id: u32) -> Self {
        Self {
            position,
            normal,
            prim_id,
            _pad1: Default::default(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct PrimitiveUniform {
    pub color: Vec4f32,
    pub translate: Vec2f32,
    pub z_index: i32,
    pub width: f32,
    pub angle: f32,
    pub scale: f32,
    _pad1: i32, // _padX aligns it to 16 bytes = AlignOf(Vec4f32/vec4<f32>):
    _pad2: i32, // https://gpuweb.github.io/gpuweb/wgsl/#alignment-and-size
}

impl PrimitiveUniform {
    pub fn new(
        color: Vec4f32,
        translate: Vec2f32,
        z_index: i32,
        width: f32,
        angle: f32,
        scale: f32,
    ) -> Self {
        Self {
            color,
            translate,
            z_index,
            width,
            angle,
            scale,
            _pad1: Default::default(),
            _pad2: Default::default(),
        }
    }
}
