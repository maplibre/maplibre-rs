use std::ffi::CString;

use maplibre::style::Style;

use super::{assign_style_layer_indices, maplibre_apple_main_with_mbtiles};

#[test]
fn local_source_entry_point_rejects_null_configuration() {
    let accepted = unsafe { maplibre_apple_main_with_mbtiles(std::ptr::null(), std::ptr::null()) };

    assert!(!accepted);
}

#[test]
fn local_source_entry_point_rejects_invalid_style() {
    let archive = CString::new("/not/opened.mbtiles").unwrap();
    let style = CString::new("not-json").unwrap();

    let accepted = unsafe { maplibre_apple_main_with_mbtiles(archive.as_ptr(), style.as_ptr()) };

    assert!(!accepted);
}

#[test]
fn local_source_entry_point_rejects_missing_archive() {
    let archive = CString::new("/missing/maplibre-rs.mbtiles").unwrap();
    let style = CString::new(r#"{"version":8,"layers":[]}"#).unwrap();

    let accepted = unsafe { maplibre_apple_main_with_mbtiles(archive.as_ptr(), style.as_ptr()) };

    assert!(!accepted);
}

#[test]
fn json_style_layers_keep_their_paint_order() {
    let mut style: Style = serde_json::from_str(
        r##"{
            "version": 8,
            "layers": [
                {
                    "id": "background",
                    "type": "background",
                    "paint": {"background-color": "#081827"}
                },
                {
                    "id": "airways",
                    "type": "line",
                    "source-layer": "airways",
                    "paint": {"line-color": "#F5C542"}
                }
            ]
        }"##,
    )
    .unwrap();

    assign_style_layer_indices(&mut style).unwrap();

    assert_eq!(style.layers[0].index, 0);
    assert_eq!(style.layers[1].index, 1);
}
