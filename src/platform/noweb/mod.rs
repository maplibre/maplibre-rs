//! Module which is used target platform is not web related.

pub use std::time::Instant;

pub mod scheduler {
    use reqwest::{Client, StatusCode};
    use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
    use reqwest_middleware_cache::managers::CACacheManager;
    use reqwest_middleware_cache::{Cache, CacheMode};

    use crate::coords::TileCoords;
    use crate::error::Error;
    use crate::io::scheduler::IOScheduler;
    use crate::io::TileRequestID;

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

    pub struct TokioScheduleMethod {
        client: ClientWithMiddleware,
    }

    impl TokioScheduleMethod {
        /// cache_path: Under which path should we cache requests.
        pub fn new(cache_path: Option<String>) -> Self {
            let mut builder = ClientBuilder::new(Client::new());

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

        async fn fetch(client: &ClientWithMiddleware, url: &str) -> Result<Vec<u8>, Error> {
            let response = client.get(url).send().await?;
            if response.status() != StatusCode::OK {
                return Err(Error::Network("response code not 200".to_string()));
            }
            let body = response.bytes().await?;
            Ok(Vec::from(body.as_ref()))
        }

        pub fn schedule_tile_request(
            &self,
            scheduler: &IOScheduler,
            request_id: TileRequestID,
            coords: TileCoords,
        ) {
            let state = scheduler.new_tessellator_state();
            let client = self.client.clone();

            tokio::task::spawn(async move {
                if let Ok(data) = Self::fetch(
                    &client,
                    format!(
                        "https://maps.tuerantuer.org/europe_germany/{z}/{x}/{y}.pbf",
                        x = coords.x,
                        y = coords.y,
                        z = coords.z
                    )
                    .as_str(),
                )
                .await
                {
                    state
                        .tessellate_layers(request_id, data.into_boxed_slice())
                        .unwrap();
                } else {
                    // TODO Error
                }
            });
        }
    }
}
