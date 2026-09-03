//! Serde wire format for projection expressions.

use serde::{de::Error as DeserializeError, Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

use super::{
    InterpolationCurve, NamedProjection, ProjectionExpression, ProjectionExpressionError,
    ProjectionExpressionKind, ProjectionStop,
};
use crate::projection::ProjectionType;

#[derive(Clone, Debug)]
pub(crate) struct ProjectionWire(ProjectionType);

impl From<ProjectionWire> for ProjectionType {
    fn from(value: ProjectionWire) -> Self {
        value.0
    }
}

impl From<ProjectionType> for ProjectionWire {
    fn from(value: ProjectionType) -> Self {
        Self(value)
    }
}

impl<'de> Deserialize<'de> for ProjectionWire {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        parse_projection(&value).map(Self).map_err(D::Error::custom)
    }
}

impl Serialize for ProjectionWire {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        projection_value(&self.0).serialize(serializer)
    }
}

fn parse_projection(value: &Value) -> Result<ProjectionType, ProjectionExpressionError> {
    if let Some(name) = value.as_str() {
        return parse_projection_type(name);
    }
    let values = value
        .as_array()
        .ok_or_else(|| unsupported("projection must be a name or expression array"))?;
    if values.len() == 3 && values.first().and_then(Value::as_str) != Some("step") {
        return parse_transition(values);
    }
    match values.first().and_then(Value::as_str) {
        Some("interpolate") => parse_interpolate(values),
        Some("step") => parse_step(values),
        _ => Err(unsupported("unknown projection expression operator")),
    }
}

fn parse_projection_type(name: &str) -> Result<ProjectionType, ProjectionExpressionError> {
    match name {
        "mercator" => Ok(ProjectionType::Mercator),
        "globe" => Ok(ProjectionType::Globe),
        "vertical-perspective" => Ok(ProjectionType::VerticalPerspective),
        _ => Err(unsupported("unknown projection name")),
    }
}

fn parse_named(value: &Value) -> Result<NamedProjection, ProjectionExpressionError> {
    match value.as_str() {
        Some("mercator") => Ok(NamedProjection::Mercator),
        Some("vertical-perspective") => Ok(NamedProjection::VerticalPerspective),
        _ => Err(unsupported("expression outputs must be named projections")),
    }
}

fn parse_transition(values: &[Value]) -> Result<ProjectionType, ProjectionExpressionError> {
    let from = values
        .first()
        .map(parse_named)
        .transpose()?
        .ok_or_else(|| unsupported("missing transition source"))?;
    let to = values
        .get(1)
        .map(parse_named)
        .transpose()?
        .ok_or_else(|| unsupported("missing transition target"))?;
    let transition = values
        .get(2)
        .and_then(Value::as_f64)
        .ok_or_else(|| invalid_number("transition must be numeric"))?;
    ProjectionType::transition(from, to, transition)
}

fn parse_interpolate(values: &[Value]) -> Result<ProjectionType, ProjectionExpressionError> {
    if values.len() < 7 || values.len().is_multiple_of(2) || !is_zoom_input(values.get(2)) {
        return Err(unsupported("invalid interpolate expression shape"));
    }
    let curve = parse_curve(
        values
            .get(1)
            .ok_or_else(|| unsupported("missing interpolation curve"))?,
    )?;
    ProjectionType::interpolate(curve, parse_stops(&values[3..])?)
}

fn parse_step(values: &[Value]) -> Result<ProjectionType, ProjectionExpressionError> {
    if values.len() < 5 || values.len().is_multiple_of(2) || !is_zoom_input(values.get(1)) {
        return Err(unsupported("invalid step expression shape"));
    }
    let default = parse_named(&values[2])?;
    ProjectionType::step(default, parse_stops(&values[3..])?)
}

fn parse_stops(values: &[Value]) -> Result<Vec<ProjectionStop>, ProjectionExpressionError> {
    values
        .chunks_exact(2)
        .map(|pair| {
            let zoom = pair[0]
                .as_f64()
                .ok_or_else(|| invalid_number("zoom stop must be numeric"))?;
            Ok(ProjectionStop::new(zoom, parse_named(&pair[1])?))
        })
        .collect()
}

