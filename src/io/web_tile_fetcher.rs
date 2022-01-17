use crate::coords::TileCoords;
use crate::error::Error;
use crate::io::{HttpFetcher, HttpFetcherConfig, TileFetcher};
use crate::platform::PlatformHttpFetcher;
use async_trait::async_trait;

pub struct WebTileFetcher {
    http_fetcher: PlatformHttpFetcher,
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl TileFetcher for WebTileFetcher {
    fn new(config: HttpFetcherConfig) -> Self {
        Self {
            http_fetcher: PlatformHttpFetcher::new(config),
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
    use crate::io::{HttpFetcherConfig, TileFetcher};

    #[tokio::test]
    async fn test_tiles_available() {
        let fetcher = WebTileFetcher::new(HttpFetcherConfig::default());
        assert!(fetcher.fetch_tile(&(0, 0, 0).into()).await.is_ok()); // World overview
    }
}
