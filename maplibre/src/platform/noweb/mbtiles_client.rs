use std::{
    io::Read,
    path::{Path, PathBuf},
    sync::Arc,
};

use async_trait::async_trait;
use flate2::read::GzDecoder;
use rusqlite::{Connection, OpenFlags, OptionalExtension};
use thiserror::Error;

use crate::io::source_client::{HttpClient, SourceFetchError};

const MAX_ZOOM: u8 = 32;

/// Reads one installed MBTiles archive without network access.
///
/// The client gets an XYZ coordinate from the final three parts of each requested tile URL. It
/// converts the row to the TMS addressing scheme that MBTiles uses. It decompresses a gzip tile
/// before it returns the tile to the renderer.
#[derive(Clone, Debug)]
pub struct MbtilesClient {
    archive: Archive,
}

#[derive(Clone, Debug)]
enum Archive {
    Available(Arc<PathBuf>),
    Unavailable(Arc<str>),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct TileCoordinate {
    zoom: u8,
    column: u32,
    row: u32,
}

#[derive(Debug, Error)]
/// A local MBTiles source error.
pub enum MbtilesError {
    #[error("the tile URL has no valid z/x/y coordinate: {url}")]
    InvalidTileUrl { url: String },
    #[error("tile coordinate {zoom}/{column}/{row} is outside its zoom matrix")]
    CoordinateOutOfRange { zoom: u8, column: u32, row: u32 },
    #[error("the MBTiles client is unavailable: {reason}")]
    Unavailable { reason: Arc<str> },
    #[error("failed to open MBTiles archive {path}")]
    Open {
        path: PathBuf,
        #[source]
        source: rusqlite::Error,
    },
    #[error("MBTiles archive {path} has no tiles table")]
    MissingTilesTable { path: PathBuf },
    #[error("MBTiles archive {path} has no metadata table")]
    MissingMetadataTable { path: PathBuf },
    #[error("the MBTiles archive path is not valid UTF-8: {path}")]
    NonUtf8Path { path: PathBuf },
    #[error("failed to read tile {zoom}/{column}/{row} from MBTiles archive {path}")]
    Read {
        path: PathBuf,
        zoom: u8,
        column: u32,
        row: u32,
        #[source]
        source: rusqlite::Error,
    },
    #[error("MBTiles archive {path} has no tile {zoom}/{column}/{row}")]
    MissingTile {
        path: PathBuf,
        zoom: u8,
        column: u32,
        row: u32,
    },
    #[error("failed to decompress tile {zoom}/{column}/{row} from MBTiles archive {path}")]
    Decompress {
        path: PathBuf,
        zoom: u8,
        column: u32,
        row: u32,
        #[source]
        source: std::io::Error,
    },
    #[error("the MBTiles read task failed")]
    Task {
        #[source]
        source: tokio::task::JoinError,
    },
}

impl MbtilesClient {
    /// Opens and validates an MBTiles archive.
    ///
    /// The reader opens the archive in read-only mode. This function blocks on file I/O.
    pub fn open_blocking(path: impl Into<PathBuf>) -> Result<Self, MbtilesError> {
        let path = path.into();
        let connection = open_read_only(&path)?;
        if !has_table_or_view(&connection, &path, "tiles")? {
            return Err(MbtilesError::MissingTilesTable { path });
        }
        if !has_table_or_view(&connection, &path, "metadata")? {
            return Err(MbtilesError::MissingMetadataTable { path });
        }

        Ok(Self::configured(path))
    }

    pub(crate) fn configured(path: impl Into<PathBuf>) -> Self {
        Self {
            archive: Archive::Available(Arc::new(path.into())),
        }
    }

    pub(crate) fn unavailable(reason: impl Into<Arc<str>>) -> Self {
        Self {
            archive: Archive::Unavailable(reason.into()),
        }
    }

