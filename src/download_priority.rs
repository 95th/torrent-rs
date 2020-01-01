#[derive(Debug, Clone)]
pub enum DownloadPriority {
    DontDownload,
    LowPriority,
    DefaultPriority,
    TopPriority,
}
