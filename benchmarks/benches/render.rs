use criterion::{criterion_group, criterion_main, Criterion};
use maplibre::{
    coords::{WorldTileCoords, ZoomLevel},
    headless::{create_headless_renderer, map::HeadlessMap, HeadlessPlugin},
    platform::run_multithreaded,
    plugin::Plugin,
    render::RenderPlugin,
    style::Style,
    vector::{DefaultVectorTransferables, VectorPlugin},
};

fn headless_render(c: &mut Criterion) {
    c.bench_function("headless_render", |b| {
        let (mut map, layer) = run_multithreaded(async {
            let (kernel, renderer) = create_headless_renderer(1000, None).await;
            let style = Style::default();

            let plugins: Vec<Box<dyn Plugin<_>>> = vec![
                Box::new(RenderPlugin::default()),
                Box::new(VectorPlugin::<DefaultVectorTransferables>::default()),
                Box::new(HeadlessPlugin::new(false)),
            ];

            let map = HeadlessMap::new(style, renderer, kernel, plugins).unwrap();

            let tile = map
                .fetch_tile(WorldTileCoords::from((0, 0, ZoomLevel::default())))
                .await
                .expect("Failed to fetch!");

            let tile = map.process_tile(tile, &["water"]).await;

            (map, tile)
        });

        b.iter(|| {
            map.render_tile(layer.clone());
        });
    });
}

criterion_group!(name = benches;
    config = Criterion::default().significance_level(0.1).sample_size(20);
    targets = headless_render);
criterion_main!(benches);