    fn archive_path(&self) -> Result<Arc<PathBuf>, MbtilesError> {
        match &self.archive {
            Archive::Available(path) => Ok(path.clone()),
            Archive::Unavailable(reason) => Err(MbtilesError::Unavailable {
                reason: reason.clone(),
            }),
        }
    }
}

#[cfg_attr(not(feature = "thread-safe-futures"), async_trait(?Send))]
#[cfg_attr(feature = "thread-safe-futures", async_trait)]
impl HttpClient for MbtilesClient {
    async fn fetch(&self, url: &str) -> Result<Vec<u8>, SourceFetchError> {
        let coordinate = parse_tile_coordinate(url).map_err(source_error)?;
        let archive_path = self.archive_path().map_err(source_error)?;
        tokio::task::spawn_blocking(move || read_tile_blocking(&archive_path, coordinate))
            .await
            .map_err(|source| source_error(MbtilesError::Task { source }))?
            .map_err(source_error)
    }
}

fn source_error(error: MbtilesError) -> SourceFetchError {
    SourceFetchError(Box::new(error))
}

fn parse_tile_coordinate(url: &str) -> Result<TileCoordinate, MbtilesError> {
    let path = url.split_once('?').map_or(url, |(path, _)| path);
    let mut segments = path.rsplit('/');
    let row = segments
        .next()
        .and_then(|value| value.split('.').next())
        .and_then(|value| value.parse::<u32>().ok());
    let column = segments.next().and_then(|value| value.parse::<u32>().ok());
    let zoom = segments.next().and_then(|value| value.parse::<u8>().ok());
    let (Some(zoom), Some(column), Some(row)) = (zoom, column, row) else {
        return Err(MbtilesError::InvalidTileUrl {
            url: url.to_string(),
        });
    };

    if zoom > MAX_ZOOM {
        return Err(MbtilesError::CoordinateOutOfRange { zoom, column, row });
    }
    let matrix_size = 1_u64 << zoom;
    if u64::from(column) >= matrix_size || u64::from(row) >= matrix_size {
        return Err(MbtilesError::CoordinateOutOfRange { zoom, column, row });
    }

    Ok(TileCoordinate { zoom, column, row })
}

fn read_tile_blocking(path: &Path, coordinate: TileCoordinate) -> Result<Vec<u8>, MbtilesError> {
    let connection = open_read_only(path)?;
    let matrix_size = 1_u64 << coordinate.zoom;
    let tms_row = matrix_size - 1 - u64::from(coordinate.row);
    let tile = connection
        .query_row(
            "SELECT tile_data FROM tiles \
             WHERE zoom_level = ?1 AND tile_column = ?2 AND tile_row = ?3",
            (
                i64::from(coordinate.zoom),
                i64::from(coordinate.column),
                tms_row as i64,
            ),
            |row| row.get::<_, Vec<u8>>(0),
        )
        .optional()
        .map_err(|source| MbtilesError::Read {
            path: path.to_path_buf(),
            zoom: coordinate.zoom,
            column: coordinate.column,
            row: coordinate.row,
            source,
        })?
        .ok_or_else(|| MbtilesError::MissingTile {
            path: path.to_path_buf(),
            zoom: coordinate.zoom,
            column: coordinate.column,
            row: coordinate.row,
        })?;

    if tile.starts_with(&[0x1f, 0x8b]) {
        let mut decoded = Vec::new();
        GzDecoder::new(tile.as_slice())
            .read_to_end(&mut decoded)
            .map_err(|source| MbtilesError::Decompress {
                path: path.to_path_buf(),
                zoom: coordinate.zoom,
                column: coordinate.column,
                row: coordinate.row,
                source,
            })?;
        Ok(decoded)
    } else {
        Ok(tile)
    }
}

fn open_read_only(path: &Path) -> Result<Connection, MbtilesError> {
    Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|source| MbtilesError::Open {
        path: path.to_path_buf(),
        source,
    })
}

fn has_table_or_view(
    connection: &Connection,
    path: &Path,
    name: &str,
) -> Result<bool, MbtilesError> {
    connection
        .query_row(
            "SELECT 1 FROM sqlite_schema WHERE type IN ('table', 'view') AND name = ?1",
            [name],
            |_| Ok(()),
        )
        .optional()
        .map(|entry| entry.is_some())
        .map_err(|source| MbtilesError::Open {
            path: path.to_path_buf(),
            source,
        })
}

#[cfg(test)]
mod tests;
