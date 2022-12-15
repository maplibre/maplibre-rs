struct ShaderCamera {
    view_proj: mat4x4<f32>,
    view_position: vec4<f32>,
};

struct ShaderLight {
    direction: vec4<f32>,
    color: vec4<f32>,
}

struct ShaderGlobals {
    camera: ShaderCamera,
    light: ShaderLight,
};

@group(0) @binding(0) var<uniform> globals: ShaderGlobals;

struct Output {
    @location(0) out_color: vec4<f32>,
};

@fragment
fn main(@builtin(position) position: vec4<f32>, @location(0) v_color: vec4<f32>, @location(1) normal: vec3<f32>) -> Output {

    // We don't need (or want) much ambient light, so 0.1 is fine
    let ambient_strength = 0.1;
    let ambient_color = globals.light.color.xyz * ambient_strength;

    let diffuse_strength = max(dot(normal, globals.light.direction.xyz), 0.0);
    let diffuse_color = globals.light.color.xyz * diffuse_strength;

    let result = (ambient_color + diffuse_color) * v_color.xyz;

    return Output(vec4<f32>(result, v_color.a));
}
