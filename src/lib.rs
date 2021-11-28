// lib.rs      mvt crate.
//
// Copyright (c) 2019-2021  Minnesota Department of Transportation
//
//! A library for encoding [mapbox vector tiles].
//!
//! A [tile] is composed of one or more [layer]s.  Each layer can have any number
//! of [feature]s, which contain the geometry to be rendered.  They can also have
//! metadata tags, which are key/value pairs.
//!
//! ## Example
//!
//! ```rust
//! use mvt::{Error, GeomEncoder, GeomType, Tile};
//! use pointy::Transform;
//!
//! fn main() -> Result<(), Error> {
//!     let mut tile = Tile::new(4096);
//!     let layer = tile.create_layer("First Layer");
//!     // NOTE: normally, the Transform would come from MapGrid::tile_transform
//!     let b = GeomEncoder::new(GeomType::Linestring, Transform::default())
//!         .point(0.0, 0.0)?
//!         .point(1024.0, 0.0)?
//!         .point(1024.0, 2048.0)?
//!         .point(2048.0, 2048.0)?
//!         .point(2048.0, 4096.0)?
//!         .encode()?;
//!     let mut feature = layer.into_feature(b);
//!     feature.set_id(1);
//!     feature.add_tag_string("key", "value");
//!     let layer = feature.into_layer();
//!     tile.add_layer(layer)?;
//!     let data = tile.to_bytes()?;
//!     println!("encoded {} bytes: {:?}", data.len(), data);
//!     Ok(())
//! }
//! ```
//!
//! [feature]: struct.Feature.html
//! [layer]: struct.Layer.html
//! [mapbox vector tiles]: https://github.com/mapbox/vector-tile-spec
//! [tile]: struct.Tile.html
#![forbid(unsafe_code)]

#[macro_use]
extern crate log;

mod encoder;
mod error;
mod mapgrid;
mod tile;
mod vector_tile;

pub use crate::encoder::{GeomData, GeomEncoder, GeomType};
pub use crate::error::Error;
pub use crate::mapgrid::{MapGrid, TileId};
pub use crate::tile::{Feature, Layer, Tile};
