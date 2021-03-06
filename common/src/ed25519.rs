use crate::random;
use crate::types::{PublicKey, SecretKey};

pub fn create_seed() -> [u8; 32] {
    let mut seed = [0u8; 32];
    random::fill_bytes(&mut seed);
    seed
}

pub fn create_keypair() -> (PublicKey, SecretKey) {
    (PublicKey::default(), SecretKey::default())
}
