use crate::sha1::Sha1Hash;

const BIT_COUNTS: [usize; 16] = [
    zeros(0),
    zeros(1),
    zeros(2),
    zeros(3),
    zeros(4),
    zeros(5),
    zeros(6),
    zeros(7),
    zeros(8),
    zeros(9),
    zeros(10),
    zeros(11),
    zeros(12),
    zeros(13),
    zeros(14),
    zeros(15),
];

/// Count of number of zeros in lower 4 bits
const fn zeros(n: u8) -> usize {
    (0xf0 | n).count_zeros() as usize
}

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
        has_bits(key.data(), &self.bits)
    }

    pub fn set(&mut self, key: &Sha1Hash) {
        set_bits(key.data(), &mut self.bits);
    }

    pub fn clear(&mut self) {
        self.bits.iter_mut().for_each(|v| *v = 0);
    }

    pub fn size(&self) -> f64 {
        let c = std::cmp::min(count_zero_bits(&self.bits), (self.bits.len() * 8) - 1) as f64;
        let m = (self.bits.len() * 8) as f64;
        (c / m).ln() / (2_f64 * (1_f64 - (1_f64 / m)).ln())
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bits
    }
}

fn has_bits(key: &[u8], bits: &[u8]) -> bool {
    let mut idx1 = (key[0] as usize) | ((key[1] as usize) << 8);
    let mut idx2 = (key[2] as usize) | ((key[3] as usize) << 8);
    idx1 %= bits.len() * 8;
    idx2 %= bits.len() * 8;
    bits[idx1 / 8] & (1 << (idx1 & 7)) != 0 && bits[idx2 / 8] & (1 << (idx2 & 7)) != 0
}

fn set_bits(key: &[u8], bits: &mut [u8]) {
    let mut idx1 = (key[0] as usize) | ((key[1] as usize) << 8);
    let mut idx2 = (key[2] as usize) | ((key[3] as usize) << 8);
    idx1 %= bits.len() * 8;
    idx2 %= bits.len() * 8;
    bits[idx1 / 8] |= 1 << (idx1 & 7);
    bits[idx2 / 8] |= 1 << (idx2 & 7);
}

fn count_zero_bits(bits: &[u8]) -> usize {
    let mut ret = 0;
    for v in bits {
        ret += BIT_COUNTS[(*v & 0xf) as usize];
        ret += BIT_COUNTS[((*v >> 4) & 0xf) as usize];
    }
    ret
}

impl From<Vec<u8>> for BloomFilter {
    fn from(bits: Vec<u8>) -> BloomFilter {
        BloomFilter { bits }
    }
}

impl From<&[u8]> for BloomFilter {
    fn from(bits: &[u8]) -> BloomFilter {
        bits.to_vec().into()
    }
}

impl From<&str> for BloomFilter {
    fn from(s: &str) -> BloomFilter {
        s.as_bytes().into()
    }
}

impl From<String> for BloomFilter {
    fn from(s: String) -> BloomFilter {
        s.into_bytes().into()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::sha1::Sha1Hash;

    #[test]
    fn test_set_and_get() {
        let mut filter = BloomFilter::new(32);
        let k1 = Sha1Hash::update(b"test1");
        let k2 = Sha1Hash::update(b"test2");
        let k3 = Sha1Hash::update(b"test3");
        let k4 = Sha1Hash::update(b"test4");
        assert!(!filter.find(&k1));
        assert!(!filter.find(&k2));
        assert!(!filter.find(&k3));
        assert!(!filter.find(&k4));

        filter.set(&k1);
        assert!(filter.find(&k1));
        assert!(!filter.find(&k2));
        assert!(!filter.find(&k3));
        assert!(!filter.find(&k4));

        filter.set(&k4);
        assert!(filter.find(&k1));
        assert!(!filter.find(&k2));
        assert!(!filter.find(&k3));
        assert!(filter.find(&k4));
    }
    #[test]
    fn test_set_bits() {
        let mut bits = [0_u8; 4];

        for i in 0_u8..4 * 8 {
            let t = [i, 0, i, 0];
            assert!(!has_bits(&t, &bits));
        }

        for i in (0_u8..4 * 8).step_by(2) {
            let t = [i, 0, i, 0];
            assert!(!has_bits(&t, &bits));
            set_bits(&t, &mut bits);
            assert!(has_bits(&t, &bits));
        }

        let compare = [0x55_u8; 4];
        assert_eq!(compare, bits);
    }

    #[test]
    fn test_count_zeroes() {
        let mut bits = [0x00_u8, 0xff, 0x55, 0xaa];

        assert_eq!(count_zero_bits(&bits), 16);

        let t = [4_u8, 0, 4, 0];
        set_bits(&t, &mut bits);
        assert_eq!(count_zero_bits(&bits), 15);

        let compare = [0x10_u8, 0xff, 0x55, 0xaa];
        assert_eq!(compare, bits);
    }

    #[test]
    fn test_to_from_string() {
        let bits = [0x10_u8, 0xff, 0x55, 0xaa];

        let mut filter: BloomFilter = BloomFilter::from(bits.as_ref());
        let bits_out = filter.as_bytes();
        assert_eq!(bits_out, bits);

        let k = Sha1Hash::from(b"\x01\x00\x02\x00                ");
        assert!(!filter.find(&k));
        filter.set(&k);
        assert!(filter.find(&k));

        let compare = [0x16_u8, 0xff, 0x55, 0xaa];
        let bits_out = filter.as_bytes();
        assert_eq!(compare, bits_out);
    }
}
