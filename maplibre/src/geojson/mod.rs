//! GeoJSON source processing — projects geographic coordinates into tile space and
//! tessellates features using the existing vector rendering pipeline.

use std::{borrow::Cow, f64::consts::PI};

use geozero::{FeatureProcessor, GeomProcessor, GeozeroDatasource, PropertyProcessor};
use thiserror::Error;

use crate::{
    coords::{WorldTileCoords, EXTENT},
    io::apc::{Context, SendError},
    sdf::{tessellation::TextTessellator, tessellation_new::TextTessellatorNew},
    style::layer::{LayerPaint, StyleLayer},
    vector::{
        tessellation::{IndexDataType, ZeroTessellator},
        transferables::{
            LayerMissing, LayerTessellated, SymbolLayerTessellated, TileTessellated,
            VectorTransferables,
        },
    },
};

#[derive(Error, Debug)]
pub enum ProcessGeoJsonError {
    #[error("sending data back through context failed")]
    SendError(SendError),
    #[error("GeoJSON parsing failed: {0}")]
    Parse(Cow<'static, str>),
}

/// Wraps a processor and reprojects geographic (lon/lat) coordinates into
/// tile-local extent coordinates (0–4096) using the Web Mercator projection.
pub struct ProjectingTessellator<T> {
    inner: T,
    tile_x: i32,
    tile_y: i32,
    zoom: u8,
    project: bool,
}

impl<T> ProjectingTessellator<T> {
    pub fn new(coords: WorldTileCoords, project: bool, inner: T) -> Self {
        Self {
            inner,
            tile_x: coords.x,
            tile_y: coords.y,
            zoom: u8::from(coords.z),
            project,
        }
    }

