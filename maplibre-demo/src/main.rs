use maplibre::coords::{WorldTileCoords, ZoomLevel};
use maplibre::error::Error;
use maplibre::headless::HeadlessMapWindowConfig;

use maplibre::io::source_client::HttpSourceClient;

use maplibre::platform::http_client::ReqwestHttpClient;
use maplibre::platform::run_multithreaded;
use maplibre::platform::schedule_method::TokioScheduleMethod;
use maplibre::render::settings::{RendererSettings, TextureFormat};
use maplibre::window::WindowSize;
use maplibre::MapBuilder;
use maplibre_winit::winit::WinitMapWindowConfig;

use maplibre::style::Style;
use maplibre::tile::tile_parser::TileParser;
use maplibre::tile::tile_repository::StoredLayer::TessellatedLayer;
use maplibre::tile::tile_tessellator::TileTessellator;

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
            .with_style(Style::default())
            .build()
            .initialize_headless()
            .await;

        let http_source_client: HttpSourceClient<ReqwestHttpClient> =
            HttpSourceClient::new(ReqwestHttpClient::new(None));

        let coords = WorldTileCoords::from((0, 0, ZoomLevel::default()));

        let data = http_source_client
            .fetch(&coords)
            .await
            .unwrap()
            .into_boxed_slice();

        let tile = TileParser::parse(data);

        for mut layer in tile.layers {
            if !map
                .map_schedule
                .map_context
                .style
                .layers
                .iter()
                .any(|style_layer| {
                    style_layer
                        .source_layer
                        .as_ref()
                        .map_or(false, |layer_name| *layer_name == layer.name)
                })
            {
                continue;
            }

            tracing::info!("layer {} at {} ready", &layer.name, coords);

            match TileTessellator::tessellate_layer(&mut layer, &map.map_schedule.map_context.style)
            {
                Err(_) => {}
                Ok((vertex_buffer, feature_indices)) => {
                    map.map_schedule_mut()
                        .map_context
                        .tile_repository
                        .put_tessellated_layer(TessellatedLayer {
                            coords,
                            buffer: vertex_buffer,
                            feature_indices,
                            layer_data: layer,
                        });
                }
            }
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
