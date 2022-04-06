use std::panic;
use std::thread::Thread;
use winit::dpi::LogicalSize;
use winit::event_loop::EventLoop;
use winit::platform::web::WindowBuilderExtWebSys;
use winit::window::{Window, WindowBuilder};

use super::schedule_method::WebWorkerPoolScheduleMethod;
use console_error_panic_hook;
pub use instant::Instant;
use js_sys::{ArrayBuffer, Error as JSError, Uint8Array};
use style_spec::source::TileAddressingScheme;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::Window as WebSysWindow;
use web_sys::Worker;
use web_sys::{Request, RequestInit, RequestMode, Response, WorkerGlobalScope};

use crate::coords::{TileCoords, WorldTileCoords};
use crate::error::Error;
use crate::io::scheduler::ScheduleMethod;
use crate::io::scheduler::Scheduler;
use crate::io::scheduler::ThreadLocalState;
use crate::io::tile_cache::TileCache;
use crate::io::TileRequestID;
use crate::MapBuilder;

use super::pool::WorkerPool;

#[wasm_bindgen]
extern "C" {
    fn schedule_tile_request(url: &str, request_id: u32);
}

#[wasm_bindgen]
pub fn new_thread_local_state(scheduler_ptr: *mut Scheduler) -> *mut ThreadLocalState {
    let scheduler: Box<Scheduler> = unsafe { Box::from_raw(scheduler_ptr) };
    let state = Box::new(scheduler.new_thread_local_state());
    let state_ptr = Box::into_raw(state);
    // Call forget such that scheduler does not get deallocated
    std::mem::forget(scheduler);
    return state_ptr;
}

#[wasm_bindgen]
pub fn tessellate_layers(state_ptr: *mut ThreadLocalState, request_id: u32, data: Box<[u8]>) {
    let state: Box<ThreadLocalState> = unsafe { Box::from_raw(state_ptr) };

    state.process_tile(request_id, data).unwrap();

    // Call forget such that scheduler does not get deallocated
    std::mem::forget(state);
}

pub fn request_tile(request_id: TileRequestID, coords: TileCoords) {
    schedule_tile_request(
        format!(
            "https://maps.tuerantuer.org/europe_germany/{z}/{x}/{y}.pbf",
            x = coords.x,
            y = coords.y,
            z = coords.z,
        )
        .as_str(),
        request_id,
    )
}
