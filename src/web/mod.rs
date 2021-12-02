use wasm_bindgen::prelude::wasm_bindgen;
use log::{Level, warn};

mod console;

#[wasm_bindgen(start)]
pub fn run() {
    console_log::init_with_level(Level::Info).expect("error initializing log");
    console::init_console_error_panic_hook();

    wasm_bindgen_futures::spawn_local(async {
        super::setup().await;
    });
}
