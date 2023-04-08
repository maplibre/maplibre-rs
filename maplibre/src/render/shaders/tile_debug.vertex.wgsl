struct VertexOutput {
    @location(0) v_color: vec4<f32>,
    @builtin(position) position: vec4<f32>,
};

var<private> EXTENT: f32 = 4096.0;
var<private> DEBUG_COLOR: vec4<f32> = vec4<f32>(1.0, 0.0, 0.0, 1.0);

@vertex
fn main(
    @location(4) translate1: vec4<f32>,
    @location(5) translate2: vec4<f32>,
    @location(6) translate3: vec4<f32>,
    @location(7) translate4: vec4<f32>,
    @location(9) zoom_factor: f32,
    @builtin(vertex_index) vertex_idx: u32,
    @builtin(instance_index) instance_idx: u32 // instance_index is used when we have multiple instances of the same "object"
) -> VertexOutput {
    let z = 0.0;

    let target_width = 1.0;
    let target_height = 1.0;

    let WIDTH = EXTENT * zoom_factor / 1024.0;

    var VERTICES: array<vec3<f32>, 24> = array<vec3<f32>, 24>(
        // Debug lines vertices
        // left
        vec3<f32>(0.0, 0.0, z),
        vec3<f32>(0.0, EXTENT, z),
        vec3<f32>(WIDTH, EXTENT, z),

        vec3<f32>(0.0, 0.0, z),
        vec3<f32>(WIDTH, EXTENT, z),
        vec3<f32>(WIDTH, 0.0, z),

        // right
        vec3<f32>(EXTENT - WIDTH, 0.0, z),
        vec3<f32>(EXTENT - WIDTH, EXTENT, z),
        vec3<f32>(EXTENT, EXTENT, z),

        vec3<f32>(EXTENT - WIDTH, 0.0, z),
        vec3<f32>(EXTENT, EXTENT, z),
        vec3<f32>(EXTENT, 0.0, z),

        // top
        vec3<f32>(0.0, 0.0, z),
        vec3<f32>(0.0, WIDTH, z),
        vec3<f32>(EXTENT, WIDTH, z),

        vec3<f32>(0.0, 0.0, z),
        vec3<f32>(EXTENT, WIDTH, z),
        vec3<f32>(EXTENT, 0.0, z),

        // bottom
        vec3<f32>(0.0, EXTENT - WIDTH, z),
        vec3<f32>(0.0, EXTENT, z),
        vec3<f32>(EXTENT, EXTENT, z),

        vec3<f32>(0.0, EXTENT - WIDTH, z),
        vec3<f32>(EXTENT, EXTENT, z),
        vec3<f32>(EXTENT, EXTENT - WIDTH, z)
    );

    let vertex = VERTICES[vertex_idx];

    let scaling: mat3x3<f32> = mat3x3<f32>(
            vec3<f32>(target_width,   0.0,            0.0),
            vec3<f32>(0.0,            target_height,  0.0),
            vec3<f32>(0.0,            0.0,            1.0)
    );

    var final_position = mat4x4<f32>(translate1, translate2, translate3, translate4) * vec4<f32>((scaling * vertex), 1.0);
    final_position.z = 1.0;
    return VertexOutput(DEBUG_COLOR, final_position);
}
