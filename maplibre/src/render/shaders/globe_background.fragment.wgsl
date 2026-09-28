struct Output {
    @location(0) out_color: vec4<f32>,
};

@fragment
fn main(
    @location(0) color: vec4<f32>,
    @location(4) horizon_distance: f32,
) -> Output {
    if horizon_distance < 0.0 {
        discard;
    }
    return Output(color);
}
