struct VertexOutput {
    @location(0) view_direction: vec3<f32>,
    @location(1) @interpolate(flat) globe_position: vec3<f32>,
    @location(2) @interpolate(flat) sun_direction: vec3<f32>,
    @location(3) @interpolate(flat) globe_radius: f32,
    @location(4) @interpolate(flat) atmosphere_blend: f32,
    @builtin(position) position: vec4<f32>,
};

@vertex
fn main(
    @builtin(vertex_index) vertex_index: u32,
    @location(8) inverse_projection_0: vec4<f32>,
    @location(9) inverse_projection_1: vec4<f32>,
    @location(10) inverse_projection_2: vec4<f32>,
    @location(11) inverse_projection_3: vec4<f32>,
    @location(12) globe_position: vec4<f32>,
    @location(13) sun_direction: vec4<f32>,
    @location(14) radius_blend_padding: vec4<f32>,
) -> VertexOutput {
    var position = vec2<f32>(-1.0, -1.0);
    if vertex_index == 1u {
        position = vec2<f32>(3.0, -1.0);
    } else if vertex_index == 2u {
        position = vec2<f32>(-1.0, 3.0);
    }
    let clip_position = vec4<f32>(position, 0.0, 1.0);
    let inverse_projection = mat4x4<f32>(
        inverse_projection_0,
        inverse_projection_1,
        inverse_projection_2,
        inverse_projection_3,
    );
    // The inverse perspective matrix already returns a ray vector. Dividing by W would
    // introduce far-plane cancellation at high globe zooms and differs from the GL JS shader.
    let view = inverse_projection * clip_position;
    return VertexOutput(
        view.xyz,
        globe_position.xyz,
        sun_direction.xyz,
        radius_blend_padding.x,
        radius_blend_padding.y,
        clip_position,
    );
}
