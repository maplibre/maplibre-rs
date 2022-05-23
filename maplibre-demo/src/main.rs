use maplibre::error::Error;
use maplibre::io::scheduler::ScheduleMethod;
use maplibre::io::source_client::HTTPClient;
use maplibre::map_schedule::MapSchedule;
use maplibre::platform::http_client::ReqwestHttpClient;
use maplibre::platform::run_multithreaded;
use maplibre::platform::schedule_method::TokioScheduleMethod;
use maplibre::render::settings::RendererSettings;
use maplibre::window::{MapWindow, MapWindowConfig, Runnable, WindowSize};
use maplibre::MapBuilder;
use maplibre_winit::winit::{WinitEventLoop, WinitMapWindow, WinitMapWindowConfig, WinitWindow};
use wgpu::TextureFormat;

#[cfg(feature = "trace")]
fn enable_tracing() {
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::Registry;

    let subscriber = Registry::default().with(tracing_tracy::TracyLayer::new());

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
}
pub struct HeadlessMapWindowConfig;

impl MapWindowConfig for HeadlessMapWindowConfig {
    type MapWindow = HeadlessMapWindow;

    fn create(&self) -> Self::MapWindow {
        Self::MapWindow {}
    }
}

pub struct HeadlessMapWindow;

impl MapWindow for HeadlessMapWindow {
    fn size(&self) -> WindowSize {
        WindowSize::new(1920, 1080).unwrap()
    }
}

impl<MWC, SM, HC> Runnable<MWC, SM, HC> for HeadlessMapWindow
where
    MWC: MapWindowConfig<MapWindow = HeadlessMapWindow>,
    SM: ScheduleMethod,
    HC: HTTPClient,
{
    fn run(mut self, mut map_state: MapSchedule<MWC, SM, HC>, max_frames: Option<u64>) {
        for i in 0..3 {
            match map_state.update_and_redraw() {
                Ok(_) => {}
                Err(Error::Render(e)) => {
                    eprintln!("{}", e);
                    if e.should_exit() {}
                }
                e => eprintln!("{:?}", e),
            };
        }
    }
}

fn run_in_window() {
    run_multithreaded(async {
        MapBuilder::new()
            .with_map_window_config(WinitMapWindowConfig::new("maplibre".to_string()))
            .with_http_client(ReqwestHttpClient::new(None))
            .with_schedule_method(TokioScheduleMethod::new())
            .build()
            .initialize()
            .await
            .run()
    })
}

fn run_headless() {
    run_multithreaded(async {
        MapBuilder::new()
            .with_map_window_config(HeadlessMapWindowConfig)
            .with_http_client(ReqwestHttpClient::new(None))
            .with_schedule_method(TokioScheduleMethod::new())
            .with_renderer_settings(RendererSettings {
                texture_format: TextureFormat::Rgba8UnormSrgb,
                ..RendererSettings::default()
            })
            .build()
            .initialize_headless()
            .await
            .run()
    })
}

fn main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    #[cfg(feature = "trace")]
    enable_tracing();

    run_headless()
}
