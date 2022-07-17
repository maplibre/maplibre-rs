use maplibre::{
    coords::TileCoords,
    io::{scheduler::Scheduler, TileRequestID},
    stages::SharedThreadState,
};
use wasm_bindgen::prelude::*;

use super::schedule_method::WebWorkerPoolScheduleMethod;

#[wasm_bindgen]
extern "C" {
    fn schedule_tile_request(url: &str, request_id: u32);
}

// FIXME
/*#[wasm_bindgen]
pub fn new_thread_local_state(scheduler_ptr: *mut Scheduler) -> *mut SharedThreadState {
    let scheduler: Box<Scheduler> = unsafe { Box::from_raw(scheduler_ptr) };
    let state = Box::new(scheduler.new_thread_local_state());
    let state_ptr = Box::into_raw(state);
    // Call forget such that scheduler does not get deallocated
    std::mem::forget(scheduler);
    return state_ptr;
}*/

#[wasm_bindgen]
pub fn new_thread_local_state(_scheduler_ptr: *mut Scheduler<WebWorkerPoolScheduleMethod>) -> u32 {
    0
}

#[wasm_bindgen]
pub fn tessellate_layers(state_ptr: *mut SharedThreadState, request_id: u32, data: Box<[u8]>) {
    let state: Box<SharedThreadState> = unsafe { Box::from_raw(state_ptr) };

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
