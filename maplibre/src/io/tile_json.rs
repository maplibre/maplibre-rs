//! Resolves TileJSON source URLs into tile URL templates before tiles are requested.

use serde::Deserialize;

use crate::{
    io::source_client::{HttpClient, SourceClient},
    style::{
        source::{Source, VectorSource},
        Style,
    },
};

/// The subset of a TileJSON document needed to address tiles.
#[derive(Debug, Deserialize, PartialEq)]
pub struct TileJson {
    /// Tile URL templates.
    pub tiles: Vec<String>,
    /// Lowest zoom level with tiles.
    #[serde(default)]
    pub minzoom: Option<u8>,
    /// Highest zoom level with tiles.
    #[serde(default)]
    pub maxzoom: Option<u8>,
    /// Bounds in which tiles are available.
    #[serde(default)]
    pub bounds: Option<(f64, f64, f64, f64)>,
}

/// Fills the tile URLs and zoom range a source leaves unspecified from its TileJSON document.
pub fn apply_tile_json(source: &mut VectorSource, tile_json: TileJson) {
    if source.tiles.is_none() {
        source.tiles = Some(tile_json.tiles);
    }
    if source.minzoom.is_none() {
        source.minzoom = tile_json.minzoom;
    }
    if source.maxzoom.is_none() {
        source.maxzoom = tile_json.maxzoom;
    }
    if source.bounds.is_none() {
        source.bounds = tile_json.bounds;
    }
}

fn pending_tile_json(source: &mut Source) -> Option<&mut VectorSource> {
    match source {
        Source::Vector(vector) | Source::Raster(vector) => {
            (vector.tiles.is_none() && vector.url.is_some()).then_some(vector)
        }
        Source::GeoJson(_) => None,
    }
}

/// Fetches the TileJSON of every tile source that declares a `url` but no `tiles`.
///
/// A source whose document cannot be fetched or parsed is left unchanged so its layers fall back
/// to the crate default source instead of failing the whole style.
pub async fn resolve_tile_json_sources<HC: HttpClient>(
    style: &mut Style,
    client: &SourceClient<HC>,
) {
    for (name, source) in &mut style.sources {
        let Some(vector) = pending_tile_json(source) else {
            continue;
        };
        let Some(url) = vector.url.clone() else {
            continue;
        };
        match client.fetch_url(&url).await {
            Ok(bytes) => match serde_json::from_slice::<TileJson>(&bytes) {
                Ok(tile_json) => {
                    tracing::info!(source = %name, %url, "resolved TileJSON source");
                    apply_tile_json(vector, tile_json);
                }
                Err(error) => {
                    tracing::warn!(source = %name, %url, %error, "TileJSON document is invalid");
                }
            },
            Err(error) => {
                tracing::warn!(source = %name, %url, %error, "TileJSON document is unreachable");
            }
        }
    }
}

#[cfg(test)]
mod tests;
