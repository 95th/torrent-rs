#[derive(Debug)]
pub enum Error {
    InvalidUrl,
    UnsupportedUrlProtocol,
    InvalidEscapedString,
    InvalidInfoHash,
    ParseInt,
    ParseString,
    ParseEndpoint,
    InvalidPort,
    ExpectedCloseBracketInAddr,
    MissingInfoHash,
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<url::ParseError> for Error {
    fn from(_e: url::ParseError) -> Self {
        Self::InvalidUrl
    }
}

impl From<std::net::AddrParseError> for Error {
    fn from(_e: std::net::AddrParseError) -> Self {
        Self::ParseEndpoint
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(_e: std::num::ParseIntError) -> Self {
        Self::ParseInt
    }
}
