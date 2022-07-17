use maplibre::benchmarking::tessellation::{IndexDataType, OverAlignedVertexBuffer};
use maplibre::coords::{TileCoords, ViewRegion, WorldTileCoords, ZoomLevel};
use maplibre::error::Error;
use maplibre::headless::HeadlessMapWindowConfig;
use maplibre::io::pipeline::Processable;
use maplibre::io::pipeline::{PipelineContext, PipelineProcessor};

use maplibre::io::source_client::{HttpClient, HttpSourceClient};
use maplibre::io::tile_pipelines::build_vector_tile_pipeline;
use maplibre::io::tile_repository::StoredLayer;
use maplibre::io::{RawLayer, TileRequest};

use maplibre::platform::http_client::ReqwestHttpClient;
use maplibre::platform::run_multithreaded;
use maplibre::platform::schedule_method::TokioScheduleMethod;
use maplibre::render::settings::{RendererSettings, TextureFormat};
use maplibre::render::ShaderVertex;
use maplibre::window::{EventLoop, WindowSize};
use maplibre::MapBuilder;
use maplibre_winit::winit::WinitMapWindowConfig;

use maplibre::headless::utils::HeadlessPipelineProcessor;
use maplibre::style::source::TileAddressingScheme;
use maplibre::util::grid::google_mercator;
use maplibre::util::math::Aabb2;
use std::collections::HashSet;
use tile_grid::{extent_wgs84_to_merc, Extent, GridIterator};

#[cfg(feature = "trace")]
fn enable_tracing() {
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::Registry;

    let subscriber = Registry::default().with(tracing_tracy::TracyLayer::new());

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
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
            .with_map_window_config(HeadlessMapWindowConfig {
                size: WindowSize::new(1000, 1000).unwrap(),
            })
            .with_http_client(ReqwestHttpClient::new(None))
            .with_schedule_method(TokioScheduleMethod::new())
            .with_renderer_settings(RendererSettings {
                texture_format: TextureFormat::Rgba8UnormSrgb,
                ..RendererSettings::default()
            })
            .build()
            .initialize_headless()
            .await;

        let tile_limits = google_mercator().tile_limits(
            extent_wgs84_to_merc(&Extent {
                minx: 11.3475219363,
                miny: 48.0345697188,
                maxx: 11.7917815798,
                maxy: 48.255861,
            }),
            0,
        );

        for (z, x, y) in GridIterator::new(10, 10, tile_limits) {
            let coords = WorldTileCoords::from((x as i32, y as i32, z.into()));
            println!("Rendering {}", &coords);
            map.map_schedule
                .fetch_process(&coords)
                .await
                .expect("Failed to fetch and process!");

            match map.map_schedule_mut().update_and_redraw() {
                Ok(_) => {}
                Err(Error::Render(e)) => {
                    eprintln!("{}", e);
                    if e.should_exit() {}
                }
                e => eprintln!("{:?}", e),
            };
        }
    })
}

fn main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    #[cfg(feature = "trace")]
    enable_tracing();

    //run_headless();
    run_in_window();
}
