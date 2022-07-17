use std::collections::HashSet;

use criterion::{criterion_group, criterion_main, Criterion};
use maplibre::{
    coords::{WorldTileCoords, ZoomLevel},
    error::Error,
    headless::{utils::HeadlessPipelineProcessor, HeadlessMapWindowConfig},
    io::{
        pipeline::{PipelineContext, Processable},
        source_client::HttpSourceClient,
        tile_pipelines::build_vector_tile_pipeline,
        TileRequest,
    },
    platform::{
        http_client::ReqwestHttpClient, run_multithreaded, schedule_method::TokioScheduleMethod,
    },
    render::settings::{RendererSettings, TextureFormat},
    window::WindowSize,
    MapBuilder,
};

fn headless_render(c: &mut Criterion) {
    c.bench_function("headless_render", |b| {
        let mut map = run_multithreaded(async {
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

            map.map_schedule
                .fetch_process(&WorldTileCoords::from((0, 0, ZoomLevel::default())))
                .await
                .expect("Failed to fetch and process!");

            map
        });

        b.to_async(
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap(),
        )
        .iter(|| {
            match map.map_schedule_mut().update_and_redraw() {
                Ok(_) => {}
                Err(Error::Render(e)) => {
                    eprintln!("{}", e);
                    if e.should_exit() {}
                }
                e => eprintln!("{:?}", e),
            };
            async {}
        });
    });
}

criterion_group!(name = benches;
    config = Criterion::default().significance_level(0.1).sample_size(20);
    targets = headless_render);
criterion_main!(benches);
