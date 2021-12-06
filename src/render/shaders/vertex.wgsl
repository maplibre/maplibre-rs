[[block]]
struct Globals {
    view_proj: mat4x4<f32>;
    view_position: vec4<f32>;
};

struct Primitive {
    color: vec4<f32>;
    translate: vec2<f32>;
    z_index: i32;
    width: f32;
    angle: f32;
    scale: f32;
    pad1: i32;
    pad2: i32;
};

[[block]]
struct Primitives {
    primitives: [[stride(48)]] array<Primitive, 256>;
};

[[group(0), binding(0)]] var<uniform> globals: Globals;
[[group(0), binding(1)]] var<uniform> u_primitives: Primitives;

struct VertexOutput {
    [[location(0)]] v_color: vec4<f32>;
    [[builtin(position)]] position: vec4<f32>;
};

[[stage(vertex)]]
fn main(
    [[location(0)]] a_position: vec2<f32>,
    [[location(1)]] a_normal: vec2<f32>,
    [[location(2)]] a_prim_id: u32,
    [[builtin(instance_index)]] instance_idx: u32 // instance_index is used when we have multiple instances of the same "object"
) -> VertexOutput {
    var prim: Primitive = u_primitives.primitives[a_prim_id + instance_idx];
    var z = 0.0;
    var world_pos = a_position + a_normal * prim.width;

    var position = globals.view_proj * vec4<f32>(world_pos, z, 1.0);

    return VertexOutput(prim.color, position);
}
