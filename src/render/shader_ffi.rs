use bytemuck_derive::{Pod, Zeroable};
use cgmath::SquareMatrix;

pub type Vec2f32 = [f32; 2];
pub type Vec3f32 = [f32; 3];
pub type Vec4f32 = [f32; 4];
pub type Mat4f32 = [Vec4f32; 4];

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
        Self {
            camera: camera_uniform,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct GpuVertexUniform {
    pub position: Vec2f32,
    pub normal: Vec2f32,
    pub tile_id: u32,
    _pad1: i32, // _padX aligns it to 8 bytes = AlignOf(Vec2f32=vec2<f32>):
                // https://gpuweb.github.io/gpuweb/wgsl/#alignment-and-size
}

impl GpuVertexUniform {
    pub fn new(position: Vec2f32, normal: Vec2f32, tile_id: u32) -> Self {
        Self {
            position,
            normal,
            tile_id,
            _pad1: Default::default(),
        }
    }
}

impl Default for GpuVertexUniform {
    fn default() -> Self {
        GpuVertexUniform::new([0.0, 0.0], [0.0, 0.0], 0)
    }
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct MaskInstanceUniform {
    pub position: Vec2f32,
    pub target_width: f32,
    pub target_height: f32,
    pub debug_color: Vec4f32,
}

impl MaskInstanceUniform {
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
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct TileUniform {
    pub color: Vec4f32,
    pub translate: Vec3f32,
    _pad1: i32, // _padX aligns it to 16 bytes = AlignOf(Vec4f32/vec4<f32>): // https://gpuweb.github.io/gpuweb/wgsl/#alignment-and-size
}

impl TileUniform {
    pub fn new(color: Vec4f32, translate: Vec3f32) -> Self {
        Self {
            color,
            translate,
            _pad1: Default::default(),
        }
    }
}
