use std::path::PathBuf;
use url::Url;

pub struct Torrent {
    /// Tracker URL
    announce_url: Url,

    /// Suggested name to save the file in case of Single file torrent
    /// and directory name in case of Directory type torrent
    name: Option<String>,

    /// Length of each piece except last one which may be shorter.
    piece_len: usize,

    /// concatenated SHA1 hashes of pieces
    pieces: Vec<u8>,

    /// Kind of torrent - file or directory
    kind: TorrentKind,
}

pub enum TorrentKind {
    File(TorrentFile),
    Directory(Vec<TorrentFile>),
}

pub struct TorrentFile {
    path: PathBuf,
    len: usize,
}

impl Torrent {
    fn parse(bytes: &[u8]) -> Option<Self> {
        let parsed = bencode::ValueRef::decode(bytes).ok()?;
        let dict = parsed.as_dict()?;

        let announce_url = dict.get("announce")?.as_str()?;
        let info = dict.get("info")?.as_dict()?;
        let name = info.get("name")?.as_str()?;
        let piece_len = info.get("piece length")?.as_int()? as usize;
        let pieces = info.get("pieces")?.as_bytes()?;
        if pieces.len() % 20 != 0 {
            return None;
        }

        let kind;
        // Single file torrent
        if let Some(len) = info.get("length") {
            if info.contains_key("files") {
                // Can't have `files` key at the same time
                return None;
            }
            let len = len.as_int()? as usize;
            kind = TorrentKind::File(TorrentFile {
                path: name.into(),
                len,
            });
        }
        // Directory torrent
        else if let Some(files) = info.get("files") {
            let mut dir = vec![];
            for file in files.as_list()? {
                let file = file.as_dict()?;
                let len = file.get("length")?.as_int()? as usize;
                let mut path = PathBuf::new();
                for segment in file.get("path")?.as_list()? {
                    path.push(segment.as_str()?);
                }
                dir.push(TorrentFile { path, len });
            }
            kind = TorrentKind::Directory(dir);
        } else {
            return None;
        }

        Some(Torrent {
            announce_url: announce_url.parse().ok()?,
            name: Some(name.to_string()),
            piece_len,
            pieces: pieces.to_vec(),
            kind,
        })
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_parse() {
        // todo
    }
}
