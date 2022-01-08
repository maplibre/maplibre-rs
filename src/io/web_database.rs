use crate::coords::TileCoords;
use crate::error::Error;
use crate::platform::download;

pub async fn get_tile(coords: &TileCoords) -> Result<Vec<u8>, Error> {
    download(format!(
        "https://maps.tuerantuer.org/europe_germany/{z}/{x}/{y}.pbf",
        x = coords.x,
        y = coords.y,
        z = coords.z
    ))
    .await
}
