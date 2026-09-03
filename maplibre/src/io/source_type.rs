//! Fetchable tile sources addressed by URL templates.

use thiserror::Error;

use crate::{coords::WorldTileCoords, style::source::TileAddressingScheme};

/// Tile coordinates that cannot be addressed by any tile URL.
#[derive(Debug, Error)]
#[error("tile coordinates {coords} are outside the addressable range")]
pub struct InvalidTileCoords {
    /// Coordinates that failed to convert into an addressing scheme.
    pub coords: WorldTileCoords,
}

fn format_template(
    template: &str,
    scheme: TileAddressingScheme,
    coords: &WorldTileCoords,
) -> Option<String> {
    let tile = coords.into_tile(scheme)?;
    Some(
        template
            .replace("{z}", &u8::from(tile.z).to_string())
            .replace("{x}", &tile.x.to_string())
            .replace("{y}", &tile.y.to_string()),
    )
}

/// Represents a source from which vector tiles are fetched.
#[derive(Clone, Debug)]
pub struct TessellateSource {
    template: String,
    scheme: TileAddressingScheme,
}

impl TessellateSource {
    /// Creates a source addressed as `{url}/{z}/{x}/{y}.{filetype}`.
    pub fn new(url: &str, filetype: &str) -> Self {
        Self::from_template(
            format!("{url}/{{z}}/{{x}}/{{y}}.{filetype}"),
            TileAddressingScheme::XYZ,
        )
    }

    /// Creates a source from a style tile URL template containing `{z}`, `{x}` and `{y}`.
    pub fn from_template(template: impl Into<String>, scheme: TileAddressingScheme) -> Self {
        Self {
            template: template.into(),
            scheme,
        }
    }

    /// Returns the URL template with unexpanded placeholders.
    pub fn template(&self) -> &str {
        &self.template
    }

    /// Returns the tile URL, or `None` when the coordinates are outside the world.
    pub fn format(&self, coords: &WorldTileCoords) -> Option<String> {
        format_template(&self.template, self.scheme, coords)
    }
}

impl Default for TessellateSource {
    fn default() -> Self {
        Self::new("https://maps.tuerantuer.org/europe_germany", "pbf")
    }
}

/// Represents a source from which raster tiles are fetched.
#[derive(Clone, Debug)]
pub struct RasterSource {
    template: String,
    scheme: TileAddressingScheme,
}

impl RasterSource {
    /// Creates a source addressed as `{url}/{z}/{x}/{y}.{filetype}?key={key}`.
    pub fn new(url: &str, filetype: &str, key: &str) -> Self {
        Self::from_template(
            format!("{url}/{{z}}/{{x}}/{{y}}.{filetype}?key={key}"),
            TileAddressingScheme::XYZ,
        )
    }

    /// Creates a source from a style tile URL template containing `{z}`, `{x}` and `{y}`.
    pub fn from_template(template: impl Into<String>, scheme: TileAddressingScheme) -> Self {
        Self {
            template: template.into(),
            scheme,
        }
    }

    /// Returns the URL template with unexpanded placeholders.
    pub fn template(&self) -> &str {
        &self.template
    }

    /// Returns the tile URL, or `None` when the coordinates are outside the world.
    pub fn format(&self, coords: &WorldTileCoords) -> Option<String> {
        format_template(&self.template, self.scheme, coords)
    }
}

impl Default for RasterSource {
    fn default() -> Self {
        Self::new(
            "https://api.maptiler.com/tiles/satellite-v2",
            "jpg",
            "qnePkfbGpMsLCi3KFBs3",
        )
    }
}

/// Represents the tiles' different types of source.
#[derive(Clone, Debug)]
pub enum SourceType {
    Raster(RasterSource),
    Tessellate(TessellateSource),
}

impl SourceType {
    /// Returns the tile URL, or `None` when the coordinates are outside the world.
    pub fn format(&self, coords: &WorldTileCoords) -> Option<String> {
        match self {
            SourceType::Raster(raster_source) => raster_source.format(coords),
            SourceType::Tessellate(tessellate_source) => tessellate_source.format(coords),
        }
    }
}

#[cfg(test)]
mod tests;
