struct ShaderCamera {
    view_proj: mat4x4<f32>,
    view_position: vec4<f32>,
};

struct ShaderGlobals {
    camera: ShaderCamera,
};

@group(0) @binding(0) var<uniform> globals: ShaderGlobals;

struct VertexOutput {
    @location(0)  v_color: vec4<f32>,
    @builtin(position) position: vec4<f32>,
};

@vertex
fn main(
    @location(0) position: vec2<f32>,
    @location(1) normal: vec2<f32>,
    @location(4) translate1: vec4<f32>,
    @location(5) translate2: vec4<f32>,
    @location(6) translate3: vec4<f32>,
    @location(7) translate4: vec4<f32>,
    @location(8) color: vec4<f32>,
    @location(9) zoom_factor: f32,
    @location(10) z_index: f32,
    @builtin(instance_index) instance_idx: u32 // instance_index is used when we have multiple instances of the same "object"
) -> VertexOutput {
    let z = 0.0;
    let width = 3.0 * zoom_factor;

    // The following code moves all "invisible" vertices to (0, 0, 0)
    //if (color.w == 0.0) {
    //   return VertexOutput(color, vec4<f32>(0.0, 0.0, 0.0, 1.0));
    //}

    var final_position = mat4x4<f32>(translate1, translate2, translate3, translate4) * vec4<f32>(position + normal * width, z, 1.0);
    // FIXME: how to fix z-fighting?
    final_position.z = z_index;

    return VertexOutput(color, final_position);
}
