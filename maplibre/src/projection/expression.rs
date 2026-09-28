//! Projection style-expression parsing and evaluation.

use serde::{Deserialize, Serialize};
use thiserror::Error;

mod wire;

pub(crate) use wire::ProjectionWire;

/// A named projection accepted by projection expressions.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NamedProjection {
    /// Web Mercator plane.
    Mercator,
    /// Vertical-perspective globe.
    VerticalPerspective,
}

impl NamedProjection {
    fn globe_weight(self) -> f64 {
        match self {
            Self::Mercator => 0.0,
            Self::VerticalPerspective => 1.0,
        }
    }
}

/// A zoom stop and its projection output.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProjectionStop {
    zoom: f64,
    projection: NamedProjection,
}

impl ProjectionStop {
    /// Creates a projection expression stop.
    pub fn new(zoom: f64, projection: NamedProjection) -> Self {
        Self { zoom, projection }
    }

    /// Returns the zoom input for this stop.
    pub fn zoom(self) -> f64 {
        self.zoom
    }

    /// Returns the projection output for this stop.
    pub fn projection(self) -> NamedProjection {
        self.projection
    }
}

/// Interpolation applied between adjacent projection stops.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InterpolationCurve {
    /// Linear interpolation.
    Linear,
    /// Exponential interpolation with the supplied positive base.
    Exponential(f64),
    /// Unit cubic Bézier easing with two control points.
    CubicBezier(f64, f64, f64, f64),
}

/// Invalid projection expression construction.
#[derive(Clone, Debug, Error, PartialEq)]
pub enum ProjectionExpressionError {
    /// The expression has an unsupported shape or operator.
    #[error("unsupported projection expression: {reason}")]
    Unsupported {
        /// Why the expression is unsupported.
        reason: String,
    },
    /// A numeric expression parameter is invalid.
    #[error("invalid projection expression number: {reason}")]
    InvalidNumber {
        /// Why the number is invalid.
        reason: String,
    },
    /// Stops are missing or not strictly ordered.
    #[error("invalid projection expression stops: {reason}")]
    InvalidStops {
        /// Why the stops are invalid.
        reason: String,
    },
}

/// A validated projection expression.
#[derive(Clone, Debug, PartialEq)]
pub struct ProjectionExpression {
    kind: ProjectionExpressionKind,
}

#[derive(Clone, Debug, PartialEq)]
enum ProjectionExpressionKind {
    Transition {
        from: NamedProjection,
        to: NamedProjection,
        transition: f64,
    },
    Interpolate {
        curve: InterpolationCurve,
        stops: Vec<ProjectionStop>,
    },
    Step {
        default: NamedProjection,
        stops: Vec<ProjectionStop>,
    },
}

impl ProjectionExpression {
    pub(crate) fn transition(
        from: NamedProjection,
        to: NamedProjection,
        transition: f64,
    ) -> Result<Self, ProjectionExpressionError> {
        if !transition.is_finite() || !(0.0..=1.0).contains(&transition) {
            return Err(invalid_number(
                "transition must be finite and between zero and one",
            ));
        }
        Ok(Self {
            kind: ProjectionExpressionKind::Transition {
                from,
                to,
                transition,
            },
        })
    }

    pub(crate) fn interpolate(
        curve: InterpolationCurve,
        stops: Vec<ProjectionStop>,
    ) -> Result<Self, ProjectionExpressionError> {
        validate_curve(curve)?;
        validate_stops(&stops, 2)?;
        Ok(Self {
            kind: ProjectionExpressionKind::Interpolate { curve, stops },
        })
    }

    pub(crate) fn step(
        default: NamedProjection,
        stops: Vec<ProjectionStop>,
    ) -> Result<Self, ProjectionExpressionError> {
        validate_stops(&stops, 1)?;
        Ok(Self {
            kind: ProjectionExpressionKind::Step { default, stops },
        })
    }

    pub(crate) fn globe_transition(&self, zoom: f64) -> f32 {
        if !zoom.is_finite() {
            return 0.0;
        }
        let weight = match &self.kind {
            ProjectionExpressionKind::Transition {
                from,
                to,
                transition,
            } => mix_weights(*from, *to, *transition),
            ProjectionExpressionKind::Interpolate { curve, stops } => {
                interpolate_stops(*curve, stops, zoom)
            }
            ProjectionExpressionKind::Step { default, stops } => step_stops(*default, stops, zoom),
        };
        weight as f32
    }
}

