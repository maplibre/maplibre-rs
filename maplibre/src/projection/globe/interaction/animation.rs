//! Globe-specific camera targets and animation sampling.

use crate::{
    coords::LatLon,
    projection::globe::{
        globe_distance_pixels, globe_zoom_adjustment, interpolate_lat_lon, wrap_longitude,
    },
};

/// Center and zoom produced by globe camera targeting.
#[derive(Clone, Copy, Debug)]
pub struct GlobeCameraTarget {
    /// Geographic camera center.
    pub center: LatLon,
    /// Mercator zoom preserving the requested apparent globe size.
    pub zoom: f64,
}

/// Resolves `jumpTo` center and zoom semantics for globe projection.
pub fn jump_to_target(
    start_center: LatLon,
    start_zoom: f64,
    target_center: LatLon,
    requested_zoom: Option<f64>,
) -> GlobeCameraTarget {
    GlobeCameraTarget {
        center: wrapped(target_center),
        zoom: requested_zoom.unwrap_or_else(|| {
            start_zoom + globe_zoom_adjustment(start_center.latitude, target_center.latitude)
        }),
    }
}

/// Prepared globe `easeTo` interpolation.
#[derive(Clone, Copy, Debug)]
pub struct GlobeEase {
    start_center: LatLon,
    target: GlobeCameraTarget,
    normalized_start_zoom: f64,
    normalized_target_zoom: f64,
    longitude_delta: f64,
    latitude_delta: f64,
}

impl GlobeEase {
    /// Creates an ease target, compensating omitted zoom for latitude.
    pub fn new(
        start_center: LatLon,
        start_zoom: f64,
        target_center: LatLon,
        requested_zoom: Option<f64>,
    ) -> Self {
        let target = jump_to_target(start_center, start_zoom, target_center, requested_zoom);
        Self {
            start_center,
            target,
            normalized_start_zoom: normalized_zoom(start_zoom, start_center.latitude),
            normalized_target_zoom: normalized_zoom(target.zoom, target.center.latitude),
            longitude_delta: shortest_angle_delta(start_center.longitude, target.center.longitude),
            latitude_delta: shortest_angle_delta(start_center.latitude, target.center.latitude),
        }
    }

    /// Returns the final center and zoom.
    pub fn target(&self) -> GlobeCameraTarget {
        self.target
    }

    /// Samples the same latitude-aware center and normalized-zoom curve as GL JS.
    pub fn sample(&self, progress: f64) -> GlobeCameraTarget {
        let progress = progress.clamp(0.0, 1.0);
        let scale = 2_f64.powf(self.normalized_target_zoom - self.normalized_start_zoom);
        let base = if self.normalized_target_zoom > self.normalized_start_zoom {
            scale.min(2.0)
        } else {
            scale.max(0.5)
        };
        let center_progress = progress * base.powf(1.0 - progress);
        let center = wrapped(interpolate_lat_lon(
            self.start_center,
            self.longitude_delta,
            self.latitude_delta,
            center_progress,
        ));
        let normalized_zoom = self.normalized_start_zoom
            + (self.normalized_target_zoom - self.normalized_start_zoom) * progress;
        GlobeCameraTarget {
            center,
            zoom: denormalized_zoom(normalized_zoom, center.latitude),
        }
    }
}

/// Prepared globe `flyTo` interpolation parameters.
#[derive(Clone, Copy, Debug)]
pub struct GlobeFly {
    start_center: LatLon,
    target: GlobeCameraTarget,
    normalized_start_zoom: f64,
    longitude_delta: f64,
    latitude_delta: f64,
    /// Zoom scale between the normalized start and target zooms.
    pub scale_of_zoom: f64,
    /// Lowest permitted normalized scale used by the flight curve.
    pub scale_of_min_zoom: f64,
    /// Great-circle path length in screen pixels.
    pub pixel_path_length: f64,
}

impl GlobeFly {
    /// Creates globe flight parameters after caller-side center constraints and offset handling.
    pub fn new(
        world_size: f64,
        start_center: LatLon,
        start_zoom: f64,
        target_center: LatLon,
        requested_zoom: Option<f64>,
        minimum_zoom: f64,
    ) -> Self {
        let target = jump_to_target(start_center, start_zoom, target_center, requested_zoom);
        let normalized_start_zoom = normalized_zoom(start_zoom, start_center.latitude);
        let normalized_target_zoom = normalized_zoom(target.zoom, target.center.latitude);
        let normalized_min_zoom = normalized_zoom(minimum_zoom, target.center.latitude)
            .min(normalized_start_zoom)
            .min(normalized_target_zoom);
        Self {
            start_center,
            target,
            normalized_start_zoom,
            longitude_delta: shortest_angle_delta(start_center.longitude, target.center.longitude),
            latitude_delta: shortest_angle_delta(start_center.latitude, target.center.latitude),
            scale_of_zoom: 2_f64.powf(normalized_target_zoom - normalized_start_zoom),
            scale_of_min_zoom: 2_f64.powf(normalized_min_zoom - normalized_start_zoom),
            pixel_path_length: globe_distance_pixels(
                world_size,
                start_center.latitude,
                start_center,
                target.center,
            ),
        }
    }

    /// Returns the final center and zoom.
    pub fn target(&self) -> GlobeCameraTarget {
        self.target
    }

    /// Samples the globe-specific part of GL JS's flight curve.
    pub fn sample(&self, progress: f64, scale: f64, center_progress: f64) -> GlobeCameraTarget {
        if progress >= 1.0 {
            return self.target;
        }
        let center = wrapped(interpolate_lat_lon(
            self.start_center,
            self.longitude_delta,
            self.latitude_delta,
            center_progress,
        ));
        let normalized_zoom = self.normalized_start_zoom + scale.log2();
        GlobeCameraTarget {
            center,
            zoom: denormalized_zoom(normalized_zoom, center.latitude),
        }
    }
}

fn normalized_zoom(zoom: f64, latitude: f64) -> f64 {
    zoom + globe_zoom_adjustment(latitude, 0.0)
}

fn denormalized_zoom(zoom: f64, latitude: f64) -> f64 {
    zoom + globe_zoom_adjustment(0.0, latitude)
}

fn shortest_angle_delta(start: f64, end: f64) -> f64 {
    wrap_longitude(end - start)
}

fn wrapped(location: LatLon) -> LatLon {
    LatLon::new(location.latitude, wrap_longitude(location.longitude))
}

#[cfg(test)]
mod tests;
