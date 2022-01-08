use criterion::{criterion_group, criterion_main, Criterion};
use lyon::tessellation::VertexBuffers;
use mapr::io::static_database;
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
    let tile = parse_tile_reader(&mut Cursor::new(
        static_database::get_tile(&(2179u32, 1421u32, 12u8).into())
            .unwrap()
            .contents(),
    ))
    .expect("failed to load tile");

    c.bench_function("tessselate_stroke", |b| b.iter(|| tessselate_stroke(&tile)));
    c.bench_function("tessselate_fill", |b| b.iter(|| tessselate_fill(&tile)));
}

fn tile2(c: &mut Criterion) {
    let tile = parse_tile_reader(&mut Cursor::new(
        static_database::get_tile(&(2179u32, 1421u32, 12u8).into())
            .unwrap()
            .contents(),
    ))
    .expect("failed to load tile");

    c.bench_function("tessselate_stroke", |b| b.iter(|| tessselate_stroke(&tile)));
    c.bench_function("tessselate_fill", |b| b.iter(|| tessselate_fill(&tile)));
}

criterion_group!(benches, tile1);
criterion_main!(benches);
