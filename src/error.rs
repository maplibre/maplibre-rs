// error.rs
//
// Copyright (c) 2019-2021  Minnesota Department of Transportation
//
use protobuf::error::ProtobufError;

/// MVT Error types
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The tile already contains a layer with the specified name.
    #[error("Duplicate name")]
    DuplicateName(),

    /// The layer extent does not match the tile extent.
    #[error("Wrong layer extent")]
    WrongExtent(),

    /// The tile ID is invalid.
    #[error("Invalid tile ID")]
    InvalidTid(),

    /// The geometry does not meet criteria of the specification.
    #[error("Invalid geometry data")]
    InvalidGeometry(),

    /// Invalid float value
    #[error("Invalid float value")]
    InvalidValue(),

    /// Error while encoding protobuf data.
    #[error("Protobuf error {0}")]
    Protobuf(#[from] ProtobufError),
}

/// MVT Result
pub type Result<T> = std::result::Result<T, Error>;
