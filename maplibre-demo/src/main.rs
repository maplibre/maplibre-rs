use maplibre::benchmarking::tessellation::{IndexDataType, OverAlignedVertexBuffer};
use maplibre::coords::{WorldTileCoords, ZoomLevel};
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
use std::collections::HashSet;

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

        let http_source_client: HttpSourceClient<ReqwestHttpClient> =
            HttpSourceClient::new(ReqwestHttpClient::new(None));

        let coords = WorldTileCoords::from((0, 0, ZoomLevel::default()));
        let request_id = 0;

        let data = http_source_client
            .fetch(&coords)
            .await
            .unwrap()
            .into_boxed_slice();

        let processor = HeadlessPipelineProcessor::default();
        let mut pipeline_context = PipelineContext::new(processor);
        let pipeline = build_vector_tile_pipeline();
        pipeline.process(
            (
                TileRequest {
                    coords,
                    layers: HashSet::from(["boundary".to_owned(), "water".to_owned()]),
                },
                request_id,
                data,
            ),
            &mut pipeline_context,
        );

        let mut processor = pipeline_context
            .take_processor::<HeadlessPipelineProcessor>()
            .unwrap();

        while let Some(v) = processor.layers.pop() {
            map.map_schedule_mut()
                .map_context
                .tile_repository
                .put_tessellated_layer(v);
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

    run_headless();
    run_in_window();
}
