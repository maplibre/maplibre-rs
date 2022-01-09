use crate::coords::TileCoords;
use crate::error::Error;
use crate::io::{HttpFetcher, TileFetcher};
use crate::platform::PlatformHttpFetcher;
use async_trait::async_trait;
use core::panicking::panic;

pub struct WebTileFetcher {
    http_fetcher: PlatformHttpFetcher,
}

#[async_trait(?Send)]
impl TileFetcher for WebTileFetcher {
    fn new() -> Self {
        Self {
            http_fetcher: PlatformHttpFetcher::new(),
        }
    }

    async fn fetch_tile(&self, coords: &TileCoords) -> Result<Vec<u8>, Error> {
        self.http_fetcher
            .fetch(
                format!(
                    "https://maps.tuerantuer.org/europe_germany/{z}/{x}/{y}.pbf",
                    x = coords.x,
                    y = coords.y,
                    z = coords.z
                )
                .as_str(),
            )
            .await
    }

    fn sync_fetch_tile(&self, _coords: &TileCoords) -> Result<Vec<u8>, Error> {
        panic!("Unable to fetch sync from the web!");
    }
}

#[cfg(test)]
mod tests {
    use super::WebTileFetcher;
    use crate::io::TileFetcher;

    #[tokio::test]
    async fn test_tiles_available() {
        let fetcher = WebTileFetcher::new();
        assert!(fetcher.fetch_tile(&(0, 0, 0).into()).await.is_ok()); // World overview
    }
}
