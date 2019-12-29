use bitflags::bitflags;

bitflags! {
    pub struct TorrentFlags: u64 {
        const SEED_MODE = 1;
        const UPLOAD_MODE = 1 << 1;
        const SHARE_MODE = 1 << 2;
        const APPLY_IP_FILTER = 1 << 3;
        const PAUSED = 1 << 4;
        const AUTO_MANAGED = 1 << 5;
        const DUPLICATE_IS_ERROR = 1 << 6;
        const UPDATE_SUBSCRIBE = 1 << 7;
        const SUPER_SEEDING = 1 << 8;
        const SEQUENTIAL_DOWNLOAD = 1 << 9;
        const STOP_WHEN_READY = 1 << 10;
        const OVERRIDE_TRACKERS = 1 << 11;
        const OVERRIDE_WEB_SEEDS = 1 << 12;
        const NEED_SAVE_RESUME = 1 << 13;
        const DISABLE_DHT = 1 << 19;
        const DISABLE_LSD = 1 << 20;
        const DISABLE_PEX = 1 << 21;
    }
}

impl Default for TorrentFlags {
    fn default() -> TorrentFlags {
        TorrentFlags::UPDATE_SUBSCRIBE
            | TorrentFlags::AUTO_MANAGED
            | TorrentFlags::PAUSED
            | TorrentFlags::APPLY_IP_FILTER
            | TorrentFlags::NEED_SAVE_RESUME
    }
}
