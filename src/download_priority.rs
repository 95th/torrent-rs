#[derive(Debug, Clone)]
pub enum DownloadPriority {
    DontDownload = 0,
    LowPriority = 1,
    DefaultPriority = 4,
    TopPriority = 7,
}
