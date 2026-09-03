//! Resolves the tile sources declared by a style into fetchable URL templates.

use std::collections::BTreeMap;

use crate::{
    coords::WorldTileCoords,
    io::source_type::{RasterSource, SourceType, TessellateSource},
    style::{
        layer::StyleLayer,
        source::{Source, TileAddressingScheme, VectorSource},
        Style,
    },
};

/// Which family of tiles a request fetches.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TileKind {
    /// Vector tiles rendered by fill, line and symbol layers.
    Vector,
    /// Raster image tiles.
    Raster,
}

impl TileKind {
    fn accepts_layer(self, layer: &StyleLayer) -> bool {
        let is_raster = layer.type_ == "raster";
        match self {
            Self::Vector => !is_raster && layer.type_ != "background",
            Self::Raster => is_raster,
        }
    }

    fn matches_source(self, source: &Source) -> Option<&VectorSource> {
        match (self, source) {
            (Self::Vector, Source::Vector(vector)) | (Self::Raster, Source::Raster(vector)) => {
                Some(vector)
            }
            _ => None,
        }
    }

    fn default_source(self) -> SourceType {
        match self {
            Self::Vector => SourceType::Tessellate(TessellateSource::default()),
            Self::Raster => SourceType::Raster(RasterSource::default()),
        }
    }

    fn source_from_template(self, template: &str, scheme: TileAddressingScheme) -> SourceType {
        match self {
            Self::Vector => {
                SourceType::Tessellate(TessellateSource::from_template(template, scheme))
            }
            Self::Raster => SourceType::Raster(RasterSource::from_template(template, scheme)),
        }
    }
}

/// Style layers that share one fetchable tile source.
#[derive(Clone, Debug)]
pub struct SourceLayerGroup {
    /// Style source name, or `None` for layers that name no resolvable source.
    pub source_name: Option<String>,
    /// Fetchable tile source.
    pub source: SourceType,
    /// Layers rendered from tiles of this source.
    pub layers: Vec<StyleLayer>,
}

fn template_of(source: &VectorSource) -> Option<(&str, TileAddressingScheme)> {
    let template = source.tiles.as_ref()?.first()?;
    Some((template.as_str(), source.scheme.unwrap_or_default()))
}

/// Groups the style layers of one tile kind by the source they read from.
///
/// Layers that name no source, or a source without tile URLs, share the crate default source so
/// styles that predate style-driven sources keep rendering. Layers of non-tile sources such as
/// GeoJSON are not part of any group.
pub fn source_layer_groups(style: &Style, kind: TileKind) -> Vec<SourceLayerGroup> {
    let mut groups: BTreeMap<Option<String>, SourceLayerGroup> = BTreeMap::new();
    for layer in style
        .layers
        .iter()
        .filter(|layer| kind.accepts_layer(layer))
    {
        let named = layer
            .source
            .as_ref()
            .map(|name| (name, style.sources.get(name)));
        let (key, source) = match named {
            Some((name, Some(source))) => match kind.matches_source(source) {
                Some(vector) => match template_of(vector) {
                    Some((template, scheme)) => (
                        Some(name.clone()),
                        kind.source_from_template(template, scheme),
                    ),
                    None => (None, kind.default_source()),
                },
                None => continue,
            },
            Some((_, None)) | None => (None, kind.default_source()),
        };
        groups
            .entry(key.clone())
            .or_insert_with(|| SourceLayerGroup {
                source_name: key,
                source,
                layers: Vec::new(),
            })
            .layers
            .push(layer.clone());
    }
    groups.into_values().collect()
}

/// Returns the most restrictive maximum zoom among the tile sources used by the style.
pub fn source_max_zoom(style: &Style, kind: TileKind) -> Option<u8> {
    style
        .layers
        .iter()
        .filter(|layer| kind.accepts_layer(layer))
        .filter_map(|layer| style.sources.get(layer.source.as_ref()?))
        .filter_map(|source| kind.matches_source(source)?.maxzoom)
        .min()
}

/// Replaces coordinates above the source maximum zoom with their ancestor at that zoom.
pub fn clamp_to_max_zoom(coords: WorldTileCoords, max_zoom: Option<u8>) -> WorldTileCoords {
    let Some(max_zoom) = max_zoom else {
        return coords;
    };
    let mut current = coords;
    while u8::from(current.z) > max_zoom {
        match current.get_parent() {
            Some(parent) => current = parent,
            None => break,
        }
    }
    current
}

#[cfg(test)]
mod tests;