    /// Convert geographic lon/lat to tile-local extent coordinates (0–4096).
    fn project(&self, lon: f64, lat: f64) -> (f64, f64) {
        let lat = lat.clamp(-85.05112877980659, 85.05112877980659);
        let scale = (1u64 << self.zoom) as f64;
        let mx = (180.0 + lon) / 360.0;
        let my = (180.0 - (180.0 / PI * ((PI / 4.0 + lat * PI / 360.0).tan()).ln())) / 360.0;
        let x = (mx * scale - self.tile_x as f64) * EXTENT;
        let y = (my * scale - self.tile_y as f64) * EXTENT;
        (x, y)
    }

    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T: GeomProcessor> GeomProcessor for ProjectingTessellator<T> {
    fn xy(&mut self, x: f64, y: f64, idx: usize) -> geozero::error::Result<()> {
        if x.is_nan() || y.is_nan() {
            println!(
                "ProjectingTessellator received NaN Input! x={}, y={}, idx={}",
                x, y, idx
            );
        }
        let (tx, ty) = self.project(x, y);
        if !tx.is_finite() || !ty.is_finite() {
            println!(
                "ProjectingTessellator output non-finite! lon={}, lat={} -> tx={}, ty={}",
                x, y, tx, ty
            );
        }
        self.inner.xy(tx, ty, idx)
    }

    fn point_begin(&mut self, idx: usize) -> geozero::error::Result<()> {
        self.inner.point_begin(idx)
    }

    fn point_end(&mut self, idx: usize) -> geozero::error::Result<()> {
        self.inner.point_end(idx)
    }

    fn multipoint_begin(&mut self, size: usize, idx: usize) -> geozero::error::Result<()> {
        self.inner.multipoint_begin(size, idx)
    }

    fn multipoint_end(&mut self, idx: usize) -> geozero::error::Result<()> {
        self.inner.multipoint_end(idx)
    }

    fn linestring_begin(
        &mut self,
        tagged: bool,
        size: usize,
        idx: usize,
    ) -> geozero::error::Result<()> {
        self.inner.linestring_begin(tagged, size, idx)
    }

    fn linestring_end(&mut self, tagged: bool, idx: usize) -> geozero::error::Result<()> {
        self.inner.linestring_end(tagged, idx)
    }

    fn multilinestring_begin(&mut self, size: usize, idx: usize) -> geozero::error::Result<()> {
        self.inner.multilinestring_begin(size, idx)
    }

    fn multilinestring_end(&mut self, idx: usize) -> geozero::error::Result<()> {
        self.inner.multilinestring_end(idx)
    }

    fn polygon_begin(
        &mut self,
        tagged: bool,
        size: usize,
        idx: usize,
    ) -> geozero::error::Result<()> {
        self.inner.polygon_begin(tagged, size, idx)
    }

    fn polygon_end(&mut self, tagged: bool, idx: usize) -> geozero::error::Result<()> {
        self.inner.polygon_end(tagged, idx)
    }

    fn multipolygon_begin(&mut self, size: usize, idx: usize) -> geozero::error::Result<()> {
        self.inner.multipolygon_begin(size, idx)
    }

    fn multipolygon_end(&mut self, idx: usize) -> geozero::error::Result<()> {
        self.inner.multipolygon_end(idx)
    }
}

impl<T: PropertyProcessor> PropertyProcessor for ProjectingTessellator<T> {
    fn property(
        &mut self,
        idx: usize,
        name: &str,
        value: &geozero::ColumnValue,
    ) -> geozero::error::Result<bool> {
        self.inner.property(idx, name, value)
    }
}

impl<T: FeatureProcessor> FeatureProcessor for ProjectingTessellator<T> {
    fn dataset_begin(&mut self, name: Option<&str>) -> geozero::error::Result<()> {
        self.inner.dataset_begin(name)
    }
    fn dataset_end(&mut self) -> geozero::error::Result<()> {
        self.inner.dataset_end()
    }
    fn feature_begin(&mut self, idx: u64) -> geozero::error::Result<()> {
        self.inner.feature_begin(idx)
    }
    fn properties_begin(&mut self) -> geozero::error::Result<()> {
        self.inner.properties_begin()
    }
    fn properties_end(&mut self) -> geozero::error::Result<()> {
        self.inner.properties_end()
    }
    fn geometry_begin(&mut self) -> geozero::error::Result<()> {
        self.inner.geometry_begin()
    }
    fn geometry_end(&mut self) -> geozero::error::Result<()> {
        self.inner.geometry_end()
    }
    fn feature_end(&mut self, idx: u64) -> geozero::error::Result<()> {
        self.inner.feature_end(idx)
    }
}

/// Request for processing GeoJSON features for a set of style layers.
pub struct GeoJsonTileRequest {
    pub coords: WorldTileCoords,
    pub layers: Vec<StyleLayer>,
    /// Name of the GeoJSON source (used to match style layers by `source` field).
    pub source_name: String,
    /// If true, applies Web Mercator projection. Tests use false.
    pub project: bool,
}

/// Process inline GeoJSON data and tessellate features for each matching style layer.
///
/// This mirrors [`crate::vector::process_vector_tile`] but works with geographic
/// (lon/lat) coordinates rather than pre-projected MVT tile coordinates.
///
/// For each style layer that references the named GeoJSON source (and has no
/// `source_layer`), ALL features in the GeoJSON are tessellated and sent back
/// via `context`. The tessellated bucket's virtual source-layer name is set to
/// `style_layer.id`, matching the fallback in `upload_system`.
pub fn process_geojson_features<T: VectorTransferables, C: Context>(
    geojson_value: &serde_json::Value,
    request: GeoJsonTileRequest,
    context: &C,
) -> Result<(), ProcessGeoJsonError> {
    let coords = request.coords;
    let json_str = geojson_value.to_string();

    for style_layer in &request.layers {
        let matches_source = style_layer
            .source
            .as_deref()
            .map_or(false, |s| s == request.source_name);
        if !matches_source {
            continue;
        }

        let Some(paint) = &style_layer.paint else {
            log::warn!("GeoJSON style layer {} has no paint", style_layer.id);
            continue;
        };

        match paint {
            LayerPaint::Fill(_) | LayerPaint::Line(_) | LayerPaint::Background(_) => {
                let mut tessellator = ZeroTessellator::<IndexDataType>::default();
                match paint {
                    LayerPaint::Fill(p) => tessellator.style_property = p.fill_color.clone(),
                    LayerPaint::Line(p) => tessellator.style_property = p.line_color.clone(),
                    LayerPaint::Background(p) => {
                        tessellator.style_property = p.background_color.clone()
                    }
                    _ => {}
                }

                let mut projecting =
                    ProjectingTessellator::new(coords, request.project, tessellator);

                let mut geojson_src = geozero::geojson::GeoJson(json_str.as_str());
                if let Err(e) = geojson_src.process(&mut projecting) {
                    log::warn!(
                        "GeoJSON tessellation for layer {} failed: {e:?}",
                        style_layer.id
                    );
                    context
                        .send_back(T::LayerMissing::build_from(coords, style_layer.id.clone()))
                        .map_err(ProcessGeoJsonError::SendError)?;
                    continue;
                }

                let mut inner = projecting.into_inner();
                // For bare GeoJSON geometries (Polygon, LineString, etc. — not a
                // FeatureCollection), geozero never calls `feature_end`, so
                // `feature_indices` stays empty while `buffer.indices` is not.
                // Manually commit the remaining geometry as a single feature.
                if inner.feature_indices.is_empty() && !inner.buffer.indices.is_empty() {
                    let _ = inner.feature_end(0);
                }

                let synthetic_layer = geozero::mvt::tile::Layer {
                    version: 2,
                    name: style_layer.id.clone(),
                    ..Default::default()
                };

                context
                    .send_back(T::LayerTessellated::build_from(
                        coords,
                        inner.buffer.into(),
                        inner.feature_indices,
                        inner.feature_colors,
                        synthetic_layer,
                        style_layer.id.clone(),
                    ))
                    .map_err(ProcessGeoJsonError::SendError)?;
            }
            LayerPaint::Symbol(symbol_paint) => {
                let mut tessellator = TextTessellator::<IndexDataType>::default();
                let text_field = symbol_paint
                    .text_field
                    .clone()
                    .unwrap_or_else(|| "name".to_string());
                let mut tessellator_new = TextTessellatorNew::new(text_field);
                let mut projecting =
                    ProjectingTessellator::new(coords, request.project, tessellator_new);

                let mut geojson_src = geozero::geojson::GeoJson(json_str.as_str());
                if let Err(e) = geojson_src.process(&mut projecting) {
                    log::warn!(
                        "GeoJSON text tessellation for layer {} failed: {e:?}",
                        style_layer.id
                    );
                    context
                        .send_back(T::LayerMissing::build_from(coords, style_layer.id.clone()))
                        .map_err(ProcessGeoJsonError::SendError)?;
                    continue;
                }

                let mut inner = projecting.into_inner();
                inner.finish();

                let synthetic_layer = geozero::mvt::tile::Layer {
                    version: 2,
                    name: style_layer.id.clone(),
                    ..Default::default()
                };

                context
                    .send_back(T::SymbolLayerTessellated::build_from(
                        coords,
                        tessellator.quad_buffer.into(),
                        inner.quad_buffer.into(),
                        inner.features,
                        synthetic_layer,
                        style_layer.id.clone(),
                    ))
                    .map_err(ProcessGeoJsonError::SendError)?;
            }
            _ => {
                log::trace!(
                    "GeoJSON layer {} has unsupported paint type, skipping",
                    style_layer.id
                );
            }
        }
    }

    context
        .send_back(T::TileTessellated::build_from(coords))
        .map_err(ProcessGeoJsonError::SendError)?;

    Ok(())
}
