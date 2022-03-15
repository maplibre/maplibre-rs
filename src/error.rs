//! Errors which can happen in various parts of the library.

use lyon::tessellation::TessellationError;

#[derive(Debug)]
pub enum Error {
    Network(String),
    File(String),
    Tesselation(TessellationError),
}

impl From<TessellationError> for Error {
    fn from(e: TessellationError) -> Self {
        Error::Tesselation(e)
    }
}
