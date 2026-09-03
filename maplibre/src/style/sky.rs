//! Sky properties used by globe rendering.

use serde::{Deserialize, Serialize};

use super::layer::StyleProperty;

/// Root-level sky configuration.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SkySpecification {
    /// Opacity of atmospheric scattering around a globe.
    #[serde(
        rename = "atmosphere-blend",
        default,
        deserialize_with = "StyleProperty::<f32>::deserialize_f32_or_none",
        skip_serializing_if = "Option::is_none"
    )]
    pub atmosphere_blend: Option<StyleProperty<f32>>,
}

impl SkySpecification {
    /// Evaluates atmospheric opacity at a continuous camera zoom.
    pub fn atmosphere_blend_at_zoom(&self, zoom: f64) -> f32 {
        let value = match self.atmosphere_blend.as_ref() {
            Some(StyleProperty::Constant(value)) => *value,
            Some(StyleProperty::Expression(expression)) => {
                evaluate_expression(expression, zoom).unwrap_or(0.0)
            }
            None => 0.0,
        };
        if value.is_finite() {
            value.clamp(0.0, 1.0)
        } else {
            0.0
        }
    }
}

fn evaluate_expression(expression: &serde_json::Value, zoom: f64) -> Option<f32> {
    let values = expression.as_array()?;
    match values.first()?.as_str()? {
        "interpolate" if is_zoom_input(values.get(2)) => interpolate_stops(values.get(3..)?, zoom),
        "step" if is_zoom_input(values.get(1)) => {
            let default = values.get(2)?.as_f64()? as f32;
            step_stops(default, values.get(3..)?, zoom)
        }
        _ => None,
    }
}

fn interpolate_stops(values: &[serde_json::Value], zoom: f64) -> Option<f32> {
    let stops = numeric_stops(values)?;
    let first = *stops.first()?;
    if zoom <= first.0 {
        return Some(first.1);
    }
    for pair in stops.windows(2) {
        if zoom <= pair[1].0 {
            let amount = ((zoom - pair[0].0) / (pair[1].0 - pair[0].0)) as f32;
            return Some(pair[0].1 + (pair[1].1 - pair[0].1) * amount);
        }
    }
    stops.last().map(|stop| stop.1)
}

fn step_stops(default: f32, values: &[serde_json::Value], zoom: f64) -> Option<f32> {
    let stops = numeric_stops(values)?;
    Some(
        stops
            .iter()
            .rev()
            .find(|stop| zoom >= stop.0)
            .map_or(default, |stop| stop.1),
    )
}

fn numeric_stops(values: &[serde_json::Value]) -> Option<Vec<(f64, f32)>> {
    if values.len() < 4 || !values.len().is_multiple_of(2) {
        return None;
    }
    let stops = values
        .chunks_exact(2)
        .map(|pair| Some((pair[0].as_f64()?, pair[1].as_f64()? as f32)))
        .collect::<Option<Vec<_>>>()?;
    stops
        .windows(2)
        .all(|pair| pair[0].0 < pair[1].0)
        .then_some(stops)
}

fn is_zoom_input(value: Option<&serde_json::Value>) -> bool {
    value
        .and_then(serde_json::Value::as_array)
        .is_some_and(|input| {
            input.len() == 1 && input.first().and_then(serde_json::Value::as_str) == Some("zoom")
        })
}

#[cfg(test)]
mod tests;
