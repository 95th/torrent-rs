#[derive(Debug)]
pub enum Error {
    UnsupportedUrlProtocol,
    InvalidEscapedString,
    InvalidInfoHash,
}

pub type Result<T> = std::result::Result<T, Error>;
