use common::sha1::Sha1Hash;
use defaults::Defaults;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::download_priority::DownloadPriority;
use crate::flags::TorrentFlags;
use crate::info::TorrentInfo;

#[derive(Defaults)]
pub struct TorrentParams {
    version: usize,
    torrent_info: Arc<TorrentInfo>,
    pub trackers: Vec<String>,
    pub tracker_tiers: Vec<isize>,
    pub dht_nodes: Vec<(String, u16)>,
    pub name: String,
    save_path: String,
    storage_mode: (),
    storage: (),
    user_data: (),
    pub file_priorities: Vec<DownloadPriority>,
    flags: TorrentFlags,
    pub info_hash: Sha1Hash,
    max_uploads: isize,
    max_connections: isize,
    upload_limit: isize,
    download_limit: isize,
    total_uploaded: usize,
    total_downloaded: usize,
    active_time: Duration,
    finished_time: Duration,
    seeding_time: Duration,

    #[def = "Instant::now()"]
    added_time: Instant,

    completed_time: Option<Instant>,
    last_seen_complete: Option<Instant>,
    num_complete: isize,
    num_incomplete: isize,
    num_downloaded: isize,
    http_seeds: Vec<String>,
    pub url_seeds: Vec<String>,
    pub peers: Vec<SocketAddr>,
    banned_peers: Vec<SocketAddr>,
    unfinished_pieces: HashMap<(), ()>,
    have_pieces: (),
    verified_pieces: (),
    piece_priorities: Vec<()>,
    merkle_tree: Vec<Sha1Hash>,
    renamed_files: HashMap<(), ()>,
    last_download: Option<Instant>,
    last_upload: Option<Instant>,
}
