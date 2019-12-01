use bitflags::bitflags;

bitflags! {
    struct Announce: u8 {
        const SEED = 1;
        const IMPLIED_PORT = 2;
        const SSL_TORRENT = 4;
    }
}
