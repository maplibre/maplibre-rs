#![allow(clippy::expect_used, clippy::panic)]

use super::{apply_tile_json, TileJson};
use crate::style::source::VectorSource;

fn source(tiles: Option<Vec<String>>, maxzoom: Option<u8>) -> VectorSource {
    VectorSource {
        attribution: None,
        bounds: None,
        maxzoom,
        minzoom: None,
        scheme: None,
        tiles,
        url: Some("https://tiles.example/tiles.json".to_string()),
    }
}

#[test]
fn parses_the_addressing_subset() {
    let document = serde_json::json!({
        "tilejson": "3.0.0",
        "name": "demo",
        "tiles": ["https://tiles.example/{z}/{x}/{y}.pbf"],
        "minzoom": 0,
        "maxzoom": 6,
        "bounds": [-180.0, -85.0, 180.0, 85.0],
        "vector_layers": [{"id": "countries"}]
    });

    let parsed: TileJson = serde_json::from_value(document).expect("parses");

    assert_eq!(
        parsed,
        TileJson {
            tiles: vec!["https://tiles.example/{z}/{x}/{y}.pbf".to_string()],
            minzoom: Some(0),
            maxzoom: Some(6),
            bounds: Some((-180.0, -85.0, 180.0, 85.0)),
        }
    );
}

#[test]
fn document_fills_only_unspecified_fields() {
    let mut vector = source(None, Some(4));
    apply_tile_json(
        &mut vector,
        TileJson {
            tiles: vec!["https://tiles.example/{z}/{x}/{y}.pbf".to_string()],
            minzoom: Some(1),
            maxzoom: Some(9),
            bounds: None,
        },
    );

    assert_eq!(
        vector.tiles.as_deref(),
        Some(&["https://tiles.example/{z}/{x}/{y}.pbf".to_string()][..])
    );
    assert_eq!(vector.minzoom, Some(1));
    assert_eq!(vector.maxzoom, Some(4));
}

#[test]
fn explicit_tiles_are_never_replaced() {
    let mut vector = source(Some(vec!["https://mine/{z}/{x}/{y}.pbf".to_string()]), None);
    apply_tile_json(
        &mut vector,
        TileJson {
            tiles: vec!["https://theirs/{z}/{x}/{y}.pbf".to_string()],
            minzoom: None,
            maxzoom: None,
            bounds: None,
        },
    );

    assert_eq!(
        vector.tiles.as_deref(),
        Some(&["https://mine/{z}/{x}/{y}.pbf".to_string()][..])
    );
}
