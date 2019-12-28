use crate::download_priority::DownloadPriority;
use common::sha1::Sha1Hash;

pub trait Torrent {
    fn tracker_tiers(&self) -> &[isize];

    fn trackers(&self) -> &[String];

    fn url_seeds(&self) -> &[String];

    fn info_hash(&self) -> &Sha1Hash;

    fn file_priorities(&self) -> &[DownloadPriority];
}
