struct ShaderCamera {
    view_proj: mat4x4<f32>,
    view_position: vec4<f32>,
};

struct ShaderGlobals {
    camera: ShaderCamera,
};

@group(0) @binding(0) var<uniform> globals: ShaderGlobals;

struct VertexOutput {
    @location(0) color: vec4<f32>,
    @builtin(position) position: vec4<f32>,
};

@vertex
fn main(
    @builtin(vertex_index) vertex_idx: u32,
    @location(0) color: vec4<f32>,
    @location(1) z_index: f32, // Passed from per-layer metadata
) -> VertexOutput {
    // Generate a fullscreen quad using standard 6-vertex triangle list layout
    var positions = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 1.0, -1.0),
        vec2<f32>(-1.0,  1.0),
        vec2<f32>(-1.0,  1.0),
        vec2<f32>( 1.0, -1.0),
        vec2<f32>( 1.0,  1.0)
    );

    let pos = positions[vertex_idx % 6u];
    
    // Position z in vulkan normalized device coordinates mapping:
    let z = z_index;

    // Output raw clip space coordinates (identity mapping)
    var out: VertexOutput;
    
    // We use a small epsilon near 0.0 (the far plane) because wgpu `Greater` won't pass 0.0 > 0.0 
    out.position = vec4<f32>(pos, 1.0e-5, 1.0);
    out.color = color;

    return out;
}
