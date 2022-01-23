//! Module which is used target platform is not web related.

use async_trait::async_trait;
use reqwest::{Client, StatusCode};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_middleware_cache::managers::CACacheManager;
use reqwest_middleware_cache::{Cache, CacheMode};

use crate::error::Error;
use crate::io::{HttpFetcher, HttpFetcherConfig};

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::Network(err.to_string())
    }
}

impl From<reqwest_middleware::Error> for Error {
    fn from(err: reqwest_middleware::Error) -> Self {
        Error::Network(err.to_string())
    }
}

pub struct PlatformHttpFetcher {
    client: ClientWithMiddleware,
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl HttpFetcher for PlatformHttpFetcher {
    fn new(config: HttpFetcherConfig) -> Self {
        let mut builder = ClientBuilder::new(Client::new());

        // FIXME: Cache only works on desktop so far
        if cfg!(not(any(target_os = "android", target_arch = "aarch64"))) {
            builder = builder.with(Cache {
                mode: CacheMode::Default,
                cache_manager: CACacheManager {
                    path: config.cache_path,
                },
            });
        }

        Self {
            client: builder.build(),
        }
    }

    async fn fetch(&self, url: &str) -> Result<Vec<u8>, Error> {
        let response = self.client.get(url).send().await?;
        if response.status() != StatusCode::OK {
            return Err(Error::Network("response code not 200".to_string()));
        }
        let body = response.bytes().await?;
        Ok(Vec::from(body.as_ref()))
    }
}
