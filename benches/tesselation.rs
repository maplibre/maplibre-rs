use criterion::{criterion_group, criterion_main, Criterion};
use lyon::tessellation::VertexBuffers;
use mapr::io::static_tile_fetcher::StaticTileFetcher;
use mapr::io::{static_tile_fetcher, HttpFetcherConfig, TileFetcher};
use mapr::tesselation::Tesselated;
use std::io::Cursor;
use vector_tile::parse_tile_reader;
use vector_tile::tile::{Layer, Tile};

fn tessselate(layer: &Layer) {
    let _: (VertexBuffers<_, u32>, _) = layer.tesselate().unwrap();
}

fn tile1(c: &mut Criterion) {
    let fetcher = StaticTileFetcher::new(HttpFetcherConfig::default());
    let tile = parse_tile_reader(&mut Cursor::new(
        fetcher
            .sync_fetch_tile(
                &(
                    mapr::example::MUNICH_X,
                    mapr::example::MUNICH_Y,
                    mapr::example::MUNICH_Z,
                )
                    .into(),
            )
            .unwrap(),
    ))
    .expect("failed to load tile");
    let layer = tile.layers().first().unwrap();

    c.bench_function("tessselate", |b| b.iter(|| tessselate(&layer)));
}

criterion_group!(benches, tile1);
criterion_main!(benches);
