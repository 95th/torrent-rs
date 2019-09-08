use crate::Sha1Hash;

const BIT_COUNTS: [usize; 16] = [
    4, // 0000
    3, // 0001
    3, // 0010
    2, // 0011
    3, // 0100
    2, // 0101
    2, // 0110
    1, // 0111
    3, // 1000
    2, // 1001
    2, // 1010
    1, // 1011
    2, // 1100
    1, // 1101
    1, // 1110
    0, // 1111
];

pub struct BloomFilter {
    bits: Vec<u8>,
}

impl BloomFilter {
    pub fn new(size: usize) -> BloomFilter {
        BloomFilter {
            bits: vec![0; size],
        }
    }

    pub fn find(&self, key: &Sha1Hash) -> bool {
        let mut idx1 = (key[0] as usize) | ((key[1] as usize) << 8);
        let mut idx2 = (key[2] as usize) | ((key[3] as usize) << 8);
        idx1 %= self.bits.len() * 8;
        idx2 %= self.bits.len() * 8;
        self.bits[idx1 / 8] & (1 << (idx1 & 7)) != 0 && self.bits[idx2 / 8] & (1 << (idx2 & 7)) != 0
    }

    pub fn set(&mut self, key: &Sha1Hash) {
        let mut idx1 = (key[0] as usize) | ((key[1] as usize) << 8);
        let mut idx2 = (key[2] as usize) | ((key[3] as usize) << 8);
        idx1 %= self.bits.len() * 8;
        idx2 %= self.bits.len() * 8;
        self.bits[idx1 / 8] |= 1 << (idx1 & 7);
        self.bits[idx2 / 8] |= 1 << (idx2 & 7);
    }

    pub fn clear(&mut self) {
        self.bits.iter_mut().for_each(|v| *v = 0);
    }

    pub fn size(&self) -> f64 {
        let c = std::cmp::min(self.count_zeros_bits(), (self.bits.len() * 8) - 1) as f64;
        let m = (self.bits.len() * 8) as f64;
        (c / m).ln() / (2_f64 * (1_f64 - (1_f64 / m)).ln())
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bits
    }

    fn count_zeros_bits(&self) -> usize {
        let mut ret = 0;
        for v in &self.bits {
            ret += BIT_COUNTS[(*v & 0xf) as usize];
            ret += BIT_COUNTS[((*v >> 4) & 0xf) as usize];
        }
        ret
    }
}
