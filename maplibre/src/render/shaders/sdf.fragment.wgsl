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

// Note: Ensure uniform control flow!
// https://www.khronos.org/opengl/wiki/Sampler_(GLSL)#Non-uniform_flow_control
@fragment
fn main(in: VertexOutput) -> Output {
    let buffer_width: f32 = 0.25;
    let buffer_center_outline: f32 = 0.8;

    // At which offset is the outline of the SDF?
    let outline_center_offset: f32 = -0.25;

    // shift outline by `outline_width` to the ouside
    let buffer_center: f32 = buffer_center_outline + outline_center_offset;

    let outline_color = vec3<f32>(0.0, 0.0, 0.0);

    // 0 => border, < 0 => inside, > 0 => outside
    let dist = textureSample(t_glyphs, s_glyphs, in.tex_coords).r;

    let alpha: f32 = smoothstep(buffer_center - buffer_width / 2.0, buffer_center + buffer_width / 2.0, dist);
    let border: f32 = smoothstep(buffer_center_outline - buffer_width / 2.0, buffer_center_outline + buffer_width / 2.0, dist);

    let color_rgb = mix(outline_color.rgb, in.color.rgb, border);

    // The translucent pass does not have a depth buffer. Therefore we do not need to discord the fragments:
    // "Another Good Trick" from https://www.sjbaker.org/steve/omniv/alpha_sorting.html
    // Using discard is an alternative for GL_ALPHA_TEST.
    // https://stackoverflow.com/questions/53024693/opengl-is-discard-the-only-replacement-for-deprecated-gl-alpha-test
    // if (alpha == 0.0) {
    //     discard;
    // }

    return Output(vec4(color_rgb, in.color.a * alpha));
}


// MapLibre SDF shader:
/*
    let SDF_PX = 8.0;
    let device_pixel_ratio = 1.0;
    let EDGE_GAMMA = 0.105 / device_pixel_ratio;

    let size = 6.0; // TODO
    let fontScale = size / 24.0; // TODO Why / 24?
    let halo_width = 0.5; // TODO
    let halo_blur = 0.5; // TODO
    let halo_color = vec4(1.0, 0.0, 0.0, 1.0);

    var color = in.color;
    var gamma_scale = 1.0;
    var gamma = EDGE_GAMMA / (fontScale * gamma_scale);
    var buff = (256.0 - 64.0) / 256.0;

    let is_halo = false;
    if (is_halo) {
        color = halo_color;
        gamma = (halo_blur * 1.19 / SDF_PX + EDGE_GAMMA) / (fontScale * gamma_scale);
        buff = (6.0 - halo_width / fontScale) / SDF_PX;
    }
*/