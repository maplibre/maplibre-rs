use maplibre::coords::WorldTileCoords;
use maplibre::error::Error;
use maplibre::io::scheduler::ScheduleMethod;
use maplibre::io::source_client::{HttpClient, HttpSourceClient};
use maplibre::map_schedule::{EventuallyMapContext, MapSchedule};
use maplibre::platform::http_client::ReqwestHttpClient;
use maplibre::platform::run_multithreaded;
use maplibre::platform::schedule_method::TokioScheduleMethod;
use maplibre::render::settings::RendererSettings;
use maplibre::window::{EventLoop, MapWindow, MapWindowConfig, WindowSize};
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
        let mut map = MapBuilder::new()
            .with_map_window_config(HeadlessMapWindowConfig)
            .with_http_client(ReqwestHttpClient::new(None))
            .with_schedule_method(TokioScheduleMethod::new())
            .with_renderer_settings(RendererSettings {
                texture_format: TextureFormat::Rgba8UnormSrgb,
                ..RendererSettings::default()
            })
            .build()
            .initialize_headless()
            .await;

        let http_source_client: HttpSourceClient<HC> =
            HttpSourceClient::new(ReqwestHttpClient::new(None));

        let coords = WorldTileCoords::from((0, 0, 0));
        let request_id = 0;

        let x = match http_source_client.fetch(&coords).await {
            Ok(data) => state.process_tile(0, data.into_boxed_slice()).unwrap(),
            Err(e) => {
                log::error!("{:?}", &e);

                state.tile_unavailable(&coords, request_id).unwrap()
            }
        };

        match map.map_schedule_mut().map_context {
            EventuallyMapContext::Full(a) => a.tile_cache.put_tessellated_layer(),
            EventuallyMapContext::Premature(_) => {}
            EventuallyMapContext::_Uninitialized => {}
        }

        match map.map_schedule_mut().update_and_redraw() {
            Ok(_) => {}
            Err(Error::Render(e)) => {
                eprintln!("{}", e);
                if e.should_exit() {}
            }
            e => eprintln!("{:?}", e),
        };
    })
}

fn main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    #[cfg(feature = "trace")]
    enable_tracing();

    run_headless()
}