fn parse_curve(value: &Value) -> Result<InterpolationCurve, ProjectionExpressionError> {
    let parts = value
        .as_array()
        .ok_or_else(|| unsupported("interpolation curve must be an array"))?;
    match parts.first().and_then(Value::as_str) {
        Some("linear") if parts.len() == 1 => Ok(InterpolationCurve::Linear),
        Some("exponential") if parts.len() == 2 => parts[1]
            .as_f64()
            .map(InterpolationCurve::Exponential)
            .ok_or_else(|| invalid_number("exponential base must be numeric")),
        Some("cubic-bezier") if parts.len() == 5 => Ok(InterpolationCurve::CubicBezier(
            curve_number(parts, 1)?,
            curve_number(parts, 2)?,
            curve_number(parts, 3)?,
            curve_number(parts, 4)?,
        )),
        _ => Err(unsupported("unsupported interpolation curve")),
    }
}

fn curve_number(parts: &[Value], index: usize) -> Result<f64, ProjectionExpressionError> {
    parts
        .get(index)
        .and_then(Value::as_f64)
        .ok_or_else(|| invalid_number("cubic-bezier controls must be numeric"))
}

fn is_zoom_input(value: Option<&Value>) -> bool {
    value.and_then(Value::as_array).is_some_and(|parts| {
        parts.len() == 1 && parts.first().and_then(Value::as_str) == Some("zoom")
    })
}

fn invalid_number(reason: &str) -> ProjectionExpressionError {
    ProjectionExpressionError::InvalidNumber {
        reason: reason.to_string(),
    }
}

fn unsupported(reason: &str) -> ProjectionExpressionError {
    ProjectionExpressionError::Unsupported {
        reason: reason.to_string(),
    }
}

fn projection_value(projection: &ProjectionType) -> Value {
    match projection {
        ProjectionType::Mercator => Value::String("mercator".to_string()),
        ProjectionType::Globe => Value::String("globe".to_string()),
        ProjectionType::VerticalPerspective => Value::String("vertical-perspective".to_string()),
        ProjectionType::Expression(expression) => expression_value(expression),
    }
}

fn expression_value(expression: &ProjectionExpression) -> Value {
    match &expression.kind {
        ProjectionExpressionKind::Transition {
            from,
            to,
            transition,
        } => Value::Array(vec![
            named_value(*from),
            named_value(*to),
            Value::from(*transition),
        ]),
        ProjectionExpressionKind::Interpolate { curve, stops } => {
            let mut values = vec![
                Value::String("interpolate".to_string()),
                curve_value(*curve),
                zoom_value(),
            ];
            append_stops(&mut values, stops);
            Value::Array(values)
        }
        ProjectionExpressionKind::Step { default, stops } => {
            let mut values = vec![
                Value::String("step".to_string()),
                zoom_value(),
                named_value(*default),
            ];
            append_stops(&mut values, stops);
            Value::Array(values)
        }
    }
}

fn append_stops(values: &mut Vec<Value>, stops: &[ProjectionStop]) {
    for stop in stops {
        values.push(Value::from(stop.zoom));
        values.push(named_value(stop.projection));
    }
}

fn named_value(projection: NamedProjection) -> Value {
    Value::String(
        match projection {
            NamedProjection::Mercator => "mercator",
            NamedProjection::VerticalPerspective => "vertical-perspective",
        }
        .to_string(),
    )
}

fn zoom_value() -> Value {
    Value::Array(vec![Value::String("zoom".to_string())])
}

fn curve_value(curve: InterpolationCurve) -> Value {
    let values = match curve {
        InterpolationCurve::Linear => vec![Value::String("linear".to_string())],
        InterpolationCurve::Exponential(base) => {
            vec![Value::String("exponential".to_string()), Value::from(base)]
        }
        InterpolationCurve::CubicBezier(x1, y1, x2, y2) => vec![
            Value::String("cubic-bezier".to_string()),
            Value::from(x1),
            Value::from(y1),
            Value::from(x2),
            Value::from(y2),
        ],
    };
    Value::Array(values)
}
