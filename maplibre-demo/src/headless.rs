use std::collections::HashSet;

use maplibre::coords::LatLon;
use maplibre::{
    benchmarking::tessellation::{IndexDataType, OverAlignedVertexBuffer},
    coords::{TileCoords, ViewRegion, WorldTileCoords, ZoomLevel},
    error::Error,
    headless::{utils::HeadlessPipelineProcessor, HeadlessMapWindowConfig},
    io::{
        pipeline::{PipelineContext, PipelineProcessor, Processable},
        source_client::{HttpClient, HttpSourceClient},
        tile_pipelines::build_vector_tile_pipeline,
        tile_repository::StoredLayer,
        RawLayer, TileRequest,
    },
    platform::{
        http_client::ReqwestHttpClient, run_multithreaded, schedule_method::TokioScheduleMethod,
    },
    render::{
        settings::{RendererSettings, TextureFormat},
        ShaderVertex,
    },
    style::source::TileAddressingScheme,
    util::{grid::google_mercator, math::Aabb2},
    window::{EventLoop, WindowSize},
    MapBuilder,
};
use maplibre_winit::winit::WinitMapWindowConfig;
use tile_grid::{extent_wgs84_to_merc, Extent, GridIterator};

pub async fn run_headless(tile_size: u32, min: LatLon, max: LatLon) {
    let mut map = MapBuilder::new()
        .with_map_window_config(HeadlessMapWindowConfig {
            size: WindowSize::new(tile_size, tile_size).unwrap(),
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
            minx: min.longitude,
            miny: min.latitude,
            maxx: max.longitude,
            maxy: max.latitude,
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
}
