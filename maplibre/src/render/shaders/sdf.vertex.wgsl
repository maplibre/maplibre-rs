struct VertexOutput {
    @location(0) is_glyph: i32,
    @location(1) tex_coords: vec2<f32>,
    @location(2) color: vec4<f32>,
    @builtin(position) position: vec4<f32>,
};

@vertex
fn main(
    @location(0) position: vec3<f32>,
    @location(1) text_anchor: vec3<f32>,
    @location(4) translate1: vec4<f32>,
    @location(5) translate2: vec4<f32>,
    @location(6) translate3: vec4<f32>,
    @location(7) translate4: vec4<f32>,
    @location(9) zoom_factor: f32,
    @location(10) z_index: f32,
    @location(11) tex_coords: vec2<f32>,
    @builtin(instance_index) instance_idx: u32 // instance_index is used when we have multiple instances of the same "object"
) -> VertexOutput {
    let scaling: mat3x3<f32> = mat3x3<f32>(
            vec3<f32>(zoom_factor,   0.0,          0.0),
            vec3<f32>(0.0,            zoom_factor, 0.0),
            vec3<f32>(0.0,            0.0,         1.0)
    );

    var position = mat4x4<f32>(translate1, translate2, translate3, translate4) * vec4<f32>((scaling * (position - text_anchor) + text_anchor), 1.0);
    position.z = z_index;

    return VertexOutput(1, tex_coords, vec4<f32>(0.0, 0.0, 0.0, 1.0), position);
}
