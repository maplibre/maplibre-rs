use std::fs::File;
use std::io::copy;
use std::path::Path;

use vector_tile::grid::*;

pub async fn download_tiles() {
    for (z, x, y) in tile_coordinates_bavaria(&google_mercator(), 6) {
        let target = format!(
            "https://maps.tuerantuer.org/europe_germany/{z}/{x}/{y}.pbf",
            z = z,
            x = x,
            y = y,
        );
        println!("{}", target);
        let client = reqwest::Client::builder()
            .gzip(true)
            .build().unwrap();

        let response = client.get(target).send().await.unwrap();
        if response.status().is_success() {
            let mut dest = {
                let fname =
                    Path::new(".").join(format!("test-data/{z}-{x}-{y}.pbf", z = z, x = x, y = y,));
                File::create(fname).unwrap()
            };
            copy(&mut  response.bytes().await.unwrap().as_ref(), &mut dest).unwrap();
        }
    }
}
#[tokio::main]
async fn main() {
    download_tiles().await;
}
