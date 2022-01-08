use crate::io::cache::Cache;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn create_cache() -> *mut Cache {
    let mut cache = Box::new(Cache::new());
    let ptr = Box::into_raw(cache);
    return ptr;
}

#[wasm_bindgen]
pub async fn run_cache_loop(cache_ptr: *mut Cache) {
    let mut cache: Box<Cache> = unsafe { Box::from_raw(cache_ptr) };

    // Either call forget or the cache loop to keep cache alive
    cache.run_loop().await;
    std::mem::forget(cache);
}
