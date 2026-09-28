//! Map projection definitions and coordinate conversion.

use serde::{Deserialize, Serialize};

mod expression;
pub mod globe;
pub mod renderer_data;

pub use expression::{
    InterpolationCurve, NamedProjection, ProjectionExpression, ProjectionExpressionError,
    ProjectionStop,
};
pub use globe::project_tile_coordinates_to_unit_sphere;

/// A projection selected by a style.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(
    from = "expression::ProjectionWire",
    into = "expression::ProjectionWire"
)]
pub enum ProjectionType {
    /// Render Web Mercator tiles on a plane.
    #[default]
    Mercator,
    /// Render Web Mercator tiles on a globe.
    Globe,
    /// Keep vertical-perspective globe rendering at every zoom level.
    VerticalPerspective,
    /// Evaluate a validated zoom expression or explicit transition.
    Expression(ProjectionExpression),
}

impl ProjectionType {
    /// Returns the active globe weight at the given continuous zoom.
    pub fn globe_transition(&self, zoom: f64) -> f32 {
        match self {
            Self::Mercator => 0.0,
            Self::Globe => globe::transition_for_zoom(zoom),
            Self::VerticalPerspective => 1.0,
            Self::Expression(expression) => expression.globe_transition(zoom),
        }
    }

    /// Returns whether globe rendering contributes to the current frame.
    pub fn uses_globe_rendering(&self, zoom: f64) -> bool {
        self.globe_transition(zoom) > 0.0
    }

    /// Creates an explicit transition between two projections.
    pub fn transition(
        from: NamedProjection,
        to: NamedProjection,
        transition: f64,
    ) -> Result<Self, ProjectionExpressionError> {
        expression::ProjectionExpression::transition(from, to, transition).map(Self::Expression)
    }

    /// Creates a zoom interpolation from validated stops.
    pub fn interpolate(
        curve: InterpolationCurve,
        stops: Vec<ProjectionStop>,
    ) -> Result<Self, ProjectionExpressionError> {
        expression::ProjectionExpression::interpolate(curve, stops).map(Self::Expression)
    }

    /// Creates a piecewise-constant zoom expression.
    pub fn step(
        default: NamedProjection,
        stops: Vec<ProjectionStop>,
    ) -> Result<Self, ProjectionExpressionError> {
        expression::ProjectionExpression::step(default, stops).map(Self::Expression)
    }
}

/// The root-level projection section of a style document.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ProjectionSpecification {
    /// Projection used to display the map.
    #[serde(rename = "type")]
    pub projection_type: ProjectionType,
}

#[cfg(test)]
mod tests;
