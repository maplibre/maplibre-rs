use criterion::{criterion_group, criterion_main, Criterion};
use lyon::tessellation::VertexBuffers;
use mapr::io::static_tile_fetcher::StaticTileFetcher;
use mapr::io::{static_tile_fetcher, TileFetcher};
use mapr::tesselation::Tesselated;
use std::io::Cursor;
use vector_tile::parse_tile_reader;
use vector_tile::tile::Tile;

fn tessselate_stroke(tile: &Tile) {
    let _: VertexBuffers<_, u16> = tile.tesselate_stroke();
}

fn tessselate_fill(tile: &Tile) {
    let _: VertexBuffers<_, u16> = tile.tesselate_fill();
}

fn tile1(c: &mut Criterion) {
    let fetcher = StaticTileFetcher::new();
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

    c.bench_function("tessselate_stroke", |b| b.iter(|| tessselate_stroke(&tile)));
    c.bench_function("tessselate_fill", |b| b.iter(|| tessselate_fill(&tile)));
}

criterion_group!(benches, tile1);
criterion_main!(benches);
