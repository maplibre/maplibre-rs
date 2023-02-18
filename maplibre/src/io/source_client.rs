//! HTTP client.

use async_trait::async_trait;
use thiserror::Error;

use crate::{coords::WorldTileCoords, io::source_type::SourceType};

/// A closure that returns a HTTP client.
pub type HTTPClientFactory<HC> = dyn Fn() -> HC;

/// On the web platform futures are not thread-safe (i.e. not Send). This means we need to tell
/// async_trait that these bounds should not be placed on the async trait:
/// [https://github.com/dtolnay/async-trait/blob/b70720c4c1cc0d810b7446efda44f81310ee7bf2/README.md#non-threadsafe-futures](https://github.com/dtolnay/async-trait/blob/b70720c4c1cc0d810b7446efda44f81310ee7bf2/README.md#non-threadsafe-futures)
///
/// Users of this library can decide whether futures from the HTTPClient are thread-safe or not via
/// the future "thread-safe-futures". Tokio futures are thread-safe.
#[cfg_attr(not(feature = "thread-safe-futures"), async_trait(?Send))]
#[cfg_attr(feature = "thread-safe-futures", async_trait)]
pub trait HttpClient: Clone + Sync + Send + 'static {
    async fn fetch(&self, url: &str) -> Result<Vec<u8>, SourceFetchError>;
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

#[derive(Error, Debug)]
#[error("failed to fetch from source")]
pub struct SourceFetchError(#[source] pub Box<dyn std::error::Error>);

/// Defines the different types of HTTP clients such as basic HTTP and Mbtiles.
/// More types might be coming such as S3 and other cloud http clients.
#[derive(Clone)]
pub struct SourceClient<HC>
where
    HC: HttpClient,
{
    http: HttpSourceClient<HC>,
}

impl<HC> SourceClient<HC>
where
    HC: HttpClient,
{
    pub fn new(http: HttpSourceClient<HC>) -> Self {
        Self { http }
    }

    pub async fn fetch(
        &self,
        coords: &WorldTileCoords,
        source_type: &SourceType,
    ) -> Result<Vec<u8>, SourceFetchError> {
        self.http.fetch(coords, source_type).await
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

    pub async fn fetch(
        &self,
        coords: &WorldTileCoords,
        source_type: &SourceType,
    ) -> Result<Vec<u8>, SourceFetchError> {
        self.inner_client
            .fetch(source_type.format(coords).as_str())
            .await
    }
}
