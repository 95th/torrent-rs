use rand::Rng;

pub fn random(max: usize) -> usize {
    rand::thread_rng().gen_range(0, max)
}
