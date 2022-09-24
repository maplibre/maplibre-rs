//! Main (platform-specific) main loop which handles:
//! * Input (Mouse/Keyboard)
//! * Platform Events like suspend/resume
//! * Render a new frame

use maplibre::{
    io::apc::SchedulerAsyncProcedureCall,
    platform::{http_client::ReqwestHttpClient, run_multithreaded, scheduler::TokioScheduler},
    window::{HeadedMapWindow, MapWindow, MapWindowConfig, WindowSize},
    MapBuilder,
};
use winit::window::WindowBuilder;

use super::{WinitEventLoop, WinitMapWindow, WinitMapWindowConfig, WinitWindow};
use crate::winit::WinitEnvironment;

impl MapWindow for WinitMapWindow {
    fn size(&self) -> WindowSize {
        let size = self.window.inner_size();
        #[cfg(target_os = "android")]
        // On android we can not get the dimensions of the window initially. Therefore, we use a
        // fallback until the window is ready to deliver its correct bounds.
        let window_size =
            WindowSize::new(size.width, size.height).unwrap_or(WindowSize::new(100, 100).unwrap());

        #[cfg(not(target_os = "android"))]
        let window_size =
            WindowSize::new(size.width, size.height).expect("failed to get window dimensions.");
        window_size
    }
}
impl HeadedMapWindow for WinitMapWindow {
    type RawWindow = WinitWindow;

    fn inner(&self) -> &Self::RawWindow {
        &self.window
    }
}

impl MapWindowConfig for WinitMapWindowConfig {
    type MapWindow = WinitMapWindow;

    fn create(&self) -> Self::MapWindow {
        let event_loop = WinitEventLoop::new();
        let window = WindowBuilder::new()
            .with_title(&self.title)
            .build(&event_loop)
            .unwrap();

        Self::MapWindow {
            window,
            event_loop: Some(event_loop),
        }
    }
}

pub fn run_headed_map(cache_path: Option<String>) {
    run_multithreaded(async {
        let client = ReqwestHttpClient::new(cache_path);
        MapBuilder::<WinitEnvironment<_, _, _, SchedulerAsyncProcedureCall<_, _>>>::new()
            .with_map_window_config(WinitMapWindowConfig::new("maplibre".to_string()))
            .with_http_client(client.clone())
            .with_apc(SchedulerAsyncProcedureCall::new(
                client,
                TokioScheduler::new(),
            ))
            .with_scheduler(TokioScheduler::new())
            .build()
            .initialize()
            .await
            .run()
    })
}
