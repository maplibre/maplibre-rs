use crate::coords::WorldTileCoords;
use crate::error::Error;
use style_spec::source::TileAddressingScheme;

pub struct HttpSourceClient {
    #[cfg(not(target_arch = "wasm32"))]
    inner_client: crate::platform::http_client::ReqwestHttpClient,
    #[cfg(target_arch = "wasm32")]
    inner_client: crate::platform::http_client::WHATWGFetchHttpClient,
}

pub enum SourceClient {
    Http(HttpSourceClient),
    Mbtiles {
        // TODO
    },
}

impl SourceClient {
    pub async fn fetch(&self, coords: &WorldTileCoords) -> Result<Vec<u8>, Error> {
        match self {
            SourceClient::Http(client) => client.fetch(coords).await,
            SourceClient::Mbtiles { .. } => unimplemented!(),
        }
    }
}

impl HttpSourceClient {
    pub fn new() -> Self {
        Self {
            #[cfg(not(target_arch = "wasm32"))]
            inner_client: crate::platform::http_client::ReqwestHttpClient::new(Some(
                "./maplibre-cache".to_string(), // TODO make path  dynamic
            )),
            #[cfg(target_arch = "wasm32")]
            inner_client: crate::platform::http_client::WHATWGFetchHttpClient::new(),
        }
    }

    pub async fn fetch(&self, coords: &WorldTileCoords) -> Result<Vec<u8>, Error> {
        let tile_coords = coords.into_tile(TileAddressingScheme::TMS).unwrap();
        self.inner_client
            .fetch(
                format!(
                    "https://maps.tuerantuer.org/europe_germany/{z}/{x}/{y}.pbf",
                    x = tile_coords.x,
                    y = tile_coords.y,
                    z = tile_coords.z
                )
                .as_str(),
            )
            .await
    }
}
