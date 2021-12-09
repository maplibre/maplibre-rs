use std::io;
use protobuf::ProtobufError;

#[derive(Debug)]
pub enum Error {
    Generic(String),
    Protobuf(ProtobufError),
    IO(io::Error),
}

impl From<ProtobufError> for Error {
    fn from(err: ProtobufError) -> Self {
        Error::Protobuf(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::IO(err)
    }
}
