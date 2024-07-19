use async_trait::async_trait;
use http_cache_reqwest::{CACacheManager, Cache, CacheMode, HttpCache, HttpCacheOptions};
use reqwest::{Client, Request, Response, StatusCode};
use reqwest_middleware::{ClientWithMiddleware, Next};
use std::path::PathBuf;

use crate::io::source_client::{HttpClient, SourceFetchError};

#[derive(Clone)]
pub struct ReqwestHttpClient {
    client: ClientWithMiddleware,
}

impl From<reqwest::Error> for SourceFetchError {
    fn from(err: reqwest::Error) -> Self {
        SourceFetchError(Box::new(err))
    }
}

impl From<reqwest_middleware::Error> for SourceFetchError {
    fn from(err: reqwest_middleware::Error) -> Self {
        SourceFetchError(Box::new(err))
    }
}

struct LoggingMiddleware;

#[async_trait::async_trait]
impl reqwest_middleware::Middleware for LoggingMiddleware {
    async fn handle(
        &self,
        req: Request,
        extensions: &mut http::Extensions,
        next: Next<'_>,
    ) -> reqwest_middleware::Result<Response> {
        println!("Request started {:?}", req);
        let res = next.run(req, extensions).await;
        println!("Result: {:?}", res);
        res
    }
}

impl ReqwestHttpClient {
    /// cache_path: Under which path should we cache requests.
    pub fn new<P>(cache_path: Option<P>) -> Self
    where
        P: Into<PathBuf>,
    {
        let mut builder = reqwest_middleware::ClientBuilder::new(Client::new());

        if let Some(cache_path) = cache_path {
            builder = builder
                .with(Cache(HttpCache {
                    mode: CacheMode::Default,
                    manager: CACacheManager {
                        path: cache_path.into(),
                    },
                    options: HttpCacheOptions::default(),
                }))
                .with(LoggingMiddleware);
        }
        let client = builder.build();

        Self {
            client,
        }
    }
}

#[cfg_attr(not(feature = "thread-safe-futures"), async_trait(?Send))]
#[cfg_attr(feature = "thread-safe-futures", async_trait)]
impl HttpClient for ReqwestHttpClient {
    async fn fetch(&self, url: &str) -> Result<Vec<u8>, SourceFetchError> {
        let response = self.client.get(url).send().await?;
        match response.error_for_status() {
            Ok(response) => {
                if response.status() == StatusCode::NOT_MODIFIED {
                    log::info!("Using data from cache");
                }

                let body = response.bytes().await?;

                Ok(Vec::from(body.as_ref()))
            }
            Err(e) => Err(SourceFetchError(Box::new(e))),
        }
    }
}
