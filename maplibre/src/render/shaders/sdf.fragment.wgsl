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
var t_glyphs: texture_2d<f32>;
@group(0) @binding(1)
var s_glyphs: sampler;

@fragment
fn main(in: VertexOutput) -> Output {
    // Note: Ensure uniform control flow!
    // https://www.khronos.org/opengl/wiki/Sampler_(GLSL)#Non-uniform_flow_control

    // 0 => border, < 0 => inside, > 0 => outside
    // dist(ance) is scaled to [0.75, -0.25]
    let glyphDist = 0.75 - textureSample(t_glyphs, s_glyphs, in.tex_coords).r;

    // TODO: support:
    // - outline
    // - blur

    let alpha: f32 = smoothstep(0.10, 0.0, glyphDist);

    // "Another Good Trick" from https://www.sjbaker.org/steve/omniv/alpha_sorting.html
    // Using discard is an alternative for GL_ALPHA_TEST.
    // https://stackoverflow.com/questions/53024693/opengl-is-discard-the-only-replacement-for-deprecated-gl-alpha-test
    // Alternative is to disable the depth buffer for the RenderPass using sdf.fragment.wgsl
    if (alpha == 0.0) {
        discard;
    }

    return Output(vec4(in.color.rgb, in.color.a * alpha));
}
