struct FragmentInput {
    @location(0) v_color: vec4<f32>,
    @location(1) v_normal: vec2<f32>,
    @location(2) v_width2: vec2<f32>,
    @location(3) v_gamma_scale: f32,
};

struct Output {
    @location(0) out_color: vec4<f32>,
};

@fragment
fn main(in: FragmentInput) -> Output {
    // Calculate the distance of the pixel from the line in pixels
    let dist = length(in.v_normal) * in.v_width2.x;

    let pixel_ratio = 1.0; 
    let blur = 0.0;
    
    // Calculate the antialiasing fade factor
    let blur2 = (blur + (1.0 / pixel_ratio)) * in.v_gamma_scale;
    let denom = max(blur2, 1e-6);
    let alpha = clamp(min(dist - (in.v_width2.y - blur2), in.v_width2.x - dist) / denom, 0.0, 1.0);

    // Output non-premultiplied alpha: the blend state (SrcAlpha, OneMinusSrcAlpha)
    // handles the premultiplication. Using v_color * alpha here would double-apply alpha.
    return Output(vec4<f32>(in.v_color.rgb, in.v_color.a * alpha));
}
