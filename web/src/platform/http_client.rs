use async_trait::async_trait;
use js_sys::{ArrayBuffer, Uint8Array};
use maplibre::io::source_client::{HttpClient, SourceFetchError};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, Response, WorkerGlobalScope};

use crate::error::WebError;

#[derive(Default)]
pub struct WHATWGFetchHttpClient;

impl WHATWGFetchHttpClient {
    async fn fetch_array_buffer(url: &str) -> Result<JsValue, WebError> {
        let mut opts = RequestInit::new();
        opts.method("GET");

        let request = Request::new_with_str_and_init(url, &opts)?;

        // Tiles are fetched from workers, but TileJSON documents are resolved on the main thread
        // before the first frame, so both global scopes must be able to fetch.
        let global = js_sys::global();
        let promise = match global.dyn_into::<WorkerGlobalScope>() {
            Ok(scope) => scope.fetch_with_request(&request),
            Err(_) => web_sys::window()
                .ok_or_else(|| {
                    WebError::TypeError("No worker or window scope to fetch from".into())
                })?
                .fetch_with_request(&request),
        };

        let maybe_response = JsFuture::from(promise).await?;
        let response: Response = maybe_response
            .dyn_into()
            .map_err(|_e| WebError::TypeError("Unable to cast to Response".into()))?;

        if !response.ok() {
            return Err(WebError::GenericError(
                format!("failed to fetch {}", response.status()).into(),
            ));
        }

        // Get ArrayBuffer
        let maybe_array_buffer = JsFuture::from(response.array_buffer()?).await?;
        Ok(maybe_array_buffer)
    }

    async fn fetch_bytes(&self, url: &str) -> Result<Vec<u8>, WebError> {
        let maybe_array_buffer = Self::fetch_array_buffer(url).await?;

        let array_buffer: ArrayBuffer = maybe_array_buffer
            .dyn_into()
            .map_err(|_e| WebError::TypeError("Unable to cast to ArrayBuffer".into()))?;

        // Copy data to Vec<u8>
        let buffer: Uint8Array = Uint8Array::new(&array_buffer);
        let mut output: Vec<u8> = vec![0; array_buffer.byte_length() as usize];
        buffer.copy_to(output.as_mut_slice());

        Ok(output)
    }
}

impl Clone for WHATWGFetchHttpClient {
    fn clone(&self) -> Self {
        WHATWGFetchHttpClient {}
    }
}

#[async_trait(?Send)]
impl HttpClient for WHATWGFetchHttpClient {
    async fn fetch(&self, url: &str) -> Result<Vec<u8>, SourceFetchError> {
        self.fetch_bytes(url)
            .await
            .map_err(|e| SourceFetchError(Box::new(e)))
    }
}
