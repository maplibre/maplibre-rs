struct ShaderCamera {
    view_proj: mat4x4<f32>,
    view_position: vec4<f32>,
};

struct ShaderGlobal {
    camera: ShaderCamera,
};

@group(0) @binding(0) var<uniform> globals: ShaderGlobal;

struct VertexOutput {
     @location(0) v_color: vec4<f32>,
    @builtin(position) position: vec4<f32>,
};

let EXTENT = 4096.0;

@vertex
fn main(
    @location(4) translate1: vec4<f32>,
    @location(5) translate2: vec4<f32>,
    @location(6) translate3: vec4<f32>,
    @location(7) translate4: vec4<f32>,
    @builtin(vertex_index) vertex_idx: u32,
    @builtin(instance_index) instance_idx: u32 // instance_index is used when we have multiple instances of the same "object"
) -> VertexOutput {
    let z = 0.0;

    let target_width = 1.0;
    let target_height = 1.0;
    let debug_color = vec4<f32>(1.0, 0.0, 0.0, 1.0);

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

    var position = mat4x4<f32>(translate1, translate2, translate3, translate4) * vec4<f32>((scaling * a_position), 1.0);
    // FIXME: how to fix z-fighting?
    position.z = 1.0;

    return VertexOutput(debug_color, position);
}
