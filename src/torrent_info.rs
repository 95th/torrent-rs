use crate::fs::FileStorage;

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
pub struct TorrentLimits {
    pub max_buf_size: usize,
    pub max_pieces: usize,
    pub max_decode_depth: usize,
    pub max_decode_tokens: usize,
}

impl Default for TorrentLimits {
    fn default() -> Self {
        Self {
            max_buf_size: 6000000,
            // the max number of pieces allowed in the torrent
            max_pieces: 0x100000,
            // the max recursion depth in the bdecoded structure
            max_decode_depth: 100,
            // the max number of bdecode tokens
            max_decode_tokens: 2000000,
        }
    }
}
