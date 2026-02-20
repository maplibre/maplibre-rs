struct ShaderCamera {
    view_proj: mat4x4<f32>,
    view_position: vec4<f32>,
};

struct ShaderGlobals {
    camera: ShaderCamera,
};

@group(0) @binding(0) var<uniform> globals: ShaderGlobals;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) v_color: vec4<f32>,
    @location(1) v_normal: vec2<f32>,
    @location(2) v_width2: vec2<f32>,
    @location(3) v_gamma_scale: f32,
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
) -> VertexOutput {
    let z = -z_index;
    
    let line_width_px = 2.0;
    let width = line_width_px * 8.0 / max(zoom_factor, 0.001);
    let blur = 0.0;
    let gapwidth = 0.0;
    
    let halfwidth = width * 0.5;
    let pixel_ratio = 1.0; 
    let antialiasing = (1.0 / pixel_ratio) * 0.5;
    
    let inset = gapwidth + select(0.0, antialiasing, gapwidth > 0.0);
    let outset = gapwidth + halfwidth * select(1.0, 2.0, gapwidth > 0.0) +
                 select(0.0, antialiasing, halfwidth != 0.0);

    let transform = mat4x4<f32>(translate1, translate2, translate3, translate4);
    
    // Extrude based on normal (from CPU tessellator) and width
    let dist = normal * outset;
    
    // The following code moves all "invisible" vertices to (0, 0, 0)
    // if (color.w == 0.0) {
    //   return VertexOutput(color, vec4<f32>(0.0, 0.0, 0.0, 1.0));
    // }

    var final_position = transform * vec4<f32>(position + dist, z, 1.0);
    final_position.z = z_index;

    // Approximating gamma scale for anti-aliasing calculation in fragment
    // Same as native, length without perspective / length with perspective
    let extrude_length_without_perspective = length(dist);
    let extrude_length_with_perspective = length(dist); // simplified until perspective matrix
    let gamma_denom = max(extrude_length_with_perspective, 1e-6);

    return VertexOutput(
        final_position,
        color,
        normal,       // raw direction
        vec2<f32>(outset, inset),
        extrude_length_without_perspective / gamma_denom
    );
}
