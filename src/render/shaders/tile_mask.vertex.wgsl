struct ShaderCamera {
    view_proj: mat4x4<f32>;
    view_position: vec4<f32>;
};

struct ShaderGlobal {
    camera: ShaderCamera;
};

[[group(0), binding(0)]] var<uniform> globals: ShaderGlobal;

struct VertexOutput {
    [[location(0)]] v_color: vec4<f32>;
    [[builtin(position)]] position: vec4<f32>;
};

let EXTENT = 4096.0;

[[stage(vertex)]]
fn main(
    [[location(4)]] mask_offset: vec2<f32>,
    [[location(5)]] target_width: f32,
    [[location(6)]] target_height: f32,
    [[location(7)]] debug_color: vec4<f32>,
    [[builtin(vertex_index)]] vertex_idx: u32,
    [[builtin(instance_index)]] instance_idx: u32 // instance_index is used when we have multiple instances of the same "object"
) -> VertexOutput {
    let z = 0.0;

    var VERTICES: array<vec3<f32>, 6> = array<vec3<f32>, 6>(
        vec3<f32>(0.0, 0.0, z),
        vec3<f32>(0.0, EXTENT, z),
        vec3<f32>(EXTENT, 0.0, z),
        vec3<f32>(EXTENT, 0.0, z),
        vec3<f32>(0.0, EXTENT, z),
        vec3<f32>(EXTENT, EXTENT, z)
    );
    let a_position = VERTICES[vertex_idx];

    let scaling: mat3x3<f32> = mat3x3<f32>(
            vec3<f32>(target_width,   0.0,            0.0),
            vec3<f32>(0.0,            target_height,  0.0),
            vec3<f32>(0.0,            0.0,            1.0)
    );

    let world_pos = scaling * a_position + vec3<f32>(mask_offset, z);

    let position = globals.camera.view_proj * vec4<f32>(world_pos, 1.0);

    return VertexOutput(debug_color, position);
}
