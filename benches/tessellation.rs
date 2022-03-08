use criterion::{criterion_group, criterion_main, Criterion};
use lyon::tessellation::VertexBuffers;
use mapr::io::static_tile_fetcher::StaticTileFetcher;
use mapr::io::{HttpFetcherConfig, TileFetcher};
use mapr::tessellation::Tessellated;
use std::io::Cursor;
use vector_tile::parse_tile_reader;
use vector_tile::tile::Layer;

fn tessselate(layer: &Layer) {
    let _: (VertexBuffers<_, u32>, _) = layer.tessellate().unwrap();
}

fn tile1(c: &mut Criterion) {
    const MUNICH_X: u32 = 17425;
    const MUNICH_Y: u32 = 11365;
    const MUNICH_Z: u8 = 15;

    let fetcher = StaticTileFetcher::new(HttpFetcherConfig::default());
    let tile = parse_tile_reader(&mut Cursor::new(
        fetcher
            .sync_fetch_tile(&(MUNICH_X, MUNICH_Y, MUNICH_Z).into())
            .unwrap(),
    ))
    .expect("failed to load tile");
    let layer = tile.layers().first().unwrap();

    c.bench_function("tessselate", |b| b.iter(|| tessselate(layer)));
}

criterion_group!(benches, tile1);
criterion_main!(benches);
