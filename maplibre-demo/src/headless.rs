use maplibre::headless::map::HeadlessMap;
use maplibre::headless::window::HeadlessMapWindowConfig;
use maplibre::kernel::KernelBuilder;
use maplibre::render::builder::RenderBuilder;
use maplibre::style::Style;
use maplibre::{
    coords::{LatLon, WorldTileCoords},
    error::Error,
    io::apc::SchedulerAsyncProcedureCall,
    platform::{http_client::ReqwestHttpClient, scheduler::TokioScheduler},
    render::settings::{RendererSettings, TextureFormat},
    util::grid::google_mercator,
    window::WindowSize,
};
use maplibre_winit::winit::WinitEnvironment;
use tile_grid::{extent_wgs84_to_merc, Extent, GridIterator};

pub async fn run_headless(tile_size: u32, min: LatLon, max: LatLon) {
    let client = ReqwestHttpClient::new(None);
    let kernel = KernelBuilder::new()
        .with_map_window_config(HeadlessMapWindowConfig::new(
            WindowSize::new(tile_size, tile_size).unwrap(),
        ))
        .with_http_client(client.clone())
        .with_apc(SchedulerAsyncProcedureCall::new(
            client,
            TokioScheduler::new(),
        ))
        .with_scheduler(TokioScheduler::new())
        .build();

    let renderer = RenderBuilder::new()
        .build()
        .initialize_headless_with(&kernel)
        .await
        .expect("Failed to initialize renderer");

    let mut map = HeadlessMap::new(Style::default(), renderer, kernel).unwrap();

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
        let tile = map
            .fetch_tile(coords, &["water"])
            .await
            .expect("Failed to fetch and process!");

        map.render_tile(tile);
    }
}
