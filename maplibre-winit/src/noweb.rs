//! Main (platform-specific) main loop which handles:
//! * Input (Mouse/Keyboard)
//! * Platform Events like suspend/resume
//! * Render a new frame

use std::{marker::PhantomData, path::PathBuf};

use maplibre::{
    environment::OffscreenKernelConfig,
    event_loop::EventLoop,
    io::apc::SchedulerAsyncProcedureCall,
    kernel::{Kernel, KernelBuilder},
    map::Map,
    platform::{
        http_client::ReqwestHttpClient, run_multithreaded, scheduler::TokioScheduler,
        ReqwestOffscreenKernelEnvironment,
    },
    render::{builder::RendererBuilder, settings::WgpuSettings, RenderPlugin},
    style::Style,
    window::{MapWindow, MapWindowConfig, PhysicalSize, WindowCreateError},
};
use winit::window::WindowAttributes;

use super::WinitMapWindow;
use crate::{WinitEnvironment, WinitEventLoop};

#[derive(Clone)]
pub struct WinitMapWindowConfig<ET> {
    title: String,
    #[cfg(target_os = "android")]
    android_app: crate::android_activity::AndroidApp,

    phantom_et: PhantomData<ET>,
}

#[cfg(target_os = "android")]
impl<ET> WinitMapWindowConfig<ET> {
    pub fn new(title: String, android_app: winit::platform::android::activity::AndroidApp) -> Self {
        Self {
            title,
            android_app,
            phantom_et: Default::default(),
        }
    }
}

#[cfg(not(target_os = "android"))]
impl<ET> WinitMapWindowConfig<ET> {
    pub fn new(title: String) -> Self {
        Self {
            title,
            phantom_et: Default::default(),
        }
    }
}

impl<ET> MapWindow for WinitMapWindow<ET> {
    fn size(&self) -> PhysicalSize {
        let size = self.window.inner_size();
        #[cfg(target_os = "android")]
        // On android we can not get the dimensions of the window initially. Therefore, we use a
        // fallback until the window is ready to deliver its correct bounds.
        let window_size = PhysicalSize::new(size.width, size.height)
            .unwrap_or(PhysicalSize::new(100, 100).unwrap());

        #[cfg(not(target_os = "android"))]
        let window_size =
            PhysicalSize::new(size.width, size.height).expect("failed to get window dimensions.");
        window_size
    }
}

impl<ET: 'static + Clone> MapWindowConfig for WinitMapWindowConfig<ET> {
    type MapWindow = WinitMapWindow<ET>;

    fn create(&self) -> Result<Self::MapWindow, WindowCreateError> {
        let mut raw_event_loop_builder = winit::event_loop::EventLoop::<ET>::with_user_event();

        #[cfg(target_os = "android")]
        use winit::platform::android::EventLoopBuilderExtAndroid;
        #[cfg(target_os = "android")]
        let mut raw_event_loop_builder =
            raw_event_loop_builder.with_android_app(self.android_app.clone());

        let raw_event_loop = raw_event_loop_builder
            .build()
            .map_err(|_| WindowCreateError::EventLoop)?;

        let window = raw_event_loop
            .create_window(WindowAttributes::new().with_title(&self.title))
            .map_err(|_| WindowCreateError::Window)?;

        Ok(Self::MapWindow {
            window,
            event_loop: Some(WinitEventLoop {
                event_loop: raw_event_loop,
            }),
        })
    }
}

pub fn run_headed_map<P>(
    cache_path: Option<P>,
    window_config: WinitMapWindowConfig<()>,
    wgpu_settings: WgpuSettings,
) where
    P: Into<PathBuf>,
{
    run_multithreaded(async {
        type Environment<S, HC, APC> =
            WinitEnvironment<S, HC, ReqwestOffscreenKernelEnvironment, APC, ()>;

        let cache_path = cache_path.map(|path| path.into());
        let client = ReqwestHttpClient::new(cache_path.clone());

        let kernel: Kernel<Environment<_, _, _>> = KernelBuilder::new()
            .with_map_window_config(window_config)
            .with_http_client(client.clone())
            .with_apc(SchedulerAsyncProcedureCall::new(
                TokioScheduler::new(),
                OffscreenKernelConfig {
                    cache_directory: cache_path.map(|path| path.to_str().unwrap().to_string()),
                },
            ))
            .with_scheduler(TokioScheduler::new())
            .build();

        let renderer_builder = RendererBuilder::new().with_wgpu_settings(wgpu_settings);

        let mut map = Map::new(
            Style::default(),
            kernel,
            renderer_builder,
            vec![
                Box::new(RenderPlugin::default()),
                Box::new(maplibre::vector::VectorPlugin::<
                    maplibre::vector::DefaultVectorTransferables,
                >::default()),
                // Box::new(maplibre::raster::RasterPlugin::<
                //     maplibre::raster::DefaultRasterTransferables,
                // >::default()),
                #[cfg(debug_assertions)]
                Box::new(maplibre::debug::DebugPlugin::default()),
            ],
        )
        .unwrap();

        #[cfg(not(target_os = "android"))]
        {
            map.initialize_renderer().await.unwrap();
        }

        map.window_mut()
            .take_event_loop()
            .expect("event loop is not available")
            .run(map, None)
            .expect("event loop creation failed")
    })
}
