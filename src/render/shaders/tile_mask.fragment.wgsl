
struct Output {
    [[location(0)]] out_color: vec4<f32>;
};

[[stage(fragment)]]
fn main() -> Output {
    return Output(vec4<f32>(1.0,1.0,1.0,1.0));
}
