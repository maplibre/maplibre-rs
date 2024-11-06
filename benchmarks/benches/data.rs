use std::collections::HashSet;

use criterion::{criterion_group, criterion_main, Criterion};
use maplibre::{
    benchmarking::io::static_tile_fetcher::StaticTileFetcher,
    coords::{TileCoords, ZoomLevel},
    io::apc::{Context, IntoMessage, SendError},
    style::source::TileAddressingScheme,
    vector::{
        process_vector_tile, DefaultVectorTransferables, ProcessVectorContext, VectorTileRequest,
    },
};

// https://tile.openstreetmap.org/15/17425/11365.png
const MUNICH_COORDS: TileCoords = TileCoords {
    x: 17425,
    y: 11365,
    z: ZoomLevel::new(15u8),
};

pub struct DummyContext;

impl Context for DummyContext {
    fn send_back<T: IntoMessage>(&self, _message: T) -> Result<(), SendError> {
        Ok(())
    }
}

fn bench_process_vector_tile(c: &mut Criterion) {
    let fetcher = StaticTileFetcher::new();

    c.bench_function("process_vector_tile", |b| {
        let data = fetcher
            .sync_fetch_tile(&MUNICH_COORDS)
            .unwrap()
            .into_boxed_slice();

        b.iter(|| {
            let _ = process_vector_tile(
                &data,
                VectorTileRequest {
                    coords: MUNICH_COORDS
                        .into_world_tile(TileAddressingScheme::XYZ)
                        .unwrap(),
                    layers: HashSet::from([
                        "transportation".to_owned(),
                        "water".to_owned(),
                        "building".to_owned(),
                    ]),
                },
                &mut ProcessVectorContext::<DefaultVectorTransferables, _>::new(DummyContext),
            );
        })
    });
}

criterion_group!(benches, bench_process_vector_tile);
criterion_main!(benches);
