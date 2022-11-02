use async_trait::async_trait;
use reqwest::{Client, StatusCode};
use reqwest_middleware::ClientWithMiddleware;
use reqwest_middleware_cache::{managers::CACacheManager, Cache, CacheMode};

use crate::{error::Error, io::source_client::HttpClient};

#[derive(Clone)]
pub struct ReqwestHttpClient {
    client: ClientWithMiddleware,
}
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

impl ReqwestHttpClient {
    /// cache_path: Under which path should we cache requests.
    // TODO: Use Into<Path> instead of String
    pub fn new(cache_path: Option<String>) -> Self {
        let mut builder = reqwest_middleware::ClientBuilder::new(Client::new());

        if let Some(cache_path) = cache_path {
            builder = builder.with(Cache {
                mode: CacheMode::Default,
                cache_manager: CACacheManager { path: cache_path },
            });
        }

        Self {
            client: builder.build(),
        }
    }
}

#[async_trait]
impl HttpClient for ReqwestHttpClient {
    async fn fetch(&self, url: &str) -> Result<Vec<u8>, Error> {
        let response = self.client.get(url).send().await?;
        match response.error_for_status() {
            Ok(response) => {
                if response.status() == StatusCode::NOT_MODIFIED {
                    log::info!("Using data from cache");
                }

                let body = response.bytes().await?;
                Ok(Vec::from(body.as_ref()))
            }
            Err(e) => Err(Error::Network(e.to_string())),
        }
    }
}
