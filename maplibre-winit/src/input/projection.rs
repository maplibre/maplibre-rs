//! Projection-aware coordinate conversion for input handlers.

use cgmath::{Point2, Vector2};
use maplibre::{
    coords::{LatLon, WorldCoords, Zoom},
    projection::globe::{
        camera::GlobeCameraState,
        interaction::{
            pan_camera_by_pixels, pan_center_to_anchor, pan_surface_location,
            zoom::{zoom_around_globe, GlobeZoomInput},
            GlobePanUpdate,
        },
    },
    render::{projection::globe_camera_for_view, view_state::ViewState},
    style::Style,
};

pub fn active_globe_camera(style: &Style, view_state: &ViewState) -> Option<GlobeCameraState> {
    uses_globe(style, view_state)
        .then(|| globe_camera_for_view(view_state).ok())
        .flatten()
}

pub fn uses_globe(style: &Style, view_state: &ViewState) -> bool {
    style.projection.as_ref().is_some_and(|projection| {
        projection
            .projection_type
            .globe_transition(view_state.zoom().value())
            > 0.0
    })
}

pub fn globe_world_at_screen(
    style: &Style,
    view_state: &ViewState,
    screen: Vector2<f64>,
) -> Option<WorldCoords> {
    let camera = active_globe_camera(style, view_state)?;
    let pixel = Point2::new(screen.x, screen.y);
    camera
        .is_point_on_map_surface(pixel)
        .then(|| camera.screen_point_to_location(pixel))
        .flatten()
        .map(|location| WorldCoords::from_lat_lon(location, view_state.zoom()))
}

pub fn center_pixel(view_state: &ViewState) -> Vector2<f64> {
    Vector2::new(view_state.width() / 2.0, view_state.height() / 2.0)
}

/// Pans the globe so the location that was under `cursor - delta` moves under `cursor`.
///
/// Bearing stays fixed, the grab falls back to the view center off the planet and the drag turns
/// into a dial around the pole at high latitudes, matching GL JS drag panning. Returns `false`
/// when the globe is not active so the caller pans the Mercator plane instead.
pub fn pan_globe_by_pixels(
    style: &Style,
    view_state: &mut ViewState,
    cursor: Vector2<f64>,
    delta: Vector2<f64>,
) -> bool {
    let Some(camera) = active_globe_camera(style, view_state) else {
        return false;
    };
    let center = center_pixel(view_state);
    if let Some(update) = pan_camera_by_pixels(
        &camera,
        Point2::new(cursor.x, cursor.y),
        Point2::new(center.x, center.y),
        delta,
    ) {
        apply_pan_update(view_state, update);
    }
    true
}

/// Pans the Mercator plane so the ground under `cursor - delta` moves under `cursor`.
pub fn pan_plane_by_pixels(view_state: &mut ViewState, cursor: Vector2<f64>, delta: Vector2<f64>) {
    let inverted_view_proj = view_state.view_projection().invert();
    let previous = cursor - delta;
    let (Some(previous), Some(current)) = (
        view_state.window_to_world_at_ground(&previous, &inverted_view_proj, false),
        view_state.window_to_world_at_ground(&cursor, &inverted_view_proj, false),
    ) else {
        return;
    };
    let shift = previous - current;
    view_state
        .camera_mut()
        .move_relative(Vector2::new(shift.x, shift.y));
}

/// Zooms the globe around a screen point with GL JS's horizon-safe pointer anchoring.
///
/// Returns `false` when the globe is not active so the caller zooms the Mercator plane instead.
pub fn zoom_globe_around_pixel(
    style: &Style,
    view_state: &mut ViewState,
    screen: Vector2<f64>,
    next_zoom: Zoom,
) -> bool {
    let Some(before) = active_globe_camera(style, view_state) else {
        return false;
    };
    let pixel = Point2::new(screen.x, screen.y);
    let start_center = before.center();
    let pointer_location = pan_surface_location(&before, pixel).unwrap_or(start_center);
    let anchor = before.screen_point_to_location(pixel);
    let previous_zoom = view_state.zoom().value();
    view_state.update_zoom(next_zoom);
    let Some(after) = active_globe_camera(style, view_state) else {
        return true;
    };
    let zoom_after_delta = view_state.zoom().value();
    let exact_center = anchor.zip(after.screen_point_to_location(pixel)).map_or(
        after.center(),
        |(anchor, cursor)| {
            pan_center_to_anchor(after.center(), after.bearing_degrees(), anchor, cursor).center
        },
    );
    let Some(ray_direction) = after.ray_direction_from_pixel(pixel) else {
        set_center(view_state, exact_center, zoom_after_delta);
        return true;
    };
    let update = zoom_around_globe(GlobeZoomInput {
        start_center,
        zoom_after_delta,
        zoom_delta: zoom_after_delta - previous_zoom,
        pointer_location,
        exact_center,
        ray_origin: after.camera_position(),
        ray_direction,
        relative_globe_radius: after.globe_radius_pixels()
            / view_state.width().min(view_state.height()),
    });
    set_center(view_state, update.center, update.zoom);
    true
}

fn apply_pan_update(view_state: &mut ViewState, update: GlobePanUpdate) {
    let zoom = view_state.zoom().value() + update.zoom_adjustment;
    set_center(view_state, update.center, zoom);
}

fn set_center(view_state: &mut ViewState, center: LatLon, zoom: f64) {
    let zoom = Zoom::new(zoom);
    view_state.update_zoom(zoom);
    let center = WorldCoords::from_lat_lon(center, zoom);
    view_state
        .camera_mut()
        .move_to(Point2::new(center.x, center.y));
}
