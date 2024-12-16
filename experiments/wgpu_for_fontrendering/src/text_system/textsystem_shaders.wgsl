// Vertex shader

struct CameraUniform {
    view_proj: mat4x4<f32>
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var prepass_target_texture: texture_2d<f32>;
@group(1) @binding(1)
var prepass_target_texture_sampler: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
};

struct InstanceInput {
    @location(2) model_matrix_0: vec4<f32>,
    @location(3) model_matrix_1: vec4<f32>,
    @location(4) model_matrix_2: vec4<f32>,
    @location(5) model_matrix_3: vec4<f32>,
    @location(6) color: vec3<f32>,
};

struct VertexOutputPrePass {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv_coords: vec2<f32>,
    @location(1) color: vec4<f32>,
};


// ########### Pre Pass ################
// Regular transformation to NDC
// Then additive fragment rendering into a texture

@vertex
fn prepass_vs(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutputPrePass {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );
    var out: VertexOutputPrePass;
    out.clip_position = camera.view_proj * model_matrix * vec4<f32>(model.position, 1.0);
    out.uv_coords = model.uv;
    out.color = vec4<f32>(instance.color.rgb, 1.0);
    return out;
}

@fragment
fn prepass_fs(in: VertexOutputPrePass) -> @location(0) vec4<f32> {
    // Discard fragments outside the curve defined by u^2 - v
    if ((in.uv_coords.x * in.uv_coords.x) - in.uv_coords.y > 0.0) {
        discard;
    }
    let color = in.color;
    return vec4<f32>(color.xyz, 1.0 / 255.0); // 1/255 so overlapping triangles add up to alpha values of n * 1/255
}

// ########## Main Pass #################
// Create a full screen quad (with uv's from 0 - 1) (assumes 6 input vertices, but disregards their coordinates and creates a screen-sized quad instead)
// Read from the prepass texture and only paint the pixels with odd alpha value

struct VertexOutputMainPass {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

@vertex
fn mainpass_vs(@builtin(vertex_index) index: u32) -> VertexOutputMainPass {
    // map the vertices that are passed in to a screen-sized quad
    var pos = array<vec2<f32>, 4>(
      vec2<f32>(-1.0,  1.0),
      vec2<f32>(-1.0, -1.0),
      vec2<f32>( 1.0, -1.0),
      vec2<f32>( 1.0,  1.0),
      );

    var uv = array<vec2<f32>, 4>(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(1.0, 0.0),
        );

    var output : VertexOutputMainPass;
    output.clip_position = vec4<f32>(pos[index], 0.0, 1.0);
    output.tex_coords = uv[index];
    return output;
}

@fragment
fn mainpass_fs(in: VertexOutputMainPass) -> @location(0) vec4<f32> {
    // look up color in texture -> TODO: currently this is all very inefficient, because we're only using the alpha of the texture!!!!
    let color = textureSample(prepass_target_texture, prepass_target_texture_sampler, in.tex_coords);
    var windingNumber: u32 = u32(color.a * 255.0);
    if (windingNumber % 2u == 1u) { 
        return vec4<f32>(color.rgb, 1.0);
    } else {
        discard;
    }
}

