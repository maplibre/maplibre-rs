#![allow(clippy::expect_used, clippy::panic)]

use super::layer_translate_tile_units;
use crate::{
    coords::ZoomLevel,
    style::layer::{FillPaint, LayerPaint, TranslateAnchor},
};

#[test]
fn viewport_translation_rotates_with_map_bearing() {
    let paint = LayerPaint::Fill(FillPaint {
        fill_translate: Some([10.0, 50.0]),
        fill_translate_anchor: TranslateAnchor::Viewport,
        ..FillPaint::default()
    });
    let translation =
        layer_translate_tile_units(Some(&paint), ZoomLevel::new(1), 1.0, 45.0_f32.to_radians());

    assert!((translation[0] + 226.274_17).abs() < 1e-4);
    assert!((translation[1] - 339.411_25).abs() < 1e-4);
}

#[test]
fn map_translation_scales_pixels_for_parent_tile() {
    let paint = LayerPaint::Fill(FillPaint {
        fill_translate: Some([10.0, 50.0]),
        ..FillPaint::default()
    });
    let translation = layer_translate_tile_units(Some(&paint), ZoomLevel::new(0), 2.0, 0.0);

    assert_eq!(translation, [20.0, 100.0]);
}
