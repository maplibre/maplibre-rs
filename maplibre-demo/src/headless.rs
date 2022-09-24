use maplibre::{
    coords::{LatLon, WorldTileCoords},
    error::Error,
    headless::{HeadlessEnvironment, HeadlessMapWindowConfig},
    io::apc::SchedulerAsyncProcedureCall,
    platform::{http_client::ReqwestHttpClient, scheduler::TokioScheduler},
    render::settings::{RendererSettings, TextureFormat},
    util::grid::google_mercator,
    window::WindowSize,
    MapBuilder,
};
use maplibre_winit::winit::WinitEnvironment;
use tile_grid::{extent_wgs84_to_merc, Extent, GridIterator};

pub async fn run_headless(tile_size: u32, min: LatLon, max: LatLon) {
    let client = ReqwestHttpClient::new(None);
    let mut map =
        MapBuilder::<HeadlessEnvironment<_, _, _, SchedulerAsyncProcedureCall<_, _>>>::new()
            .with_map_window_config(HeadlessMapWindowConfig {
                size: WindowSize::new(tile_size, tile_size).unwrap(),
            })
            .with_http_client(client.clone())
            .with_apc(SchedulerAsyncProcedureCall::new(
                client,
                TokioScheduler::new(),
            )) // FIXME (wasm-executor): avoid passing client and scheduler here
            .with_scheduler(TokioScheduler::new())
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
