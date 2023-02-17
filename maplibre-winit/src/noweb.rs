//! Main (platform-specific) main loop which handles:
//! * Input (Mouse/Keyboard)
//! * Platform Events like suspend/resume
//! * Render a new frame

use std::marker::PhantomData;

use maplibre::{
    debug::DebugPlugin,
    event_loop::EventLoop,
    io::apc::SchedulerAsyncProcedureCall,
    kernel::{Kernel, KernelBuilder},
    map::Map,
    platform::{
        http_client::ReqwestHttpClient, run_multithreaded, scheduler::TokioScheduler,
        ReqwestOffscreenKernelEnvironment,
    },
    raster::{DefaultRasterTransferables, RasterPlugin},
    render::{builder::RendererBuilder, settings::WgpuSettings, RenderPlugin},
    style::Style,
    window::{MapWindow, MapWindowConfig, WindowSize},
};
use winit::window::WindowBuilder;

use super::WinitMapWindow;
use crate::{WinitEnvironment, WinitEventLoop};

pub struct WinitMapWindowConfig<ET> {
    title: String,

    phantom_et: PhantomData<ET>,
}

impl<ET> WinitMapWindowConfig<ET> {
    pub fn new(title: String) -> Self {
        Self {
            title,
            phantom_et: Default::default(),
        }
    }
}

impl<ET> MapWindow for WinitMapWindow<ET> {
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

impl<ET: 'static> MapWindowConfig for WinitMapWindowConfig<ET> {
    type MapWindow = WinitMapWindow<ET>;

    fn create(&self) -> Self::MapWindow {
        let raw_event_loop = winit::event_loop::EventLoopBuilder::<ET>::with_user_event().build();
        let window = WindowBuilder::new()
            .with_title(&self.title)
            .build(&raw_event_loop)
            .unwrap();

        Self::MapWindow {
            window,
            event_loop: Some(WinitEventLoop {
                event_loop: raw_event_loop,
            }),
        }
    }
}

pub fn run_headed_map(cache_path: Option<String>) {
    run_multithreaded(async {
        type Environment<S, HC, APC> =
            WinitEnvironment<S, HC, ReqwestOffscreenKernelEnvironment, APC, ()>;

        let client = ReqwestHttpClient::new(cache_path);

        let kernel: Kernel<Environment<_, _, _>> = KernelBuilder::new()
            .with_map_window_config(WinitMapWindowConfig::new("maplibre".to_string()))
            .with_http_client(client.clone())
            .with_apc(SchedulerAsyncProcedureCall::new(TokioScheduler::new()))
            .with_scheduler(TokioScheduler::new())
            .build();

        let renderer_builder = RendererBuilder::new().with_wgpu_settings(WgpuSettings {
            backends: Some(maplibre::render::settings::Backends::all()),
            ..WgpuSettings::default()
        });

        let mut map = Map::new(
            Style::default(),
            kernel,
            renderer_builder,
            vec![
                Box::new(RenderPlugin::default()),
                //Box::new(VectorPlugin::<DefaultVectorTransferables>::default()),
                Box::new(RasterPlugin::<DefaultRasterTransferables>::default()),
                Box::new(DebugPlugin::default()),
            ],
        )
        .unwrap();

        #[cfg(not(target_os = "android"))]
        {
            map.initialize_renderer().await.unwrap();
        }

        map.window_mut()
            .take_event_loop()
            .expect("Event loop is not available")
            .run(map, None)
    })
}
