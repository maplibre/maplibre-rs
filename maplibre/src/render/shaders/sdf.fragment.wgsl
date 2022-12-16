struct VertexOutput {
    @location(0) tex_coords: vec2<f32>,
    @builtin(position) position: vec4<f32>,
};

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;


//layout(set = 1, binding = 0) uniform texture2D t_sprites;
  //layout(set = 1, binding = 1) uniform sampler s_sprites;
  //
  //layout(set = 2, binding = 0) uniform texture2D t_glyphs;
  //layout(set = 2, binding = 1) uniform sampler s_glyphs;

@fragment
//layout(location=0) flat in uint f_glyph;
  //layout(location=1) in vec2 f_tex_coords;
  //layout(location=2) in vec4 color;
fn main(in: VertexOutput) -> @location(0) vec4<f32> {

    // Note: we access both textures to ensure uniform control flow:
    // https://www.khronos.org/opengl/wiki/Sampler_(GLSL)#Non-uniform_flow_control

    vec4 tex_color = texture(sampler2D(t_sprites, s_sprites), f_tex_coords);

    // 0 => border, < 0 => inside, > 0 => outside
    // dist(ance) is scaled to [0.75, -0.25]
    float glyphDist = 0.75 - texture(sampler2D(t_glyphs, s_glyphs), f_tex_coords).r;

    if (f_glyph == 0) {
        f_color = tex_color.bgra;
    } else {
        // TODO: support:
        // - outline
        // - blur

        float alpha = smoothstep(0.10, 0, glyphDist);
        f_color = vec4(color.bgr, color.a * alpha);
    }

    //return textureSample(t_diffuse, s_diffuse, in.tex_coords);
}
