struct VertexInput {
    @location(0) view_direction: vec3<f32>,
    @location(1) @interpolate(flat) globe_position: vec3<f32>,
    @location(2) @interpolate(flat) sun_direction: vec3<f32>,
    @location(3) @interpolate(flat) globe_radius: f32,
    @location(4) @interpolate(flat) atmosphere_blend: f32,
};

const ATMOSPHERE_PI: f32 = 3.141592653589793;
const PRIMARY_STEPS: u32 = 5u;
const SECONDARY_STEPS: u32 = 3u;
const EARTH_RADIUS: f32 = 6371000.0;
const ATMOSPHERE_RADIUS: f32 = 6471000.0;
const RAYLEIGH_COEFFICIENT: vec3<f32> = vec3<f32>(5.5e-6, 13.0e-6, 22.4e-6);
const MIE_COEFFICIENT: f32 = 21.0e-6;
const RAYLEIGH_SCALE_HEIGHT: f32 = 8000.0;
const MIE_SCALE_HEIGHT: f32 = 1200.0;
const MIE_DIRECTION: f32 = 0.758;

fn ray_sphere_intersection(
    ray_origin: vec3<f32>,
    ray_direction: vec3<f32>,
    radius: f32,
) -> vec2<f32> {
    let a = dot(ray_direction, ray_direction);
    let b = 2.0 * dot(ray_direction, ray_origin);
    let c = dot(ray_origin, ray_origin) - radius * radius;
    let discriminant = b * b - 4.0 * a * c;
    if discriminant < 0.0 {
        return vec2<f32>(1e5, -1e5);
    }
    let root = sqrt(discriminant);
    return vec2<f32>((-b - root) / (2.0 * a), (-b + root) / (2.0 * a));
}

fn scattering(
    raw_ray_direction: vec3<f32>,
    ray_origin: vec3<f32>,
    raw_sun_direction: vec3<f32>,
) -> vec4<f32> {
    let sun_direction = normalize(raw_sun_direction);
    let ray_direction = normalize(raw_ray_direction);
    var primary_range = ray_sphere_intersection(
        ray_origin,
        ray_direction,
        ATMOSPHERE_RADIUS,
    );
    if primary_range.x > primary_range.y {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }
    primary_range.x = max(primary_range.x, 0.0);
    let planet_range = ray_sphere_intersection(ray_origin, ray_direction, EARTH_RADIUS);
    if planet_range.x <= planet_range.y && planet_range.x > 0.0 {
        primary_range.y = min(primary_range.y, planet_range.x);
    }

    let primary_step_size = (primary_range.y - primary_range.x) / f32(PRIMARY_STEPS);
    var primary_time = primary_range.x + primary_step_size * 0.5;
    var total_rayleigh = vec3<f32>(0.0);
    var total_mie = vec3<f32>(0.0);
    var primary_depth_rayleigh = 0.0;
    var primary_depth_mie = 0.0;

    let mu = dot(ray_direction, sun_direction);
    let mu_squared = mu * mu;
    let g_squared = MIE_DIRECTION * MIE_DIRECTION;
    let phase_rayleigh = 3.0 / (16.0 * ATMOSPHERE_PI) * (1.0 + mu_squared);
    let phase_mie = 3.0 / (8.0 * ATMOSPHERE_PI)
        * ((1.0 - g_squared) * (mu_squared + 1.0))
        / (pow(1.0 + g_squared - 2.0 * mu * MIE_DIRECTION, 1.5)
            * (2.0 + g_squared));

    for (var primary_index = 0u; primary_index < PRIMARY_STEPS; primary_index++) {
        let primary_position = ray_origin + ray_direction * primary_time;
        let primary_height = length(primary_position) - EARTH_RADIUS;
        let depth_step_rayleigh = exp(-primary_height / RAYLEIGH_SCALE_HEIGHT)
            * primary_step_size;
        let depth_step_mie = exp(-primary_height / MIE_SCALE_HEIGHT) * primary_step_size;
        primary_depth_rayleigh += depth_step_rayleigh;
        primary_depth_mie += depth_step_mie;

        let secondary_step_size = ray_sphere_intersection(
            primary_position,
            sun_direction,
            ATMOSPHERE_RADIUS,
        ).y / f32(SECONDARY_STEPS);
        var secondary_time = secondary_step_size * 0.5;
        var secondary_depth_rayleigh = 0.0;
        var secondary_depth_mie = 0.0;
        for (var secondary_index = 0u;
            secondary_index < SECONDARY_STEPS;
            secondary_index++) {
            let secondary_position = primary_position + sun_direction * secondary_time;
            let secondary_height = length(secondary_position) - EARTH_RADIUS;
            secondary_depth_rayleigh += exp(-secondary_height / RAYLEIGH_SCALE_HEIGHT)
                * secondary_step_size;
            secondary_depth_mie += exp(-secondary_height / MIE_SCALE_HEIGHT)
                * secondary_step_size;
            secondary_time += secondary_step_size;
        }

        let attenuation = exp(-(
            MIE_COEFFICIENT * (primary_depth_mie + secondary_depth_mie)
            + RAYLEIGH_COEFFICIENT
                * (primary_depth_rayleigh + secondary_depth_rayleigh)
        ));
        total_rayleigh += depth_step_rayleigh * attenuation;
        total_mie += depth_step_mie * attenuation;
        primary_time += primary_step_size;
    }

    let opacity = exp(-(
        length(RAYLEIGH_COEFFICIENT) * length(total_rayleigh)
        + MIE_COEFFICIENT * length(total_mie)
    ));
    let color = 22.0 * (
        phase_rayleigh * RAYLEIGH_COEFFICIENT * total_rayleigh
        + phase_mie * MIE_COEFFICIENT * total_mie
    );
    return vec4<f32>(color, opacity);
}

@fragment
fn main(in: VertexInput) -> @location(0) vec4<f32> {
    let camera_relative_to_globe = -in.globe_position * EARTH_RADIUS / in.globe_radius;
    let raw_color = scattering(
        normalize(in.view_direction),
        camera_relative_to_globe,
        in.sun_direction,
    );
    let exposed = vec3<f32>(1.0) - exp(-raw_color.rgb);
    let gamma = 1.0 / 2.2;
    let color = pow(max(exposed, vec3<f32>(0.0)), vec3<f32>(gamma));
    let alpha = 1.0 - pow(clamp(raw_color.a, 0.0, 1.0), gamma);
    return vec4<f32>(color, alpha) * in.atmosphere_blend;
}
