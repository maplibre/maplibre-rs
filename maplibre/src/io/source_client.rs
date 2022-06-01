//! HTTP client.

use crate::coords::WorldTileCoords;
use crate::error::Error;
use crate::style::source::TileAddressingScheme;
use async_trait::async_trait;

/// A closure that returns a HTTP client.
pub type HttpClientFactory<HC> = dyn Fn() -> HC;

// On the web platform futures are not thread-safe (i.e. not Send). This means we need to tell
// async_trait that these bounds should not be placed on the async trait:
// https://github.com/dtolnay/async-trait/blob/b70720c4c1cc0d810b7446efda44f81310ee7bf2/README.md#non-threadsafe-futures
#[async_trait(?Send)]
pub trait HttpClient: Clone + Sync + Send + 'static {
    async fn fetch(&self, url: &str) -> Result<Vec<u8>, Error>;
}

/// Gives access to the HTTP client which can be of multiple types,
/// see [crates::io::source_client::SourceClient]
#[derive(Clone)]
pub struct HttpSourceClient<HC>
where
    HC: HttpClient,
{
    inner_client: HC,
}

/// Defines the different types of HTTP clients such as basic HTTP and Mbtiles.
/// More types might be coming such as S3 and other cloud http clients.
#[derive(Clone)]
pub enum SourceClient<HC>
where
    HC: HttpClient,
{
    Http(HttpSourceClient<HC>),
    Mbtiles {
        // TODO
    },
}

impl<HC> SourceClient<HC>
where
    HC: HttpClient,
{
    pub async fn fetch(&self, coords: &WorldTileCoords) -> Result<Vec<u8>, Error> {
        match self {
            SourceClient::Http(client) => client.fetch(coords).await,
            SourceClient::Mbtiles { .. } => unimplemented!(),
        }
    }
}

impl<HC> HttpSourceClient<HC>
where
    HC: HttpClient,
{
    pub fn new(http_client: HC) -> Self {
        Self {
            inner_client: http_client,
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
