struct ShaderProjectionData {
    main_matrix: mat4x4<f32>,
    clipping_plane: vec4<f32>,
    transition_and_padding: vec4<f32>,
};

struct ProjectedTilePosition {
    clip_position: vec4<f32>,
    horizon_distance: f32,
};

@group(0) @binding(0) var<uniform> projection: ShaderProjectionData;

const PROJECTION_PI: f32 = 3.141592653589793;
const PROJECTION_TWO_PI: f32 = 6.283185307179586;
const GLOBE_Z_CLIPPING_START: f32 = 0.2;
const POLE_TRANSITION_START: f32 = 0.98;

fn tile_position_on_unit_sphere(
    tile_position: vec2<f32>,
    tile_mercator_coords: vec4<f32>,
) -> vec3<f32> {
    let mercator = tile_mercator_coords.xy + tile_position * tile_mercator_coords.zw;
    let longitude = mercator.x * PROJECTION_TWO_PI + PROJECTION_PI;
    let tangent_half_latitude = exp(PROJECTION_PI - mercator.y * PROJECTION_TWO_PI);
    let tangent_half_latitude_squared = tangent_half_latitude * tangent_half_latitude;
    let denominator = tangent_half_latitude_squared + 1.0;
    let sin_latitude = (tangent_half_latitude_squared - 1.0) / denominator;
    let cos_latitude = 2.0 * tangent_half_latitude / denominator;
    var surface = vec3<f32>(
        sin(longitude) * cos_latitude,
        sin_latitude,
        cos(longitude) * cos_latitude,
    );
    if tile_position.y < -32767.5 {
        surface = vec3<f32>(0.0, 1.0, 0.0);
    } else if tile_position.y > 32766.5 {
        surface = vec3<f32>(0.0, -1.0, 0.0);
    }
    return surface;
}

fn globe_circumference_ratio_at_tile_y(
    tile_y: f32,
    tile_mercator_coords: vec4<f32>,
) -> f32 {
    let mercator_y = tile_mercator_coords.y + tile_mercator_coords.w * tile_y;
    let tangent_half_latitude = exp(PROJECTION_PI - mercator_y * PROJECTION_TWO_PI);
    return (2.0 * tangent_half_latitude) /
        (tangent_half_latitude * tangent_half_latitude + 1.0);
}

fn project_symbol_scale(tile_y: f32, tile_mercator_coords: vec4<f32>) -> f32 {
    let circumference = max(
        globe_circumference_ratio_at_tile_y(tile_y, tile_mercator_coords),
        1e-6,
    );
    return mix(1.0, 1.0 / circumference, projection.transition_and_padding.x);
}

fn globe_horizon_distance(
    surface: vec3<f32>,
    transition: f32,
    is_pole: bool,
) -> f32 {
    let clipping_transition = clamp(
        (transition - GLOBE_Z_CLIPPING_START) / (1.0 - GLOBE_Z_CLIPPING_START),
        0.0,
        1.0,
    );
    let surface_distance = mix(
        1.0,
        dot(projection.clipping_plane, vec4<f32>(surface, 1.0)),
        clipping_transition,
    );
    if !is_pole {
        return surface_distance;
    }
    let pole_transition = clamp(
        (transition - POLE_TRANSITION_START) / (1.0 - POLE_TRANSITION_START),
        0.0,
        1.0,
    );
    return mix(-1.0, surface_distance, pow(pole_transition, 8.0));
}

fn interpolate_clip_position(
    mercator_clip: vec4<f32>,
    globe_clip: vec4<f32>,
    transition: f32,
) -> vec4<f32> {
    var result = globe_clip;
    result.x = mix(mercator_clip.x, globe_clip.x, transition);
    result.y = mix(mercator_clip.y, globe_clip.y, transition);
    result.w = mix(mercator_clip.w, globe_clip.w, transition);
    let z_transition = clamp(
        (transition - GLOBE_Z_CLIPPING_START) / (1.0 - GLOBE_Z_CLIPPING_START),
        0.0,
        1.0,
    );
    result.z = mix(0.0, globe_clip.z, z_transition);
    return result;
}

fn project_tile_position(
    tile_position: vec3<f32>,
    fallback_matrix: mat4x4<f32>,
    tile_mercator_coords: vec4<f32>,
) -> ProjectedTilePosition {
    let transition = projection.transition_and_padding.x;
    let surface = tile_position_on_unit_sphere(tile_position.xy, tile_mercator_coords);
    let mercator_clip = fallback_matrix * vec4<f32>(tile_position, 1.0);
    let globe_clip = projection.main_matrix * vec4<f32>(surface, 1.0);
    let is_pole = tile_position.y < -32767.5 || tile_position.y > 32766.5;
    let horizon_distance = globe_horizon_distance(
        surface,
        transition,
        is_pole,
    );
    return ProjectedTilePosition(
        interpolate_clip_position(mercator_clip, globe_clip, transition),
        horizon_distance,
    );
}

fn project_tile_mesh_position(
    tile_position: vec3<f32>,
    raw_position: vec2<i32>,
    fallback_matrix: mat4x4<f32>,
    tile_mercator_coords: vec4<f32>,
) -> ProjectedTilePosition {
    var surface = tile_position_on_unit_sphere(tile_position.xy, tile_mercator_coords);
    if raw_position.y < -32767 {
        surface = vec3<f32>(0.0, 1.0, 0.0);
    } else if raw_position.y > 32766 {
        surface = vec3<f32>(0.0, -1.0, 0.0);
    }
    let transition = projection.transition_and_padding.x;
    let mercator_clip = fallback_matrix * vec4<f32>(tile_position, 1.0);
    let globe_clip = projection.main_matrix * vec4<f32>(surface, 1.0);
    let is_pole = raw_position.y < -32767 || raw_position.y > 32766;
    let horizon_distance = globe_horizon_distance(
        surface,
        transition,
        is_pole,
    );
    return ProjectedTilePosition(
        interpolate_clip_position(mercator_clip, globe_clip, transition),
        horizon_distance,
    );
}
