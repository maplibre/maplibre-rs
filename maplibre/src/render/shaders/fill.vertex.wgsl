// @include projection.vertex.wgsl

struct VertexOutput {
    @location(0)  v_color: vec4<f32>,
    @location(4) horizon_distance: f32,
    @builtin(position) position: vec4<f32>,
};

@vertex
fn main(
    @location(0) position: vec2<f32>,
    @location(1) normal: vec2<f32>,
    @location(2) tile_mercator_coords: vec4<f32>,
    @location(4) translate1: vec4<f32>,
    @location(5) translate2: vec4<f32>,
    @location(6) translate3: vec4<f32>,
    @location(7) translate4: vec4<f32>,
    @location(8) color: vec4<f32>,
    @location(9) zoom_factor: f32,
    @location(10) z_index: f32,
    @location(15) layer_translate: vec2<f32>,
    @builtin(instance_index) instance_idx: u32 // instance_index is used when we have multiple instances of the same "object"
) -> VertexOutput {
    let z = -z_index;
    let width = 3.0 * zoom_factor;

    // The following code moves all "invisible" vertices to (0, 0, 0)
    //if (color.w == 0.0) {
    //   return VertexOutput(color, vec4<f32>(0.0, 0.0, 0.0, 1.0));
    //}

    let projected = project_tile_position(
        vec3<f32>(position + layer_translate + normal * width, z),
        mat4x4<f32>(translate1, translate2, translate3, translate4),
        tile_mercator_coords,
    );
    var final_position = projected.clip_position;
    final_position.z = z_index;

    return VertexOutput(color, projected.horizon_distance, final_position);
}
