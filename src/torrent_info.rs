use crate::fs::FileStorage;
use defaults::Defaults;

#[derive(Default)]
pub struct TorrentInfo {
    files: FileStorage,
}

impl TorrentInfo {
    pub fn new() -> Self {
        Self::default()
    }
}

/// This object holds configuration options for limits to use when loading
/// torrents. They are meant to prevent loading potentially malicious torrents
/// that cause excessive memory allocations.
#[derive(Defaults)]
pub struct TorrentLimits {
    #[def = "6000000"]
    pub max_buf_size: usize,

    /// the max number of pieces allowed in the torrent
    #[def = "0x100000"]
    pub max_pieces: usize,

    /// the max recursion depth in the bdecoded structure
    #[def = "100"]
    pub max_decode_depth: usize,

    /// the max number of bdecode tokens
    #[def = "2000000"]
    pub max_decode_tokens: usize,
}
