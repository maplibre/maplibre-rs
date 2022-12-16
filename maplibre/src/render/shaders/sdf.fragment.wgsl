struct VertexOutput {
    @location(0) is_glyph: i32,
    @location(1) tex_coords: vec2<f32>,
    @location(2) color: vec4<f32>,
    @builtin(position) position: vec4<f32>,
};

struct Output {
    @location(0) out_color: vec4<f32>,
};

@group(0) @binding(0)
var t_sprites: texture_2d<f32>;
@group(0) @binding(1)
var s_sprites: sampler;

@group(1) @binding(0)
var t_glyphs: texture_2d<f32>;
@group(1) @binding(1)
var s_glyphs: sampler;

@fragment
fn main(in: VertexOutput) -> Output {
    // Note: we access both textures to ensure uniform control flow:
    // https://www.khronos.org/opengl/wiki/Sampler_(GLSL)#Non-uniform_flow_control

    let tex_color = textureSample(t_sprites, s_sprites, in.tex_coords);

    // 0 => border, < 0 => inside, > 0 => outside
    // dist(ance) is scaled to [0.75, -0.25]
    let glyphDist = 0.75 - textureSample(t_glyphs, s_glyphs, in.tex_coords).r;

    if (in.is_glyph == 0) {
        return Output(tex_color);
    } else {
        // TODO: support:
        // - outline
        // - blur

        let alpha: f32 = smoothstep(0.10, 0.0, glyphDist);
        return Output(vec4(in.color.bgr, in.color.a * alpha));
    }
}
