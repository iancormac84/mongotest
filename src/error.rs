use mongodb::{coll::error::WriteException, EncoderError};
use std::{fmt, io};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    BoxStd(Box<dyn std::error::Error + Send + Sync + 'static>),
    Io(io::Error),
    None(std::option::NoneError),
    Deserialize(serde::de::value::Error),
    BsonEncode(EncoderError),
    Json(serde_json::Error),
    Oid(mongodb::oid::Error),
    MongoDb(mongodb::error::Error),
    WriteException(WriteException),
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self)
    }
}

impl From<Box<dyn std::error::Error + Send + Sync + 'static>> for Error {
    fn from(err: Box<dyn std::error::Error + Send + Sync + 'static>) -> Error {
        Error::BoxStd(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<mongodb::error::Error> for Error {
    fn from(err: mongodb::error::Error) -> Error {
        Error::MongoDb(err)
    }
}

impl From<WriteException> for Error {
    fn from(err: WriteException) -> Error {
        Error::WriteException(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Error {
        Error::Json(err)
    }
}

impl From<std::option::NoneError> for Error {
    fn from(err: std::option::NoneError) -> Self {
        Error::None(err)
    }
}

impl From<serde::de::value::Error> for Error {
    fn from(err: serde::de::value::Error) -> Self {
        Error::Deserialize(err)
    }
}

impl From<EncoderError> for Error {
    fn from(err: EncoderError) -> Self {
        Error::BsonEncode(err)
    }
}

impl From<mongodb::oid::Error> for Error {
    fn from(err: mongodb::oid::Error) -> Self {
        Error::Oid(err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;
        match self {
            BoxStd(ref err) => err.fmt(fmt),
            Io(ref err) => err.fmt(fmt),
            None(ref err) => write!(fmt, "{:?}", err),
            Deserialize(ref err) => err.fmt(fmt),
            Json(ref err) => err.fmt(fmt),
            BsonEncode(ref err) => err.fmt(fmt),
            Oid(ref err) => err.fmt(fmt),
            MongoDb(ref err) => err.fmt(fmt),
            WriteException(ref err) => err.fmt(fmt),
        }
    }
}
