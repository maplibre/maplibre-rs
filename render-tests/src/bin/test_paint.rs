use maplibre::style::layer::LayerPaint;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct MockStyleLayer {
    #[serde(flatten)]
    pub paint: Option<LayerPaint>,
}

#[derive(Deserialize, Debug)]
pub struct MockStyleLayerWithType {
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(flatten)]
    pub paint: Option<LayerPaint>,
}

fn main() {
    let json = r#"{
      "id": "fill",
      "type": "fill",
      "source": "geojson",
      "paint": {
        "fill-color": "red"
      }
    }"#;
    let layer_without: Result<MockStyleLayer, _> = serde_json::from_str(json);
    println!("Parsed WITHOUT type: {:?}", layer_without);

    let layer_with: Result<MockStyleLayerWithType, _> = serde_json::from_str(json);
    println!("Parsed WITH type: {:?}", layer_with);
}
