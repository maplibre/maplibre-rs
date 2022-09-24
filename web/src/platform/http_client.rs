use async_trait::async_trait;
use js_sys::{ArrayBuffer, Uint8Array};
use maplibre::{error::Error, io::source_client::HttpClient};
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, Response, WorkerGlobalScope};

use crate::error::WebError;

pub struct WHATWGFetchHttpClient {}

impl WHATWGFetchHttpClient {
    pub fn new() -> Self {
        Self {}
    }

    async fn fetch_array_buffer(url: &str) -> Result<JsValue, JsValue> {
        let mut opts = RequestInit::new();
        opts.method("GET");

        let request = Request::new_with_str_and_init(url, &opts)?;

        // Get the global scope
        let global = js_sys::global();
        assert!(global.is_instance_of::<WorkerGlobalScope>());
        let scope = global.dyn_into::<WorkerGlobalScope>().unwrap(); // TODO: remove unwrap

        // Call fetch on global scope
        let maybe_response = JsFuture::from(scope.fetch_with_request(&request)).await?;
        assert!(maybe_response.is_instance_of::<Response>());
        let response: Response = maybe_response.dyn_into().unwrap(); // TODO: remove unwrap

        // Get ArrayBuffer
        let maybe_array_buffer = JsFuture::from(response.array_buffer()?).await?;
        Ok(maybe_array_buffer)
    }

    async fn fetch_bytes(&self, url: &str) -> Result<Vec<u8>, WebError> {
        let maybe_array_buffer = Self::fetch_array_buffer(url).await?;

        assert!(maybe_array_buffer.is_instance_of::<ArrayBuffer>());
        let array_buffer: ArrayBuffer = maybe_array_buffer.dyn_into().unwrap(); // TODO: remove unwrap

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
    async fn fetch(&self, url: &str) -> Result<Vec<u8>, Error> {
        self.fetch_bytes(url)
            .await
            .map_err(|WebError(msg)| Error::Network(msg))
    }
}
