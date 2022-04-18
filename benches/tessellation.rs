use criterion::{criterion_group, criterion_main, Criterion};
use lyon::tessellation::VertexBuffers;
use mapr::benchmarking::io::static_tile_fetcher::StaticTileFetcher;
use mapr::benchmarking::tessellation::Tessellated;
use std::io::Cursor;

const MUNICH_X: u32 = 17425;
const MUNICH_Y: u32 = 11365;
const MUNICH_Z: u8 = 15;

fn parse_tile(c: &mut Criterion) {
    let fetcher = StaticTileFetcher::new();

    /*    c.bench_function("parse", |b| {
        b.iter(|| {
            parse_tile_reader(&mut Cursor::new(
                fetcher
                    .sync_fetch_tile(&(MUNICH_X, MUNICH_Y, MUNICH_Z).into())
                    .unwrap(),
            ))
            .expect("failed to load tile")
        })
    });*/
}

fn tessellate_tile(c: &mut Criterion) {
    /*    let fetcher = StaticTileFetcher::new();
    let tile = parse_tile_reader(&mut Cursor::new(
        fetcher
            .sync_fetch_tile(&(MUNICH_X, MUNICH_Y, MUNICH_Z).into())
            .unwrap(),
    ))
    .expect("failed to load tile");
    let layer = tile.layers().first().unwrap();

    fn tessselate(layer: &Layer) {
        let _: (VertexBuffers<_, u32>, _) = layer.tessellate().unwrap();
    }

    c.bench_function("tessselate", |b| b.iter(|| tessselate(layer)));*/
}

criterion_group!(benches, parse_tile, tessellate_tile);
criterion_main!(benches);
