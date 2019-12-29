use crate::info::TorrentInfo;
use std::sync::Weak;
use std::time::Duration;

pub struct TorrentStatus {
    save_path: String,
    name: String,
    torrent_file: Weak<TorrentInfo>,
    next_announce: Duration,

    // the URL of the last working tracker. If no tracker request has
    // been successful yet, it's set to an empty string.
    current_tracker: String,

    // the number of bytes downloaded and uploaded to all peers, accumulated,
    // *this session* only. The session is considered to restart when a
    // torrent is paused and restarted again. When a torrent is paused, these
    // counters are reset to 0. If you want complete, persistent, stats, see
    // ``all_time_upload`` and ``all_time_download``.
    total_downloaded: u64,
    total_upload: u64,

    // counts the amount of bytes send and received this session, but only
    // the actual payload data (i.e the interesting data), these counters
    // ignore any protocol overhead. The session is considered to restart
    // when a torrent is paused and restarted again. When a torrent is
    // paused, these counters are reset to 0.
    total_payload_download: u64,
    total_payload_upload: u64,

    // the number of bytes that has been downloaded and that has failed the
    // piece hash test. In other words, this is just how much crap that has
    // been downloaded since the torrent was last started. If a torrent is
    // paused and then restarted again, this counter will be reset.
    total_failed_bytes: u64,

    // the number of bytes that has been downloaded even though that data
    // already was downloaded. The reason for this is that in some situations
    // the same data can be downloaded by mistake. When libtorrent sends
    // requests to a peer, and the peer doesn't send a response within a
    // certain timeout, libtorrent will re-request that block. Another
    // situation when libtorrent may re-request blocks is when the requests
    // it sends out are not replied in FIFO-order (it will re-request blocks
    // that are skipped by an out of order block). This is supposed to be as
    // low as possible. This only counts bytes since the torrent was last
    // started. If a torrent is paused and then restarted again, this counter
    // will be reset.
    total_redundant_bytes: u64,
}

#[non_exhaustive]
pub enum State {
    // The torrent has not started its download yet, and is
    // currently checking existing files.
    CheckingFiles,

    // The torrent is trying to download metadata from peers.
    // This implies the ut_metadata extension is in use.
    DownloadingMetadata,

    // The torrent is being downloaded. This is the state
    // most torrents will be in most of the time. The progress
    // meter will tell how much of the files that has been
    // downloaded.
    Downloading,

    // In this state the torrent has finished downloading but
    // still doesn't have the entire torrent. i.e. some pieces
    // are filtered and won't get downloaded.
    Finished,

    // In this state the torrent has finished downloading and
    // is a pure seeder.
    Seeding,

    // If the torrent was started in full allocation mode, this
    // indicates that the (disk) storage for the torrent is
    // allocated.
    Allocating,

    // The torrent is currently checking the fast resume data and
    // comparing it to the files on disk. This is typically
    // completed in a fraction of a second, but if you add a
    // large number of torrents at once, they will queue up.
    CheckingResumeData,
}
