struct VertexOutput {
    @location(0) tex_coords: vec2<f32>,
    @builtin(position) clip_position: vec4<f32>,
};

var<private> EXTENT: f32 = 4096.0;

@vertex
fn main(
    @location(4) translate1: vec4<f32>,
    @location(5) translate2: vec4<f32>,
    @location(6) translate3: vec4<f32>,
    @location(7) translate4: vec4<f32>,
    @location(9) zoom_factor: f32,

    @location(10) z_index: f32,

    @builtin(vertex_index) vertex_idx: u32,
) -> VertexOutput {
    let z = 0.0;

    var VERTICES: array<vec3<f32>, 6> = array<vec3<f32>, 6>(
        // Tile vertices
        vec3<f32>(0.0, 0.0, z),
        vec3<f32>(0.0, EXTENT, z),
        vec3<f32>(EXTENT, 0.0, z),
        vec3<f32>(EXTENT, 0.0, z),
        vec3<f32>(0.0, EXTENT, z),
        vec3<f32>(EXTENT, EXTENT, z),
    );
    let vertex = VERTICES[vertex_idx];


    var TEX_COORDS: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 1.0),
    );
    let tex_coords = TEX_COORDS[vertex_idx];

    var final_position = mat4x4<f32>(translate1, translate2, translate3, translate4) * vec4<f32>(vertex, 1.0);
    final_position.z = z_index;

    return VertexOutput(tex_coords, final_position);
}
