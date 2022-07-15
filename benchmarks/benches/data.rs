use criterion::{criterion_group, criterion_main, Criterion};
use maplibre::benchmarking::io::static_tile_fetcher::StaticTileFetcher;
use maplibre::benchmarking::tessellation::Tessellated;
use maplibre::coords::{TileCoords, WorldTileCoords, ZoomLevel};
use maplibre::io::pipeline::{PipelineContext, PipelineProcessor, Processable};
use maplibre::io::tile_pipelines::{ParseTile, TessellateLayer};
use maplibre::io::TileRequest;
use maplibre::style::source::TileAddressingScheme;
use std::collections::HashSet;
use std::io::Cursor;

const MUNICH_COORDS: TileCoords = TileCoords {
    x: 17425,
    y: 11365,
    z: ZoomLevel::new(15u8),
};

pub struct DummyPipelineProcessor;

impl PipelineProcessor for DummyPipelineProcessor {}

fn parse_tile(c: &mut Criterion) {
    let fetcher = StaticTileFetcher::new();

    c.bench_function("parse", |b| {
        b.iter(|| {
            let request = TileRequest {
                coords: MUNICH_COORDS
                    .into_world_tile(TileAddressingScheme::XYZ)
                    .unwrap(),
                layers: HashSet::from(["boundary".to_owned(), "water".to_owned()]),
            };
            let data = fetcher
                .sync_fetch_tile(&MUNICH_COORDS)
                .unwrap()
                .into_boxed_slice();
            ParseTile::default().process(
                (request, 0, data),
                &mut PipelineContext::new(DummyPipelineProcessor),
            );
        })
    });
}

fn tessellate_tile(c: &mut Criterion) {
    let fetcher = StaticTileFetcher::new();
    let request = TileRequest {
        coords: MUNICH_COORDS
            .into_world_tile(TileAddressingScheme::XYZ)
            .unwrap(),
        layers: HashSet::from(["boundary".to_owned(), "water".to_owned()]),
    };
    let data = fetcher
        .sync_fetch_tile(&MUNICH_COORDS)
        .unwrap()
        .into_boxed_slice();
    let parsed = ParseTile::default().process(
        (request, 0, data),
        &mut PipelineContext::new(DummyPipelineProcessor),
    );

    c.bench_function("tessselate", |b| {
        b.iter(|| {
            TessellateLayer::default().process(
                parsed.clone(),
                &mut PipelineContext::new(DummyPipelineProcessor),
            );
        })
    });
}

criterion_group!(benches, parse_tile, tessellate_tile);
criterion_main!(benches);
