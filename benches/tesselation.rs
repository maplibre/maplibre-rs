use criterion::{black_box, criterion_group, criterion_main, Criterion};
use lyon::tessellation::VertexBuffers;
use mapr::io::static_database;
use mapr::render::shader_ffi::GpuVertexUniform;
use mapr::tesselation::Tesselated;
use std::io::Cursor;
use vector_tile::parse_tile_reader;
use vector_tile::tile::Tile;

fn tessselate_stroke(tile: &Tile) {
    let mut geometry: VertexBuffers<GpuVertexUniform, u32> = VertexBuffers::new();
    tile.tesselate_stroke(&mut geometry, 1);
}

fn tessselate_fill(tile: &Tile) {
    let mut geometry: VertexBuffers<GpuVertexUniform, u32> = VertexBuffers::new();
    tile.tesselate_fill(&mut geometry, 1);
}

fn tile1(c: &mut Criterion) {
    let tile = parse_tile_reader(&mut Cursor::new(
        static_database::get_tile(2179, 1421, 12)
            .unwrap()
            .contents(),
    ))
    .expect("failed to load tile");

    c.bench_function("tessselate_stroke", |b| b.iter(|| tessselate_stroke(&tile)));
    c.bench_function("tessselate_fill", |b| b.iter(|| tessselate_fill(&tile)));
}

fn tile2(c: &mut Criterion) {
    let tile = parse_tile_reader(&mut Cursor::new(
        static_database::get_tile(2180, 1421, 12)
            .unwrap()
            .contents(),
    ))
    .expect("failed to load tile");

    c.bench_function("tessselate_stroke", |b| b.iter(|| tessselate_stroke(&tile)));
    c.bench_function("tessselate_fill", |b| b.iter(|| tessselate_fill(&tile)));
}

criterion_group!(benches, tile1);
criterion_main!(benches);
