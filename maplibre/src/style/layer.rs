//! Vector tile layer drawing utilities.

use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
};

use cint::{Alpha, EncodedSrgb};
use csscolorparser::Color;
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum StyleProperty<T> {
    Constant(T),
    Expression(serde_json::Value),
}

impl<T: std::str::FromStr + Clone> StyleProperty<T> {
    pub fn evaluate(&self, feature_properties: &HashMap<String, String>) -> Option<T> {
        match self {
            StyleProperty::Constant(value) => Some(value.clone()),
            StyleProperty::Expression(expr) => {
                if let Some(arr) = expr.as_array() {
                    if let Some(op) = arr.get(0).and_then(|v| v.as_str()) {
                        if op == "match" && arr.len() > 3 {
                            // Extract the getter e.g. ["get", "ADM0_A3"]
                            if let Some(get_arr) = arr.get(1).and_then(|v| v.as_array()) {
                                if get_arr.get(0).and_then(|v| v.as_str()) == Some("get") {
                                    if let Some(prop_name) = get_arr.get(1).and_then(|v| v.as_str())
                                    {
                                        let feature_val_opt = feature_properties.get(prop_name);

                                        // If property is missing, skip match pairs and return fallback
                                        if feature_val_opt.is_none() {
                                            if let Some(fallback) =
                                                arr.last().and_then(|v| v.as_str())
                                            {
                                                return fallback.parse::<T>().ok();
                                            }
                                            return None;
                                        }

                                        let feature_val = feature_val_opt.unwrap();

                                        // Search the match array pairs
                                        let mut i = 2;
                                        while i < arr.len() - 1 {
                                            if let Some(match_keys) =
                                                arr.get(i).and_then(|v| v.as_array())
                                            {
                                                // Does this feature_val exist in the match keys?
                                                let matches = match_keys.iter().any(|k| {
                                                    k.as_str() == Some(feature_val.as_str())
                                                });
                                                if matches {
                                                    if let Some(color_str) =
                                                        arr.get(i + 1).and_then(|v| v.as_str())
                                                    {
                                                        return color_str.parse::<T>().ok();
                                                    }
                                                }
                                            }
                                            i += 2;
                                        }
                                        // Fallback (last element)
                                        if i == arr.len() - 1 {
                                            if let Some(fallback) =
                                                arr.get(i).and_then(|v| v.as_str())
                                            {
                                                return fallback.parse::<T>().ok();
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                None
            }
        }
    }

    pub fn deserialize_color_or_none<'de, D>(
        deserializer: D,
    ) -> Result<Option<StyleProperty<T>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        // For Color types, allow either a raw color string, or an expression value.
        let v = serde_json::Value::deserialize(deserializer).map_err(serde::de::Error::custom)?;
        if let Some(s) = v.as_str() {
            if let Ok(color) = s.parse::<T>() {
                return Ok(Some(StyleProperty::Constant(color)));
            }
        }
        // If it's a structural generic expression like match arrays
        if v.is_array() {
            return Ok(Some(StyleProperty::Expression(v)));
        }
        Ok(None)
    }
}

impl StyleProperty<f32> {
    pub fn deserialize_f32_or_none<'de, D>(
        deserializer: D,
    ) -> Result<Option<StyleProperty<f32>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v = serde_json::Value::deserialize(deserializer).map_err(serde::de::Error::custom)?;
        if let Some(f) = v.as_f64() {
            return Ok(Some(StyleProperty::Constant(f as f32)));
        }
        if v.is_array() {
            return Ok(Some(StyleProperty::Expression(v)));
        }
        Ok(None)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BackgroundPaint {
    #[serde(rename = "background-color")]
    #[serde(
        default,
        deserialize_with = "StyleProperty::<Color>::deserialize_color_or_none"
    )]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_color: Option<StyleProperty<Color>>,
    // TODO a lot
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FillPaint {
    #[serde(rename = "fill-color")]
    #[serde(
        default,
        deserialize_with = "StyleProperty::<Color>::deserialize_color_or_none"
    )]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fill_color: Option<StyleProperty<Color>>,
    // TODO a lot
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LinePaint {
    #[serde(rename = "line-color")]
    #[serde(
        default,
        deserialize_with = "StyleProperty::<Color>::deserialize_color_or_none"
    )]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_color: Option<StyleProperty<Color>>,

    #[serde(rename = "line-width")]
    #[serde(
        default,
        deserialize_with = "StyleProperty::<f32>::deserialize_f32_or_none"
    )]
    pub line_width: Option<StyleProperty<f32>>,
    // TODO a lot
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RasterResampling {
    #[serde(rename = "linear")]
    Linear,
    #[serde(rename = "nearest")]
    Nearest,
}

/// Raster tile layer description
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RasterPaint {
    #[serde(rename = "raster-brightness-max")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raster_brightness_max: Option<f32>,
    #[serde(rename = "raster-brightness-min")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raster_brightness_min: Option<f32>,
    #[serde(rename = "raster-contrast")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raster_contrast: Option<f32>,
    #[serde(rename = "raster-fade-duration")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raster_fade_duration: Option<u32>,
    #[serde(rename = "raster-hue-rotate")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raster_hue_rotate: Option<f32>,
    #[serde(rename = "raster-opacity")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raster_opacity: Option<f32>,
    #[serde(rename = "raster-resampling")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raster_resampling: Option<RasterResampling>,
    #[serde(rename = "raster-saturation")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raster_saturation: Option<f32>,
}

impl Default for RasterPaint {
    fn default() -> Self {
        RasterPaint {
            raster_brightness_max: Some(1.0),
            raster_brightness_min: Some(0.0),
            raster_contrast: Some(0.0),
            raster_fade_duration: Some(0),
            raster_hue_rotate: Some(0.0),
            raster_opacity: Some(1.0),
            raster_resampling: Some(RasterResampling::Linear),
            raster_saturation: Some(0.0),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SymbolPaint {
    #[serde(rename = "text-field")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_field: Option<String>,
    // TODO a lot
}

/// Extract the property name from a text-field template string like "{NAME}" → "NAME".
/// If no braces, returns the string as-is.
fn extract_text_field_property(template: &str) -> String {
    let trimmed = template.trim();
    if trimmed.starts_with('{') && trimmed.ends_with('}') {
        trimmed[1..trimmed.len() - 1].to_string()
    } else {
        trimmed.to_string()
    }
}

/// Extract a text-field property name from a layout JSON value.
/// Handles both:
///   - `"text-field": "{NAME}"` (constant string)
///   - `"text-field": {"stops": [[2, "{ABBREV}"], [4, "{NAME}"]]}` (zoom-dependent)
fn parse_text_field_from_layout(layout: &serde_json::Value) -> Option<String> {
    let tf = layout.get("text-field")?;
    if let Some(s) = tf.as_str() {
        return Some(extract_text_field_property(s));
    }
    // Zoom-dependent: use the last stop's value (highest zoom = most detailed)
    if let Some(stops) = tf.get("stops").and_then(|v| v.as_array()) {
        if let Some(last_stop) = stops.last() {
            if let Some(s) = last_stop.get(1).and_then(|v| v.as_str()) {
                return Some(extract_text_field_property(s));
            }
        }
    }
    None
}

/// The different types of paints.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "paint")]
pub enum LayerPaint {
    #[serde(rename = "background")]
    Background(BackgroundPaint),
    #[serde(rename = "line")]
    Line(LinePaint),
    #[serde(rename = "fill")]
    Fill(FillPaint),
    #[serde(rename = "raster")]
    Raster(RasterPaint),
    #[serde(rename = "symbol")]
    Symbol(SymbolPaint),
}

impl LayerPaint {
    pub fn get_color(&self) -> Option<Alpha<EncodedSrgb<f32>>> {
        match self {
            LayerPaint::Background(paint) => paint.background_color.as_ref().and_then(|property| {
                if let StyleProperty::Constant(color) = property {
                    Some(color.clone().into())
                } else {
                    None // Expression types have no single static color
                }
            }),
            LayerPaint::Line(paint) => paint.line_color.as_ref().and_then(|property| {
                if let StyleProperty::Constant(color) = property {
                    Some(color.clone().into())
                } else {
                    None
                }
            }),
            LayerPaint::Fill(paint) => paint.fill_color.as_ref().and_then(|property| {
                if let StyleProperty::Constant(color) = property {
                    Some(color.clone().into())
                } else {
                    None
                }
            }),
            LayerPaint::Raster(_) => None,
            LayerPaint::Symbol(_) => None,
        }
    }
}

/// Stores all the styles for a specific layer.
#[derive(Serialize, Debug, Clone)]
pub struct StyleLayer {
    #[serde(skip)]
    pub index: u32, // FIXME: How is this initialized?
    pub id: String, // todo make sure that ids are unique. Styles with non-unique layer ids must not exist
    #[serde(rename = "type")]
    pub type_: String,
    // TODO filter
    // TODO layout
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maxzoom: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minzoom: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    pub paint: Option<LayerPaint>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_layer: Option<String>,
}

#[derive(Deserialize)]
struct StyleLayerDef {
    id: String,
    #[serde(rename = "type")]
    type_: String,
    maxzoom: Option<u8>,
    minzoom: Option<u8>,
    metadata: Option<HashMap<String, String>>,
    source: Option<String>,
    #[serde(rename = "source-layer")]
    source_layer: Option<String>,
    paint: Option<serde_json::Value>,
    layout: Option<serde_json::Value>,
}

impl<'de> serde::Deserialize<'de> for StyleLayer {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let def = StyleLayerDef::deserialize(deserializer)?;

        let paint = if let Some(p) = def.paint {
            match def.type_.as_str() {
                "background" => serde_json::from_value(p.clone())
                    .map(LayerPaint::Background)
                    .ok(),
                "line" => serde_json::from_value(p.clone())
                    .map(LayerPaint::Line)
                    .map_err(|e| log::error!("line paint failed {}: {:?}", def.id, e))
                    .ok(),
                "fill" => serde_json::from_value(p.clone())
                    .map(LayerPaint::Fill)
                    .map_err(|e| log::error!("fill paint failed {}: {:?}", def.id, e))
                    .ok(),
                "raster" => serde_json::from_value(p.clone())
                    .map(LayerPaint::Raster)
                    .ok(),
                "symbol" => {
                    let mut paint: Option<SymbolPaint> =
                        serde_json::from_value(p.clone())
                            .map_err(|e| log::error!("symbol paint failed {}: {:?}", def.id, e))
                            .ok();
                    // text-field lives in layout, not paint — merge it in
                    if let (Some(sp), Some(layout)) = (paint.as_mut(), def.layout.as_ref()) {
                        if sp.text_field.is_none() {
                            sp.text_field = parse_text_field_from_layout(layout);
                        }
                    }
                    paint.map(LayerPaint::Symbol)
                }
                _ => None,
            }
        } else if def.type_ == "symbol" {
            // Symbol layers may have no paint but still have layout with text-field
            let text_field = def.layout.as_ref().and_then(parse_text_field_from_layout);
            Some(LayerPaint::Symbol(SymbolPaint { text_field }))
        } else {
            None
        };

        Ok(StyleLayer {
            index: 0,
            id: def.id,
            type_: def.type_,
            maxzoom: def.maxzoom,
            minzoom: def.minzoom,
            metadata: def.metadata,
            paint,
            source: def.source,
            source_layer: def.source_layer,
        })
    }
}

impl Eq for StyleLayer {}
impl PartialEq for StyleLayer {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id)
    }
}

impl Hash for StyleLayer {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}

impl Default for StyleLayer {
    fn default() -> Self {
        Self {
            index: 0,
            id: "id".to_string(),
            type_: "background".to_string(),
            maxzoom: None,
            minzoom: None,
            metadata: None,
            paint: None,
            source: None,
            source_layer: Some("does not exist".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluate_match_missing_property_returns_fallback() {
        let json = r#"
        [
            "match",
            ["get", "ADM0_A3"],
            ["ARM", "ATG"],
            "rgba(1, 2, 3, 1)",
            "rgba(9, 9, 9, 1)"
        ]
        "#;
        let expr: serde_json::Value = serde_json::from_str(json).unwrap();
        let prop: StyleProperty<csscolorparser::Color> = StyleProperty::Expression(expr);

        // Feature that does NOT have the property → should return the JSON fallback color
        let empty_props = HashMap::new();
        let color = prop.evaluate(&empty_props).unwrap();
        assert_eq!(color.to_rgba8(), [9, 9, 9, 255]);
    }

    #[test]
    fn test_evaluate_match() {
        let json = r#"
        [
            "match",
            ["get", "ADM0_A3"],
            ["ARM", "ATG"],
            "rgba(1, 2, 3, 1)",
            "rgba(0, 0, 0, 1)"
        ]
        "#;
        let expr: serde_json::Value = serde_json::from_str(json).unwrap();
        let prop: StyleProperty<csscolorparser::Color> = StyleProperty::Expression(expr);

        let mut feature_properties = HashMap::new();
        feature_properties.insert("ADM0_A3".to_string(), "ARM".to_string());

        let color = prop.evaluate(&feature_properties).unwrap();
        assert_eq!(color.to_rgba8(), [1, 2, 3, 255]);
    }

    #[test]
    fn test_symbol_text_field_from_layout() {
        let json = r#"{
            "id": "countries-label",
            "type": "symbol",
            "paint": {
                "text-color": "rgba(8, 37, 77, 1)"
            },
            "layout": {
                "text-field": "{NAME}",
                "text-font": ["Open Sans Semibold"]
            },
            "source": "maplibre",
            "source-layer": "centroids"
        }"#;
        let layer: StyleLayer = serde_json::from_str(json).unwrap();
        assert_eq!(layer.type_, "symbol");
        match &layer.paint {
            Some(LayerPaint::Symbol(sp)) => {
                assert_eq!(sp.text_field.as_deref(), Some("NAME"));
            }
            other => panic!("expected Symbol paint, got {:?}", other),
        }
    }

    #[test]
    fn test_symbol_text_field_zoom_dependent() {
        let json = r#"{
            "id": "test-label",
            "type": "symbol",
            "paint": {},
            "layout": {
                "text-field": {"stops": [[2, "{ABBREV}"], [4, "{NAME}"]]}
            },
            "source": "maplibre",
            "source-layer": "centroids"
        }"#;
        let layer: StyleLayer = serde_json::from_str(json).unwrap();
        match &layer.paint {
            Some(LayerPaint::Symbol(sp)) => {
                // Should pick the last stop (highest zoom) → NAME
                assert_eq!(sp.text_field.as_deref(), Some("NAME"));
            }
            other => panic!("expected Symbol paint, got {:?}", other),
        }
    }

    #[test]
    fn test_demotiles_symbol_layers_have_text_field() {
        let style: crate::style::Style = Default::default();
        for layer in &style.layers {
            if layer.type_ == "symbol" {
                match &layer.paint {
                    Some(LayerPaint::Symbol(sp)) => {
                        assert!(
                            sp.text_field.is_some(),
                            "symbol layer '{}' should have text_field parsed from layout",
                            layer.id
                        );
                    }
                    _ => panic!("symbol layer '{}' has no Symbol paint", layer.id),
                }
            }
        }
    }
}
