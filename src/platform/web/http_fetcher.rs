use crate::error::Error;
use crate::io::HttpFetcher;
use async_trait::async_trait;
use js_sys::ArrayBuffer;
use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response, WorkerGlobalScope};

impl From<JsValue> for Error {
    fn from(err: JsValue) -> Self {
        Error::Network("JsValue error".to_string())
    }
}

pub struct PlatformHttpFetcher;

#[async_trait(?Send)]
impl HttpFetcher for PlatformHttpFetcher {
    fn new() -> Self {
        Self {}
    }

    async fn fetch(&self, url: &str) -> Result<Vec<u8>, Error> {
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
        assert!(maybe_array_buffer.is_instance_of::<ArrayBuffer>());
        let array_buffer: ArrayBuffer = maybe_array_buffer.dyn_into().unwrap();

        // Copy data to Vec<u8>
        let buffer: Uint8Array = Uint8Array::new(&array_buffer);
        let mut output: Vec<u8> = vec![0; array_buffer.byte_length() as usize];
        buffer.copy_to(output.as_mut_slice());

        Ok(output)
    }
}
