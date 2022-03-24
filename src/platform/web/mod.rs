mod pool;

use std::panic;

use log::error;
use log::info;
use log::Level;
use winit::dpi::LogicalSize;
use winit::event_loop::EventLoop;
use winit::platform::web::WindowBuilderExtWebSys;
use winit::window::{Window, WindowBuilder};

use crate::io::scheduler::IOScheduler;
use crate::io::scheduler::ScheduleMethod;
use crate::io::scheduler::ThreadLocalTessellatorState;
use crate::MapBuilder;
use console_error_panic_hook;
pub use instant::Instant;
use scheduler::WebWorkerPoolScheduleMethod;
use scheduler::WebWorkerScheduleMethod;
use style_spec::source::TileAdressingScheme;
use wasm_bindgen::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::Window as WebSysWindow;
use web_sys::Worker;

// WebGPU
#[cfg(not(feature = "web-webgl"))]
pub const COLOR_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8Unorm;

// WebGL
#[cfg(feature = "web-webgl")]
pub const COLOR_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

#[wasm_bindgen(start)]
pub fn wasm_bindgen_start() {
    if let Err(_) = console_log::init_with_level(Level::Info) {
        // Failed to initialize logging. No need to log a message.
    }
    panic::set_hook(Box::new(console_error_panic_hook::hook));
}

#[wasm_bindgen]
pub fn create_scheduler() -> *mut IOScheduler {
    let scheduler = Box::new(IOScheduler::new(ScheduleMethod::WebWorker(
        WebWorkerScheduleMethod::new(),
    )));
    let scheduler_ptr = Box::into_raw(scheduler);
    return scheduler_ptr;
}

#[wasm_bindgen]
pub fn create_pool_scheduler(new_worker: js_sys::Function) -> *mut IOScheduler {
    let scheduler = Box::new(IOScheduler::new(ScheduleMethod::WebWorkerPool(
        WebWorkerPoolScheduleMethod::new(new_worker),
    )));
    let scheduler_ptr = Box::into_raw(scheduler);
    return scheduler_ptr;
}

pub fn get_body_size() -> Option<LogicalSize<i32>> {
    let web_window: WebSysWindow = web_sys::window().unwrap();
    let document = web_window.document().unwrap();
    let body = document.body().unwrap();
    Some(LogicalSize {
        width: body.client_width(),
        height: body.client_height(),
    })
}

pub fn get_canvas(element_id: &'static str) -> web_sys::HtmlCanvasElement {
    let web_window: WebSysWindow = web_sys::window().unwrap();
    let document = web_window.document().unwrap();
    document
        .get_element_by_id(element_id)
        .unwrap()
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .unwrap()
}

#[wasm_bindgen]
pub async fn run(scheduler_ptr: *mut IOScheduler) {
    let scheduler: Box<IOScheduler> = unsafe { Box::from_raw(scheduler_ptr) };

    // Either call forget or the main loop to keep worker loop alive
    MapBuilder::from_canvas("mapr")
        .with_existing_scheduler(scheduler)
        .build()
        .run_async()
        .await;

    // std::mem::forget(scheduler);
}

pub mod scheduler {
    use super::pool::WorkerPool;
    use crate::coords::{TileCoords, WorldTileCoords};
    use crate::error::Error;
    use crate::io::scheduler::{IOScheduler, ScheduleMethod, ThreadLocalTessellatorState};
    use crate::io::tile_cache::TileCache;
    use crate::io::TileRequestID;
    use js_sys::{ArrayBuffer, Error as JSError, Uint8Array};
    use log::warn;
    use std::thread::Thread;
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;
    use web_sys::Worker;
    use web_sys::{Request, RequestInit, RequestMode, Response, WorkerGlobalScope};

    #[wasm_bindgen]
    extern "C" {
        pub fn schedule_tile_request(url: &str, request_id: u32);
    }

    #[wasm_bindgen]
    pub fn new_tessellator_state(
        scheduler_ptr: *mut IOScheduler,
    ) -> *mut ThreadLocalTessellatorState {
        let scheduler: Box<IOScheduler> = unsafe { Box::from_raw(scheduler_ptr) };
        let tessellator_state = Box::new(scheduler.new_tessellator_state());
        let tessellator_state_ptr = Box::into_raw(tessellator_state);
        // Call forget such that scheduler does not get deallocated
        std::mem::forget(scheduler);
        return tessellator_state_ptr;
    }

    #[wasm_bindgen]
    pub fn tessellate_layers(
        tessellator_state_ptr: *mut ThreadLocalTessellatorState,
        request_id: u32,
        data: Box<[u8]>,
    ) {
        let tessellator_state: Box<ThreadLocalTessellatorState> =
            unsafe { Box::from_raw(tessellator_state_ptr) };

        tessellator_state
            .tessellate_layers(request_id, data)
            .unwrap();

        // Call forget such that scheduler does not get deallocated
        std::mem::forget(tessellator_state);
    }

    pub struct WebWorkerScheduleMethod;

    impl WebWorkerScheduleMethod {
        pub fn new() -> Self {
            Self
        }

        pub fn schedule_tile_request(
            &self,
            _scheduler: &IOScheduler,
            request_id: TileRequestID,
            coords: TileCoords,
        ) {
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
    }

    impl From<JsValue> for Error {
        fn from(maybe_error: JsValue) -> Self {
            assert!(maybe_error.is_instance_of::<JSError>());
            let error: JSError = maybe_error.dyn_into().unwrap();
            Error::Network(error.message().as_string().unwrap())
        }
    }

    pub struct WebWorkerPoolScheduleMethod {
        pool: WorkerPool,
    }

    impl WebWorkerPoolScheduleMethod {
        pub fn new(new_worker: js_sys::Function) -> Self {
            Self {
                pool: WorkerPool::new(
                    4,
                    Box::new(move || {
                        new_worker
                            .call0(&JsValue::undefined())
                            .unwrap()
                            .dyn_into::<Worker>()
                            .unwrap()
                    }),
                )
                .unwrap(),
            }
        }

        async fn fetch(
            state: ThreadLocalTessellatorState,
            request_id: TileRequestID,
            url: &str,
        ) -> Result<JsValue, JsValue> {
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

            state
                .tessellate_layers(request_id, output.into_boxed_slice())
                .unwrap();
            Ok(JsValue::undefined())
        }

        pub fn schedule_tile_request(
            &self,
            scheduler: &IOScheduler,
            request_id: TileRequestID,
            coords: TileCoords,
        ) {
            let state = scheduler.new_tessellator_state();

            self.pool
                .run(move || {
                    wasm_bindgen_futures::future_to_promise(async move {
                        let string = format!(
                            "https://maps.tuerantuer.org/europe_germany/{z}/{x}/{y}.pbf",
                            x = coords.x,
                            y = coords.y,
                            z = coords.z,
                        );
                        Self::fetch(state, request_id, string.as_str()).await
                    })
                })
                .unwrap();
        }
    }
}
