struct CameraUniform {
    view_proj: mat4x4<f32>;
    view_position: vec4<f32>;
};


struct GlobalsUniform {
    camera: CameraUniform;
};

[[group(0), binding(0)]] var<uniform> globals: GlobalsUniform;

struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
};

[[stage(vertex)]]
fn main(
    [[location(0)]] a_position: vec2<f32>,
    [[location(4)]] mask_offset: vec2<f32>,
    [[builtin(instance_index)]] instance_idx: u32 // instance_index is used when we have multiple instances of the same "object"
) -> VertexOutput {
    var z = 0.0;

    var world_pos = a_position + mask_offset;

    if (instance_idx == u32(1)) {
        if (a_position.x == 0.0) {
            world_pos.x = 0.0;
        }
    }

    var position = globals.camera.view_proj * vec4<f32>(world_pos, z, 1.0);

    return VertexOutput(position);
}
