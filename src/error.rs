#[derive(Debug)]
pub enum Error {
    UnsupportedUrlProtocol,
    InvalidEscapedString,
    InvalidInfoHash,
    ParseInt,
    ParseString,
    ParseEndpoint,
    InvalidPort,
    ExpectedCloseBracketInAddr,
}

pub type Result<T> = std::result::Result<T, Error>;
