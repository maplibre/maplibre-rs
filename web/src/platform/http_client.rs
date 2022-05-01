use std::thread::Thread;

use js_sys::{ArrayBuffer, Error as JSError, Uint8Array};
use maplibre::io::source_client::HTTPClient;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::Worker;
use web_sys::{Request, RequestInit, RequestMode, Response, WorkerGlobalScope};

use crate::error::WebError;
use async_trait::async_trait;
use maplibre::coords::{TileCoords, WorldTileCoords};
use maplibre::error::Error;
use maplibre::io::scheduler::{ScheduleMethod, Scheduler};
use maplibre::io::tile_cache::TileCache;
use maplibre::io::TileRequestID;

use super::pool::WorkerPool;

pub struct WHATWGFetchHttpClient {}

impl From<JsValue> for WebError {
    fn from(maybe_error: JsValue) -> Self {
        assert!(maybe_error.is_instance_of::<JSError>());
        let error: JSError = maybe_error.dyn_into().unwrap();
        WebError::Network(error.message().as_string().unwrap())
    }
}

impl WHATWGFetchHttpClient {
    pub fn new() -> Self {
        Self {}
    }

    async fn whatwg_fetch(url: &str) -> Result<JsValue, JsValue> {
        let mut opts = RequestInit::new();
        opts.method("GET");

        let request = Request::new_with_str_and_init(&url, &opts)?;

        // Get the global scope
        let global = js_sys::global();
        assert!(global.is_instance_of::<WorkerGlobalScope>());
        let scope = global.dyn_into::<WorkerGlobalScope>().unwrap();

        // Call fetch on global scope
        let maybe_response = JsFuture::from(scope.fetch_with_request(&request)).await?;
        assert!(maybe_response.is_instance_of::<Response>());
        let response: Response = maybe_response.dyn_into().unwrap();

        // Get ArrayBuffer
        let maybe_array_buffer = JsFuture::from(response.array_buffer()?).await?;
        Ok(maybe_array_buffer)
    }

    async fn fetch(&self, url: &str) -> Result<Vec<u8>, WebError> {
        let maybe_array_buffer = Self::whatwg_fetch(url).await?;

        assert!(maybe_array_buffer.is_instance_of::<ArrayBuffer>());
        let array_buffer: ArrayBuffer = maybe_array_buffer.dyn_into().unwrap();

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
impl HTTPClient for WHATWGFetchHttpClient {
    async fn fetch(&self, url: &str) -> Result<Vec<u8>, Error> {
        self.fetch(url)
            .await
            .map_err(|e| Error::Network("web error".to_string()))
    }
}
