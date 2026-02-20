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
    @location(11) viewport_width: f32,
    @location(12) viewport_height: f32,
) -> VertexOutput {
    let line_width_px = 2.0;
    let blur = 0.0;
    let gapwidth = 0.0;

    let halfwidth = line_width_px * 0.5;
    let pixel_ratio = 1.0;
    let antialiasing = (1.0 / pixel_ratio) * 0.5;

    let inset = gapwidth + select(0.0, antialiasing, gapwidth > 0.0);
    let outset = gapwidth + halfwidth * select(1.0, 2.0, gapwidth > 0.0) +
                 select(0.0, antialiasing, halfwidth != 0.0);

    let transform = mat4x4<f32>(translate1, translate2, translate3, translate4);

    // Transform center position to clip space
    var center = transform * vec4<f32>(position, 0.0, 1.0);

    // Transform the normal direction to clip space (w=0 for direction vectors)
    let normal_clip = transform * vec4<f32>(normal, 0.0, 0.0);
    let dir = normalize(normal_clip.xy);

    // Apply pixel-width offset in clip space.
    // NDC spans 2 units across the viewport, so 1 pixel = 2/viewport_px in NDC.
    // Multiply by center.w to compensate for the perspective divide.
    // Use per-axis conversion to handle non-square viewports correctly.
    let px_to_clip_x = (2.0 / viewport_width) * center.w;
    let px_to_clip_y = (2.0 / viewport_height) * center.w;
    let clip_offset = vec2<f32>(dir.x * outset * px_to_clip_x, dir.y * outset * px_to_clip_y);
    center = vec4<f32>(center.x + clip_offset.x, center.y + clip_offset.y, z_index, center.w);

    return VertexOutput(
        center,
        color,
        normal,
        vec2<f32>(outset, inset),
        1.0
    );
}
