// @include projection.vertex.wgsl

struct VertexOutput {
    @location(0) v_color: vec4<f32>,
    @location(4) horizon_distance: f32,
    @builtin(position) position: vec4<f32>,
};

var<private> EXTENT: f32 = 4096.0;
var<private> DEBUG_COLOR: vec4<f32> = vec4<f32>(1.0, 0.0, 0.0, 1.0);

// Each tile edge is a strip of thin quads so the outline follows the curvature of the globe
// instead of cutting a chord between the tile corners.
const SEGMENTS: u32 = 32u;
const VERTICES_PER_QUAD: u32 = 6u;

@vertex
fn main(
    @location(2) tile_mercator_coords: vec4<f32>,
    @location(4) translate1: vec4<f32>,
    @location(5) translate2: vec4<f32>,
    @location(6) translate3: vec4<f32>,
    @location(7) translate4: vec4<f32>,
    @location(9) zoom_factor: f32,
    @builtin(vertex_index) vertex_idx: u32,
) -> VertexOutput {
    let width = EXTENT / 256.0 * zoom_factor; // Width is 1/256 of a tile

    let edge = vertex_idx / (SEGMENTS * VERTICES_PER_QUAD);
    let segment = (vertex_idx / VERTICES_PER_QUAD) % SEGMENTS;
    let corner = vertex_idx % VERTICES_PER_QUAD;
    let start = f32(segment) / f32(SEGMENTS) * EXTENT;
    let end = f32(segment + 1u) / f32(SEGMENTS) * EXTENT;

    // Two triangles per quad: (start, 0) (end, 0) (end, width) and (start, 0) (end, width) (start, width).
    var along = start;
    var across = 0.0;
    switch corner {
        case 1u, 4u: {
            along = end;
            across = select(0.0, width, corner == 4u);
        }
        case 2u: {
            along = end;
            across = width;
        }
        case 5u: {
            across = width;
        }
        default: {}
    }

    var position = vec2<f32>(across, along); // left edge
    switch edge {
        case 1u: {
            position = vec2<f32>(EXTENT - across, along); // right edge
        }
        case 2u: {
            position = vec2<f32>(along, across); // top edge
        }
        case 3u: {
            position = vec2<f32>(along, EXTENT - across); // bottom edge
        }
        default: {}
    }

    let projected = project_tile_position(
        vec3<f32>(position, 0.0),
        mat4x4<f32>(translate1, translate2, translate3, translate4),
        tile_mercator_coords,
    );
    return VertexOutput(DEBUG_COLOR, projected.horizon_distance, projected.clip_position);
}
