struct Output {
    @location(0) out_color: vec4<f32>,
};

@fragment
fn main(
    @location(0) v_color: vec4<f32>
) -> Output {
    // Basic fill shader fragment implementation.
    // Receives the per-vertex color/opacity and outputs it.
    // In native, opacity might be uniform or per-vertex.
    // For now we assume v_color has alpha pre-multiplied or applied.
    return Output(v_color);
}
