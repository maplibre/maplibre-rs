use serde::{Deserialize, Serialize};

/// TileJSON struct that represents map metadata
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct TileJSON {
    /// A semver.org style version number. Describes the version of
    /// the TileJSON spec that is implemented by this JSON object.
    pub tilejson: String,

    /// The tileset id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// A name describing the tileset. The name can
    /// contain any legal character. Implementations SHOULD NOT interpret the
    /// name as HTML.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// A text description of the tileset. The
    /// description can contain any legal character. Implementations SHOULD NOT
    /// interpret the description as HTML.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// A semver.org style version number. When
    /// changes across tiles are introduced, the minor version MUST change.
    /// This may lead to cut off labels. Therefore, implementors can decide to
    /// clean their cache when the minor version changes. Changes to the patch
    /// level MUST only have changes to tiles that are contained within one tile.
    /// When tiles change significantly, the major version MUST be increased.
    /// Implementations MUST NOT use tiles with different major versions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// Contains an attribution to be displayed
    /// when the map is shown to a user. Implementations MAY decide to treat this
    /// as HTML or literal text. For security reasons, make absolutely sure that
    /// this field can't be abused as a vector for XSS or beacon tracking.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attribution: Option<String>,

    /// Contains a mustache template to be used to
    /// format data from grids for interaction.
    /// See https://github.com/mapbox/utfgrid-spec/tree/master/1.2
    /// for the interactivity specification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,

    /// Contains a legend to be displayed with the map.
    /// Implementations MAY decide to treat this as HTML or literal text.
    /// For security reasons, make absolutely sure that this field can't be
    /// abused as a vector for XSS or beacon tracking.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub legend: Option<String>,

    /// Either "xyz" or "tms". Influences the y
    /// direction of the tile coordinates.
    /// The global-mercator (aka Spherical Mercator) profile is assumed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scheme: Option<String>,

    /// An array of tile endpoints. {z}, {x} and {y}, if present,
    /// are replaced with the corresponding integers. If multiple endpoints are specified, clients
    /// may use any combination of endpoints. All endpoints MUST return the same
    /// content for the same URL. The array MUST contain at least one endpoint.
    pub tiles: Vec<String>,

    /// An array of interactivity endpoints. {z}, {x}
    /// and {y}, if present, are replaced with the corresponding integers. If multiple
    /// endpoints are specified, clients may use any combination of endpoints.
    /// All endpoints MUST return the same content for the same URL.
    /// If the array doesn't contain any entries, interactivity is not supported
    /// for this tileset.
    /// See https://github.com/mapbox/utfgrid-spec/tree/master/1.2
    /// for the interactivity specification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grids: Option<Vec<String>>,

    /// An array of data files in GeoJSON format.
    /// {z}, {x} and {y}, if present,
    /// are replaced with the corresponding integers. If multiple
    /// endpoints are specified, clients may use any combination of endpoints.
    /// All endpoints MUST return the same content for the same URL.
    /// If the array doesn't contain any entries, then no data is present in
    /// the map.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Vec<String>>,

    /// An integer specifying the minimum zoom level.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minzoom: Option<u8>,

    /// An integer specifying the maximum zoom level. MUST be >= minzoom.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maxzoom: Option<u8>,

    /// The maximum extent of available map tiles. Bounds MUST define an area
    /// covered by all zoom levels. The bounds are represented in WGS:84
    /// latitude and longitude values, in the order left, bottom, right, top.
    /// Values may be integers or floating point numbers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bounds: Option<Vec<f32>>,

    /// The first value is the longitude, the second is latitude (both in
    /// WGS:84 values), the third value is the zoom level as an integer.
    /// Longitude and latitude MUST be within the specified bounds.
    /// The zoom level MUST be between minzoom and maxzoom.
    /// Implementations can use this value to set the default location. If the
    /// value is null, implementations may use their own algorithm for
    /// determining a default location.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub center: Option<Vec<i32>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reading() {
        // language=JSON
        let tilejson_str = r#"
        {
            "tilejson": "2.2.0",
            "attribution": "",
            "name": "compositing",
            "scheme": "tms",
            "tiles": [
                "http://localhost:8888/admin/1.0.0/world-light,broadband/{z}/{x}/{y}.png"
            ]
        }
        "#;

        let tilejson: TileJSON = serde_json::from_str(tilejson_str).unwrap();

        assert_eq!(
            tilejson,
            TileJSON {
                tilejson: "2.2.0".to_string(),
                id: None,
                name: Some(String::from("compositing")),
                description: None,
                version: None,
                attribution: Some("".to_string()),
                template: None,
                legend: None,
                scheme: Some(String::from("tms")),
                tiles: vec![String::from(
                    "http://localhost:8888/admin/1.0.0/world-light,broadband/{z}/{x}/{y}.png"
                )],
                grids: None,
                data: None,
                minzoom: None,
                maxzoom: None,
                bounds: None,
                center: None,
            }
        )
    }
}
