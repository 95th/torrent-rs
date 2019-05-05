use ed25519_dalek::{Keypair, Signature};
use digest::Digest;

fn main() {
    let mut rng = rand::thread_rng();
    let key_pair: Keypair = Keypair::generate(&mut rng);
    let msg: &[u8] = b"Hello world";

    let sign: Signature = key_pair.sign(msg);
    println!("{:?}", sign);
}