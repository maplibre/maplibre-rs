use serde::{Deserialize, Serialize};

pub type TileUrl = String;

pub type TileJSONUrl = String;

#[derive(Serialize, Deserialize, Debug)]
pub enum TileAdressingScheme {
    #[serde(rename(serialize = "xyz"))]
    XYZ,
    #[serde(rename(serialize = "tms"))]
    TMS,
}

impl Default for TileAdressingScheme {
    fn default() -> Self {
        TileAdressingScheme::XYZ
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VectorSource {
    /// String which contains attribution information for the used tiles
    attribution: Option<String>,
    /// The bounds in which tiles are available
    bounds: Option<(f64, f64, f64, f64)>,
    /// Max zoom level at which tiles are available
    maxzoom: Option<u8>,
    /// Min zoom level at which tiles are available
    minzoom: Option<u8>,
    // TODO: promoteId
    #[serde(default)]
    scheme: TileAdressingScheme,
    /// Array of URLs which can contain place holders like {x}, {y}, {z}.
    tiles: Option<TileUrl>,
    // url: Option<TileJSONUrl>,
    // TODO volatile
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Source {
    #[serde(rename(serialize = "vector"))]
    Vector(VectorSource),
    #[serde(rename(serialize = "raster"))]
    Raster(VectorSource),
}
