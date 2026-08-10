use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use flate2::{write::GzEncoder, Compression};
use rusqlite::Connection;

use super::{parse_tile_coordinate, HttpClient, MbtilesClient, MbtilesError, TileCoordinate};

static NEXT_ARCHIVE: AtomicU64 = AtomicU64::new(0);

struct TestArchive(PathBuf);

impl TestArchive {
    fn new(tile: &[u8]) -> Self {
        let sequence = NEXT_ARCHIVE.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "maplibre-rs-mbtiles-{}-{sequence}.mbtiles",
            std::process::id()
        ));
        create_archive(&path, tile);
        Self(path)
    }
}

impl Drop for TestArchive {
    fn drop(&mut self) {
        fs::remove_file(&self.0).ok();
    }
}

#[tokio::test]
async fn reads_gzipped_xyz_tile_from_tms_archive_without_network() {
    let payload = b"mapbox-vector-tile";
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(payload).unwrap();
    let archive = TestArchive::new(&encoder.finish().unwrap());
    let bytes_before = fs::read(&archive.0).unwrap();
    let client = MbtilesClient::open_blocking(&archive.0).unwrap();

    let tile = client
        .fetch("https://must-not-resolve.invalid/2/1/2.pbf")
        .await
        .unwrap();

    assert_eq!(tile, payload);
    assert_eq!(fs::read(&archive.0).unwrap(), bytes_before);
}

#[tokio::test]
async fn returns_uncompressed_tile_unchanged() {
    let payload = b"png-data";
    let archive = TestArchive::new(payload);
    let client = MbtilesClient::open_blocking(&archive.0).unwrap();

    let tile = client.fetch("tiles/2/1/2.png?ignored=true").await.unwrap();

    assert_eq!(tile, payload);
}

#[test]
fn rejects_coordinates_outside_zoom_matrix() {
    let error = parse_tile_coordinate("tiles/2/4/0.pbf").unwrap_err();

    assert!(matches!(error, MbtilesError::CoordinateOutOfRange { .. }));
}

#[test]
fn rejects_zoom_above_u32_coordinate_range() {
    let error = parse_tile_coordinate("tiles/33/1/1.pbf").unwrap_err();

    assert!(matches!(error, MbtilesError::CoordinateOutOfRange { .. }));
}

#[test]
fn rejects_archive_without_tiles_table() {
    let archive = TestArchive::new(b"tile");
    fs::remove_file(&archive.0).unwrap();
    let connection = Connection::open(&archive.0).unwrap();
    connection
        .execute("CREATE TABLE metadata (name TEXT, value TEXT)", [])
        .unwrap();

    let error = MbtilesClient::open_blocking(&archive.0).unwrap_err();

    assert!(matches!(error, MbtilesError::MissingTilesTable { .. }));
}

#[test]
fn rejects_archive_without_metadata_table() {
    let archive = TestArchive::new(b"tile");
    let connection = Connection::open(&archive.0).unwrap();
    connection.execute("DROP TABLE metadata", []).unwrap();

    let error = MbtilesClient::open_blocking(&archive.0).unwrap_err();

    assert!(matches!(error, MbtilesError::MissingMetadataTable { .. }));
}

#[test]
fn accepts_normalized_tiles_view() {
    let archive = TestArchive::new(b"tile");
    let connection = Connection::open(&archive.0).unwrap();
    connection
        .execute("ALTER TABLE tiles RENAME TO tile_rows", [])
        .unwrap();
    connection
        .execute("CREATE VIEW tiles AS SELECT * FROM tile_rows", [])
        .unwrap();

    MbtilesClient::open_blocking(&archive.0).unwrap();
}

#[test]
fn parses_xyz_coordinate_with_extension_and_query() {
    let coordinate = parse_tile_coordinate("mbtiles://source/8/72/93.pbf?key=no-network").unwrap();

    assert_eq!(
        coordinate,
        TileCoordinate {
            zoom: 8,
            column: 72,
            row: 93,
        }
    );
}

fn create_archive(path: &Path, tile: &[u8]) {
    let connection = Connection::open(path).unwrap();
    connection
        .execute_batch(
            "CREATE TABLE metadata (name TEXT, value TEXT); \
             CREATE TABLE tiles (zoom_level INTEGER, tile_column INTEGER, \
             tile_row INTEGER, tile_data BLOB);",
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO tiles (zoom_level, tile_column, tile_row, tile_data) \
             VALUES (2, 1, 1, ?1)",
            [tile],
        )
        .unwrap();
}
