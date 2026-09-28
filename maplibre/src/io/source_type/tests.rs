#![allow(clippy::expect_used, clippy::panic)]

use super::{RasterSource, SourceType, TessellateSource};
use crate::{
    coords::{WorldTileCoords, ZoomLevel},
    style::source::TileAddressingScheme,
};

fn coords(x: i32, y: i32, z: u8) -> WorldTileCoords {
    WorldTileCoords {
        x,
        y,
        z: ZoomLevel::new(z),
    }
}

#[test]
fn base_url_source_keeps_legacy_layout() {
    let source = TessellateSource::new("https://tiles.example/base", "pbf");

    assert_eq!(
        source.format(&coords(3, 5, 4)).expect("in range"),
        "https://tiles.example/base/4/3/5.pbf"
    );
}

#[test]
fn template_source_substitutes_every_placeholder() {
    let source = TessellateSource::from_template(
        "https://tiles.example/{z}/{x}/{y}.pbf?v={z}",
        TileAddressingScheme::XYZ,
    );

    assert_eq!(
        source.format(&coords(1, 2, 3)).expect("in range"),
        "https://tiles.example/3/1/2.pbf?v=3"
    );
}

#[test]
fn tms_scheme_flips_rows() {
    let source = RasterSource::from_template(
        "https://r.example/{z}/{x}/{y}.png",
        TileAddressingScheme::TMS,
    );

    assert_eq!(
        source.format(&coords(0, 0, 1)).expect("in range"),
        "https://r.example/1/0/1.png"
    );
}

#[test]
fn out_of_world_coordinates_have_no_url() {
    let source = SourceType::Tessellate(TessellateSource::from_template(
        "https://tiles.example/{z}/{x}/{y}.pbf",
        TileAddressingScheme::XYZ,
    ));

    assert!(source.format(&coords(-1, 0, 0)).is_none());
    assert!(source.format(&coords(2, 0, 1)).is_none());
}

#[test]
fn keyed_raster_source_keeps_query_string() {
    let source = RasterSource::new("https://r.example/sat", "jpg", "secret");

    assert_eq!(
        source.format(&coords(0, 0, 0)).expect("in range"),
        "https://r.example/sat/0/0/0.jpg?key=secret"
    );
}
