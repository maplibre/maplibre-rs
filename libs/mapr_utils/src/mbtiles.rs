use std::{fs, io};
use std::collections::{HashMap};
use std::fs::File;
use std::io::Write;
use std::path::{Path};

use flate2::bufread::GzDecoder;
use rusqlite::{Connection, Row};

#[derive(Debug)]
pub enum Error {
    IO(String)
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Error::IO(error.to_string())
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::IO(error.to_string())
    }
}


impl From<rusqlite::Error> for Error {
    fn from(error: rusqlite::Error) -> Self {
        Error::IO(error.to_string())
    }
}

pub fn extract<P: AsRef<Path>, R: AsRef<Path>>(input_mbtiles: P,
                                               output_dir: R)
                                               -> Result<(), Error> {
    let input_path = input_mbtiles.as_ref().to_path_buf();
    if !input_path.is_file() {
        return Err(Error::IO(format!("Input file {:?} is not a file", input_path)));
    }

    let output_path = output_dir.as_ref().to_path_buf();
    if output_path.exists() {
        return Err(Error::IO(format!("Output directory {:?} already exists", output_path)));
    }
    let connection = Connection::open(input_path)?;

    fs::create_dir_all(&output_path)?;

    extract_metadata(&connection, &output_path)?;

    // language=SQL
    let mut prepared_statement = connection
        .prepare("SELECT zoom_level, tile_column, tile_row, tile_data FROM tiles;")?;
    let mut tiles_rows = prepared_statement.query([])?;

    while let Ok(Some(tile)) = tiles_rows.next() {
        extract_tile(&tile, &output_path)?;
    }

    Ok(())
}

fn extract_tile(tile: &Row,
                output_path: &Path)
                -> Result<(), Error> {
    let (z, x, mut y): (u32, u32, u32) = (tile.get::<_, u32>(0)?,
                                          tile.get::<_, u32>(1)?,
                                          tile.get::<_, u32>(2)?);

    // Flip vertical axis
    y = 2u32.pow(z) - 1 - y;

    let tile_dir = output_path.join(format!("{}/{}", z, x));

    fs::create_dir_all(&tile_dir)?;

    let tile_path = tile_dir.join(format!("{}.{}", y, "pbf"));
    let tile_data = tile.get::<_, Vec<u8>>(3)?;
    let mut decoder = GzDecoder::new(tile_data.as_ref());

    let mut tile_file = File::create(tile_path)?;
    io::copy(&mut decoder, &mut tile_file)?;
    Ok(())
}

fn extract_metadata(connection: &Connection,
                    output_path: &Path)
                    -> Result<(), Error> {
    // language=SQL
    let mut prepared_statement = connection
        .prepare("SELECT name, value FROM metadata;")?;
    let metadata = prepared_statement
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;

    let metadata_map: HashMap<String, String> = metadata.filter_map(|result| {
        match result {
            Ok(tuple) => Some(tuple),
            Err(_) => None,
        }
    }).collect::<HashMap<String, String>>();

    let json_string = serde_json::to_string(&metadata_map)?;
    let metadata_path = output_path.join("metadata.json");
    let mut metadata_file = File::create(metadata_path)?;
    metadata_file.write(json_string.as_bytes())?;
    Ok(())
}
