#![allow(clippy::expect_used, clippy::panic)]

use super::canonical_tile;
use crate::coords::{WorldTileCoords, ZoomLevel};

#[test]
fn canonical_symbol_tile_wraps_antimeridian_copies() {
    let zoom = ZoomLevel::new(2);

    assert_eq!(
        canonical_tile(WorldTileCoords {
            x: -1,
            y: 1,
            z: zoom
        })
        .map(|tile| tile.x),
        Some(3)
    );
    assert_eq!(
        canonical_tile(WorldTileCoords {
            x: 4,
            y: 1,
            z: zoom
        })
        .map(|tile| tile.x),
        Some(0)
    );
}

#[test]
fn canonical_symbol_tile_rejects_polar_overflow() {
    let zoom = ZoomLevel::new(2);

    assert!(canonical_tile(WorldTileCoords {
        x: 0,
        y: -1,
        z: zoom
    })
    .is_none());
    assert!(canonical_tile(WorldTileCoords {
        x: 0,
        y: 4,
        z: zoom
    })
    .is_none());
}
