pub mod bloom_filter;
pub mod ed25519;
pub mod random;
pub mod sha1;
pub mod types;

pub fn clamp<T: PartialOrd>(v: T, lo: T, hi: T) -> T {
    assert!(lo <= hi);
    if v < lo {
        lo
    } else if v > hi {
        hi
    } else {
        v
    }
}
