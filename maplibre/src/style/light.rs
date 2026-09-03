//! Root light properties used by atmosphere and lit globe rendering.

use cgmath::{InnerSpace, Vector3};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::layer::StyleProperty;
use crate::projection::globe::camera::GlobeCameraState;

const DEFAULT_POSITION: [f64; 3] = [1.15, 210.0, 30.0];
const MIN_DIRECTION_LENGTH_SQUARED: f64 = 1e-24;

/// Coordinate frame in which the root light position is expressed.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LightAnchor {
    /// Light remains fixed relative to the viewport.
    #[default]
    Viewport,
    /// Light rotates with the geographic map.
    Map,
}

/// Root light configuration relevant to globe rendering.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LightSpecification {
    /// Coordinate frame for the light position.
    #[serde(default)]
    pub anchor: LightAnchor,
    /// Spherical position as radius, azimuth, and polar angle.
    #[serde(default = "default_position_property")]
    pub position: StyleProperty<[f64; 3]>,
}

impl Default for LightSpecification {
    fn default() -> Self {
        Self {
            anchor: LightAnchor::Viewport,
            position: default_position_property(),
        }
    }
}

/// Invalid root light data used by the atmosphere pass.
#[derive(Clone, Copy, Debug, Error, PartialEq)]
pub enum LightError {
    /// The light position cannot produce a finite direction.
    #[error("light position must contain a positive radius and finite angles")]
    InvalidPosition,
    /// The evaluated light expression has an unsupported form.
    #[error("light position expression cannot be evaluated at zoom {zoom}")]
    UnsupportedPositionExpression {
        /// Zoom at which evaluation failed.
        zoom: f64,
    },
    /// A valid light direction collapsed under the camera transform.
    #[error("light direction cannot be transformed into the current view")]
    InvalidViewDirection,
}

impl LightSpecification {
    /// Evaluates the light and returns the sun direction in camera-view axes.
    pub fn sun_direction_in_view(
        &self,
        camera: &GlobeCameraState,
        zoom: f64,
    ) -> Result<Vector3<f64>, LightError> {
        let position = evaluate_position(&self.position, zoom)?;
        let cartesian = spherical_to_cartesian(position)?;
        let sun = -cartesian;
        match self.anchor {
            LightAnchor::Viewport => normalize(sun),
            LightAnchor::Map => camera
                .world_direction_to_view(sun)
                .ok_or(LightError::InvalidViewDirection),
        }
    }
}

fn default_position_property() -> StyleProperty<[f64; 3]> {
    StyleProperty::Constant(DEFAULT_POSITION)
}

fn evaluate_position(
    property: &StyleProperty<[f64; 3]>,
    zoom: f64,
) -> Result<[f64; 3], LightError> {
    match property {
        StyleProperty::Constant(position) => Ok(*position),
        StyleProperty::Expression(expression) => evaluate_position_expression(expression, zoom)
            .ok_or(LightError::UnsupportedPositionExpression { zoom }),
    }
}

fn evaluate_position_expression(value: &serde_json::Value, zoom: f64) -> Option<[f64; 3]> {
    let stops = value.get("stops")?.as_array()?;
    let parsed = stops
        .iter()
        .map(|stop| {
            let pair = stop.as_array()?;
            Some((pair.first()?.as_f64()?, parse_position(pair.get(1)?)?))
        })
        .collect::<Option<Vec<_>>>()?;
    if parsed.is_empty() || !parsed.windows(2).all(|pair| pair[0].0 < pair[1].0) {
        return None;
    }
    if zoom <= parsed[0].0 {
        return Some(parsed[0].1);
    }
    for pair in parsed.windows(2) {
        if zoom <= pair[1].0 {
            let amount = (zoom - pair[0].0) / (pair[1].0 - pair[0].0);
            return Some(interpolate_position(pair[0].1, pair[1].1, amount));
        }
    }
    parsed.last().map(|stop| stop.1)
}

fn parse_position(value: &serde_json::Value) -> Option<[f64; 3]> {
    let values = value.as_array()?;
    (values.len() == 3).then(|| {
        Some([
            values[0].as_f64()?,
            values[1].as_f64()?,
            values[2].as_f64()?,
        ])
    })?
}

fn interpolate_position(from: [f64; 3], to: [f64; 3], amount: f64) -> [f64; 3] {
    [
        from[0] + (to[0] - from[0]) * amount,
        from[1] + (to[1] - from[1]) * amount,
        from[2] + (to[2] - from[2]) * amount,
    ]
}

fn spherical_to_cartesian(position: [f64; 3]) -> Result<Vector3<f64>, LightError> {
    let [radius, azimuth_degrees, polar_degrees] = position;
    if !radius.is_finite()
        || radius <= 0.0
        || !azimuth_degrees.is_finite()
        || !polar_degrees.is_finite()
    {
        return Err(LightError::InvalidPosition);
    }
    let azimuth = (azimuth_degrees + 90.0).to_radians();
    let polar = polar_degrees.to_radians();
    normalize(Vector3::new(
        radius * azimuth.cos() * polar.sin(),
        radius * azimuth.sin() * polar.sin(),
        radius * polar.cos(),
    ))
}

fn normalize(direction: Vector3<f64>) -> Result<Vector3<f64>, LightError> {
    (direction.magnitude2().is_finite() && direction.magnitude2() > MIN_DIRECTION_LENGTH_SQUARED)
        .then(|| direction.normalize())
        .ok_or(LightError::InvalidPosition)
}

#[cfg(test)]
mod tests;
