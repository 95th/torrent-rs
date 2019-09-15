use rand::{Rng, RngCore};

pub fn random_usize(max: usize) -> usize {
    rand::thread_rng().gen_range(0, max)
}

pub fn fill_bytes(buf: &mut [u8]) {
    rand::thread_rng().fill_bytes(buf)
}
