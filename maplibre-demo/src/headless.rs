use maplibre::{
    coords::{LatLon, WorldTileCoords},
    headless::{create_headless_renderer, map::HeadlessMap},
    style::Style,
    util::grid::google_mercator,
};
use tile_grid::{extent_wgs84_to_merc, Extent, GridIterator};

pub async fn run_headless(tile_size: u32, min: LatLon, max: LatLon) {
    let (kernel, renderer) = create_headless_renderer(tile_size, None).await;

    let style = Style::default();

    let requested_layers = style
        .layers
        .iter()
        .map(|layer| layer.source_layer.as_ref().unwrap().clone())
        .collect::<Vec<_>>();

    let mut map = HeadlessMap::new(style, renderer, kernel).unwrap();

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
            .fetch_tile(
                coords,
                &requested_layers
                    .iter()
                    .map(|layer| layer.as_str())
                    .collect::<Vec<_>>(),
            )
            .await
            .expect("Failed to fetch and process");

        map.render_tile(tile).expect("Rendering failed");
    }
}
