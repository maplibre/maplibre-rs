struct ShaderCamera {
    view_proj: mat4x4<f32>;
    view_position: vec4<f32>;
};

struct ShaderGlobals {
    camera:  ShaderCamera;
};

[[group(0), binding(0)]] var<uniform> globals: ShaderGlobals;

struct VertexOutput {
    [[location(0)]] v_color: vec4<f32>;
    [[builtin(position)]] position: vec4<f32>;
};

[[stage(vertex)]]
fn main(
    [[location(0)]] position: vec2<f32>,
    [[location(1)]] normal: vec2<f32>,
    [[location(3)]] color: vec4<f32>,
    [[location(4)]] translate: vec3<f32>,
    [[builtin(instance_index)]] instance_idx: u32 // instance_index is used when we have multiple instances of the same "object"
) -> VertexOutput {
    let z = 0.0;
    let width = 3.0;

    let world_pos = vec3<f32>(position + normal * width, z) + translate;

    let position = globals.camera.view_proj * vec4<f32>(world_pos, 1.0);

    return VertexOutput(color, position);
}
