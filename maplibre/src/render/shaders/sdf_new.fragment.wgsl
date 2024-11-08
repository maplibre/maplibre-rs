struct VertexOutput {
    @location(1) v_data0: vec2<f32>,
    @location(2) v_data1: vec3<f32>,
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
let SDF_PX:f32 =  8.0;

    let   fill_color: vec4<f32> = vec4<f32>(1.0, 0.0, 0.0, 1.0);  // highp
    let  halo_color: vec4<f32>= vec4<f32>(0.0, 1.0, 0.0, 1.0);   // highp
    let   opacity: f32 = 1.0; // lowp
    let   halo_width: f32 = 1.0; // lowp
    let   halo_blur: f32 = 1.0; // lowp

let  u_is_halo: bool = false;
let  u_gamma_scale: f32 = 1.0; // highp
let  u_device_pixel_ratio: f32 = 1.0; // lowp
let  u_is_text: bool = true;

let tex: vec2<f32> = in.v_data0.xy;


 let EDGE_GAMMA: f32 = 0.105 / u_device_pixel_ratio;


    let gamma_scale: f32 = in.v_data1.x;
    let size: f32 = in.v_data1.y;
    let fade_opacity: f32 = in.v_data1[2];

    let fontScale: f32 = select(size,  size / 24.0, u_is_text);

    var color: vec4<f32> = fill_color; // lowp
    var  gamma: f32 = EDGE_GAMMA / (fontScale * u_gamma_scale); // highp
     var buff = (256.0 - 64.0) / 256.0; // lowp
    if (u_is_halo) {
        color = halo_color;
        gamma = (halo_blur * 1.19 / SDF_PX + EDGE_GAMMA) / (fontScale * u_gamma_scale);
        buff = (6.0 - halo_width / fontScale) / SDF_PX;
    }

     let dist: f32 =  textureSample(t_glyphs, s_glyphs, tex).r; // lowp
     let gamma_scaled: f32 = gamma * gamma_scale; // highp
     let alpha: f32 = smoothstep(buff - gamma_scaled, buff + gamma_scaled, dist); // highp

    let fragColor = color * (alpha * opacity * fade_opacity);



    return Output(fragColor);




}

/*
 let buffer_width: f32 = 0.25;
        let buffer_center_outline: f32 = 0.8;

        // At which offset is the outline of the SDF?
        let outline_center_offset: f32 = -0.25;

        // shift outline by `outline_width` to the ouside
        let buffer_center: f32 = buffer_center_outline + outline_center_offset;

        let outline_color = vec3<f32>(0.0, 0.0, 0.0);

        // 0 => border, < 0 => inside, > 0 => outside
        let dist = textureSample(t_glyphs, s_glyphs, tex).r;

        let alpha: f32 = smoothstep(buffer_center - buffer_width / 2.0, buffer_center + buffer_width / 2.0, dist);
        let border: f32 = smoothstep(buffer_center_outline - buffer_width / 2.0, buffer_center_outline + buffer_width / 2.0, dist);

        let color = vec4<f32>(1.0, 0.0, 0.0, 1.0);

        let color_rgb = mix(outline_color.rgb, color.rgb, border);

        // The translucent pass does not have a depth buffer. Therefore we do not need to discord the fragments:
        // "Another Good Trick" from https://www.sjbaker.org/steve/omniv/alpha_sorting.html
        // Using discard is an alternative for GL_ALPHA_TEST.
        // https://stackoverflow.com/questions/53024693/opengl-is-discard-the-only-replacement-for-deprecated-gl-alpha-test


        return Output(vec4(color_rgb, color.a * alpha * 1.0));
       //   return Output(vec4(vec3<f32>(1.0, 0.0, 0.0), 1.0)); // debug bounding box, alpha 0.2 to see collisions
*/


