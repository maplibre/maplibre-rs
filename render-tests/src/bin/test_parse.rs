use maplibre::style::{layer::StyleLayer, Style};
use serde_json::Value;

fn main() {
    let background_json = r#"{
        "id": "background",
        "type": "background",
        "paint": {
            "background-color": "rgba(0,0,0,1.0)"
        }
    }"#;

    let fill_json = r#"{
        "id": "fill",
        "type": "fill",
        "source": "geojson",
        "paint": {
            "fill-color": "rgba(255,0,0,1.0)"
        }
    }"#;

    let bg_layer: Result<StyleLayer, _> = serde_json::from_str(background_json);
    println!("StyleLayer bg parse: {:?}", bg_layer);

    let fill_layer: Result<StyleLayer, _> = serde_json::from_str(fill_json);
    println!("StyleLayer fill parse: {:?}", fill_layer);
}
