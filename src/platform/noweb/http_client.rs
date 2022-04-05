use crate::error::Error;
use reqwest::{Client, StatusCode};
use reqwest_middleware::ClientWithMiddleware;
use reqwest_middleware_cache::managers::CACacheManager;
use reqwest_middleware_cache::{Cache, CacheMode};

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

    pub async fn fetch(&self, url: &str) -> Result<Vec<u8>, Error> {
        let response = self.client.get(url).send().await?;
        if response.status() != StatusCode::OK {
            return Err(Error::Network("response code not 200".to_string()));
        }
        let body = response.bytes().await?;
        Ok(Vec::from(body.as_ref()))
    }
}
