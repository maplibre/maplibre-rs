
struct VertexOutput {
    @location(1) v_data0: vec2<f32>,
    @location(2) v_data1: vec3<f32>,
    @builtin(position) position: vec4<f32>,
};

@vertex
fn main(
    @location(0) a_pos_offset: vec4<i32>,
    @location(1) a_data: vec4<u32>,
    @location(2) a_pixeloffset: vec4<i32>,



    @location(4) translate1: vec4<f32>,
    @location(5) translate2: vec4<f32>,
    @location(6) translate3: vec4<f32>,
    @location(7) translate4: vec4<f32>,
    @location(9) zoom_factor: f32,
    //@location(10) z_index: f32,
    //@location(12) opacity: f32,
    @builtin(instance_index) instance_idx: u32 // instance_index i0 used when we have multiple instances of the same "object"
) -> VertexOutput {

    let unit: mat4x4<f32> = mat4x4<f32>(
            vec4<f32>(1.0,   0.0,          0.0,0.0),
            vec4<f32>(0.0,            1.0, 0.0,0.0),
            vec4<f32>(0.0,            0.0,         1.0,0.0),
            vec4<f32>(0.0,            0.0,         0.0,1.0)
    );


// contents of a_size vary based on the type of property value
// used for {text,icon}-size.
// For constants, a_size is disabled.
// For source functions, we bind only one value per vertex: the value of {text,icon}-size evaluated for the current feature.
// For composite functions:
// [ text-size(lowerZoomStop, feature),
//   text-size(upperZoomStop, feature) ]
let  u_is_size_zoom_constant: bool = true;
let  u_is_size_feature_constant: bool= true;
let   u_size_t: f32 = 0.0; // highp // used to interpolate between zoom stops when size is a composite function
let   u_size: f32 = 30.0; // highp // used when size is both zoom and feature constant
let u_matrix: mat4x4<f32> = mat4x4<f32>(translate1, translate2, translate3, translate4);
let u_label_plane_matrix: mat4x4<f32> = unit;
let u_coord_matrix: mat4x4<f32> = unit;
let u_is_text: bool = true;
let u_pitch_with_map: bool = true;
let  u_pitch: f32= 0.0; // highp
let u_rotate_symbol: bool = false;
let   u_aspect_ratio: f32 = 0.0; // highp
let   u_camera_to_center_distance: f32 = 0.0; // highp
let u_fade_change: f32 = 0.0;
let  u_texsize: vec2<f32>= vec2<f32>(3178.0, 30.0);

let a_projected_pos: vec3<f32> =  vec3<f32>(vec2<f32>(a_pos_offset.xy), 0.0);




    let a_pos: vec2<i32> = a_pos_offset.xy;
    let a_offset: vec2<i32>  = a_pos_offset.zw;

    let a_tex: vec2<u32>  = a_data.xy;
    let a_size: vec2<u32> = a_data.zw;

    let a_size_min: f32 = floor(f32(a_size[0]) * 0.5);
    let a_pxoffset: vec2<i32>  = a_pixeloffset.xy;

    let segment_angle: f32 = -a_projected_pos[2]; // highp
    var  size: f32 = u_size;

    if (!u_is_size_zoom_constant && !u_is_size_feature_constant) {
        size = mix(a_size_min, f32(a_size[1]), u_size_t) / 128.0;
    } else if (u_is_size_zoom_constant && !u_is_size_feature_constant) {
        size = a_size_min / 128.0;
    }

    let projectedPoint: vec4<f32> = u_matrix * vec4(vec2<f32>(a_pos), 0.0, 1.0);
    let camera_to_anchor_distance: f32 = projectedPoint.w; // highp
    // If the label is pitched with the map, layout is done in pitched space,
    // which makes labels in the distance smaller relative to viewport space.
    // We counteract part of that effect by multiplying by the perspective ratio.
    // If the label isn't pitched with the map, we do layout in viewport space,
    // which makes labels in the distance larger relative to the features around
    // them. We counteract part of that effect by dividing by the perspective ratio.
    let distance_ratio: f32 = select(
        u_camera_to_center_distance / camera_to_anchor_distance,
        camera_to_anchor_distance / u_camera_to_center_distance,
        u_pitch_with_map
    ); // highp

     let perspective_ratio: f32 = clamp(
        0.5 + 0.5 * distance_ratio,
        0.0, // Prevents oversized near-field symbols in pitched/overzoomed tiles
        4.0); // highp

    size *= perspective_ratio;

    let fontScale: f32 = select(size,  size / 24.0, u_is_text);

    let symbol_rotation: f32 = 0.0; // highp
    //if (u_rotate_symbol) {
    //    // Point labels with 'rotation-alignment: map' are horizontal with respect to tile units
    //    // To figure out that angle in projected space, we draw a short horizontal line in tile
    //    // space, project it, and measure its angle in projected space.
    //    vec4 offsetProjectedPoint = u_matrix * vec4(a_pos + vec2(1, 0), 0, 1);

    //    vec2 a = projectedPoint.xy / projectedPoint.w;
    //    vec2 b = offsetProjectedPoint.xy / offsetProjectedPoint.w;

    //    symbol_rotation = atan((b.y - a.y) / u_aspect_ratio, b.x - a.x);
    //}

     let angle_sin: f32 = sin(segment_angle + symbol_rotation); // highp
     let angle_cos: f32  = cos(segment_angle + symbol_rotation); // highp
    let rotation_matrix: mat2x2<f32> = mat2x2<f32>(angle_cos, -1.0 * angle_sin, angle_sin, angle_cos);

    let projected_pos: vec4<f32> = u_label_plane_matrix * vec4(a_projected_pos.xy, 0.0, 1.0);
    let gl_Position = u_coord_matrix * vec4(projected_pos.xy / projected_pos.w + rotation_matrix * (vec2<f32>(a_offset) / 32.0 * fontScale + vec2<f32>(a_pxoffset)), 0.0, 1.0);
    let gamma_scale: f32 = gl_Position.w;

    // TODO let fade_opacity: vec4<f32> = unpack_opacity(a_fade_opacity);
    let fade_opacity: vec4<f32> = vec4<f32>(1.0,1.0,1.0,1.0);
    let fade_change: f32  = select(-u_fade_change, u_fade_change, fade_opacity[1] > 0.5);
    let interpolated_fade_opacity: f32  = max(0.0, min(1.0, fade_opacity[0] + fade_change));


    let v_data0: vec2<f32> = vec2<f32>(a_tex) / u_texsize;
    let v_data1 = vec3<f32>(gamma_scale, size, interpolated_fade_opacity);


    //var final_position = mat4x4<f32>(translate1, translate2, translate3, translate4) * vec4<f32>(a_projected_pos + vec3<f32>(vec2<f32>(a_offset) / 32.0 * fontScale + vec2<f32>(a_pxoffset), 0.0), 1.0);
    var final_position =  mat4x4<f32>(translate1, translate2, translate3, translate4)*gl_Position;
    final_position.z = 10.0;
    return VertexOutput(v_data0, v_data1, final_position);
}