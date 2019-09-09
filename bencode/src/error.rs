use std::fmt;
use std::io;

#[derive(Debug, PartialEq, PartialOrd)]
pub enum Error {
    IO,
    EOF,
    ParseInt,
    ParseBytes,
    ParseString,
    ParseList,
    ParseDict,
    InvalidChar(u8),
    ExpectedChar(u8),
    DepthLimit,
    ItemLimit,
}

pub type Result<T> = std::result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl std::error::Error for Error {}

impl From<Error> for io::Error {
    fn from(e: Error) -> Self {
        use Error::*;
        match e {
            IO => io::ErrorKind::Other.into(),
            EOF => io::ErrorKind::UnexpectedEof.into(),
            ParseInt => io::Error::new(io::ErrorKind::InvalidData, "Unable to parse int"),
            ParseBytes => io::Error::new(io::ErrorKind::InvalidData, "Unable to parse bytes"),
            ParseString => io::Error::new(io::ErrorKind::InvalidData, "Unable to parse string"),
            ParseList => io::Error::new(io::ErrorKind::InvalidData, "Unable to parse list"),
            ParseDict => io::Error::new(io::ErrorKind::InvalidData, "Unable to parse dictionary"),
            InvalidChar(v) => {
                io::Error::new(io::ErrorKind::InvalidData, format!("Unexpected {}", v))
            }
            ExpectedChar(v) => {
                io::Error::new(io::ErrorKind::InvalidData, format!("Expected {}", v))
            }
            DepthLimit => io::Error::new(io::ErrorKind::InvalidData, "Depth limit reached"),
            ItemLimit => io::Error::new(io::ErrorKind::InvalidData, "Item limit reached"),
        }
    }
}
