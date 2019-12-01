use common::sha1::Sha1Hash;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Default)]
pub struct FileStorage {
    piece_len: usize,
}

impl FileStorage {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_valid(&self) -> bool {
        self.piece_len > 0
    }
}

pub struct FileEntry {
    path: PathBuf,
    symlink_path: PathBuf,
    offset: i64,
    size: i64,
    modified_time: Instant,
    file_hash: Sha1Hash,
    pad_file: bool,
    hidden_attr: bool,
    executable_attr: bool,
    symlink_attr: bool,
}

pub struct InternalFileEntry {}
