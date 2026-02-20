use std::mem::size_of;

#[repr(C)]
struct Vec4f32(f32, f32, f32, f32);

#[repr(C)]
struct Mat4x4f32(Vec4f32, Vec4f32, Vec4f32, Vec4f32);

#[repr(C)]
struct ShaderTileMetadata {
    pub transform: Mat4x4f32,
    pub zoom_factor: f32,
}

#[repr(C)]
struct FillShaderFeatureMetadata {
    pub color: Vec4f32,
}

#[repr(C)]
struct ShaderLayerMetadata {
    pub z_index: f32,
}

fn main() {
    println!("ShaderTileMetadata size: {}", size_of::<ShaderTileMetadata>());
    println!("FillShaderFeatureMetadata size: {}", size_of::<FillShaderFeatureMetadata>());
    println!("ShaderLayerMetadata size: {}", size_of::<ShaderLayerMetadata>());
}
