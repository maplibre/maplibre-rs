struct VertexOutput {
    @location(0) tex_coords: vec2<f32>,
    @builtin(position) clip_position: vec4<f32>,
};

@vertex
fn main(
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(4) translate1: vec4<f32>,
    @location(5) translate2: vec4<f32>,
    @location(6) translate3: vec4<f32>,
    @location(7) translate4: vec4<f32>,
    @location(9) zoom_factor: f32,
) -> VertexOutput {
    let z = 0.0;
    let width = 3.0 * zoom_factor;
    let normal = vec2<f32>(0.0,0.0);

    var position = mat4x4<f32>(translate1, translate2, translate3, translate4) * vec4<f32>(position + normal * width, z, 1.0);
    position.z = 1.0;

    return VertexOutput(tex_coords, position);
}