fn validate_curve(curve: InterpolationCurve) -> Result<(), ProjectionExpressionError> {
    match curve {
        InterpolationCurve::Linear => Ok(()),
        InterpolationCurve::Exponential(base) if base.is_finite() && base > 0.0 => Ok(()),
        InterpolationCurve::CubicBezier(x1, y1, x2, y2)
            if [x1, y1, x2, y2].into_iter().all(f64::is_finite)
                && (0.0..=1.0).contains(&x1)
                && (0.0..=1.0).contains(&x2) =>
        {
            Ok(())
        }
        _ => Err(invalid_number("interpolation curve parameters are invalid")),
    }
}

fn validate_stops(
    stops: &[ProjectionStop],
    minimum: usize,
) -> Result<(), ProjectionExpressionError> {
    if stops.len() < minimum {
        return Err(invalid_stops("not enough stops"));
    }
    if stops.iter().any(|stop| !stop.zoom.is_finite()) {
        return Err(invalid_stops("zoom stops must be finite"));
    }
    if stops.windows(2).any(|pair| pair[0].zoom >= pair[1].zoom) {
        return Err(invalid_stops("zoom stops must be strictly increasing"));
    }
    Ok(())
}

fn mix_weights(from: NamedProjection, to: NamedProjection, transition: f64) -> f64 {
    from.globe_weight() + (to.globe_weight() - from.globe_weight()) * transition
}

fn interpolate_stops(curve: InterpolationCurve, stops: &[ProjectionStop], zoom: f64) -> f64 {
    let Some(first) = stops.first().copied() else {
        return 0.0;
    };
    if zoom <= first.zoom {
        return first.projection.globe_weight();
    }
    for pair in stops.windows(2) {
        if zoom <= pair[1].zoom {
            let linear = (zoom - pair[0].zoom) / (pair[1].zoom - pair[0].zoom);
            return mix_weights(
                pair[0].projection,
                pair[1].projection,
                interpolation_factor(curve, linear),
            );
        }
    }
    stops
        .last()
        .map_or(0.0, |stop| stop.projection.globe_weight())
}

fn step_stops(default: NamedProjection, stops: &[ProjectionStop], zoom: f64) -> f64 {
    stops
        .iter()
        .rev()
        .find(|stop| zoom >= stop.zoom)
        .map_or_else(
            || default.globe_weight(),
            |stop| stop.projection.globe_weight(),
        )
}

fn interpolation_factor(curve: InterpolationCurve, linear: f64) -> f64 {
    match curve {
        InterpolationCurve::Linear => linear,
        InterpolationCurve::Exponential(base) if (base - 1.0).abs() > f64::EPSILON => {
            (base.powf(linear) - 1.0) / (base - 1.0)
        }
        InterpolationCurve::Exponential(_) => linear,
        InterpolationCurve::CubicBezier(x1, y1, x2, y2) => {
            cubic_bezier_y_for_x(linear, x1, y1, x2, y2)
        }
    }
}

fn cubic_bezier_y_for_x(x: f64, x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    let mut lower = 0.0;
    let mut upper = 1.0;
    for _ in 0..24 {
        let t = (lower + upper) * 0.5;
        if cubic_coordinate(t, x1, x2) < x {
            lower = t;
        } else {
            upper = t;
        }
    }
    cubic_coordinate((lower + upper) * 0.5, y1, y2)
}

fn cubic_coordinate(t: f64, first: f64, second: f64) -> f64 {
    let inverse = 1.0 - t;
    3.0 * inverse * inverse * t * first + 3.0 * inverse * t * t * second + t * t * t
}

fn invalid_number(reason: &str) -> ProjectionExpressionError {
    ProjectionExpressionError::InvalidNumber {
        reason: reason.to_string(),
    }
}

fn invalid_stops(reason: &str) -> ProjectionExpressionError {
    ProjectionExpressionError::InvalidStops {
        reason: reason.to_string(),
    }
}

#[cfg(test)]
mod tests;
