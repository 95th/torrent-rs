use crate::sha1::Sha1Hash;

macro_rules! bloom_filter {
    ($ty:ident, $size:expr) => {
        pub struct $ty {
            bits: [u8; $size],
        }

        impl Default for $ty {
            fn default() -> Self {
                Self::new()
            }
        }

        impl $ty {
            pub fn new() -> Self {
                Self { bits: [0; $size] }
            }

            pub fn find(&self, key: &Sha1Hash) -> bool {
                has_bits(key.data(), &self.bits, $size)
            }

            pub fn set(&mut self, key: &Sha1Hash) {
                set_bits(key.data(), &mut self.bits, $size);
            }

            pub fn clear(&mut self) {
                self.bits.iter_mut().for_each(|v| *v = 0);
            }

            pub fn size(&self) -> f64 {
                let c = std::cmp::min(count_zero_bits(&self.bits, $size), ($size * 8) - 1) as f64;
                let m = ($size * 8) as f64;
                (c / m).ln() / (2_f64 * (1_f64 - (1_f64 / m)).ln())
            }

            pub fn as_bytes(&self) -> &[u8] {
                &self.bits
            }
        }

        impl From<Vec<u8>> for $ty {
            fn from(bits: Vec<u8>) -> Self {
                Self::from(&bits[..])
            }
        }

        impl From<&[u8]> for $ty {
            fn from(bits: &[u8]) -> Self {
                let mut f = Self::new();
                f.bits.copy_from_slice(bits);
                f
            }
        }

        impl From<[u8; $size]> for $ty {
            fn from(bits: [u8; $size]) -> Self {
                Self { bits }
            }
        }

        impl From<&str> for $ty {
            fn from(s: &str) -> Self {
                s.as_bytes().into()
            }
        }

        impl From<String> for $ty {
            fn from(s: String) -> Self {
                s.into_bytes().into()
            }
        }
    };
}

bloom_filter!(BloomFilter128, 128);
bloom_filter!(BloomFilter256, 256);

fn has_bits(key: &[u8], bits: &[u8], len: usize) -> bool {
    let mut idx1 = (key[0] as usize) | ((key[1] as usize) << 8);
    let mut idx2 = (key[2] as usize) | ((key[3] as usize) << 8);
    idx1 %= len * 8;
    idx2 %= len * 8;
    bits[idx1 / 8] & (1 << (idx1 & 7)) != 0 && bits[idx2 / 8] & (1 << (idx2 & 7)) != 0
}

fn set_bits(key: &[u8], bits: &mut [u8], len: usize) {
    let mut idx1 = (key[0] as usize) | ((key[1] as usize) << 8);
    let mut idx2 = (key[2] as usize) | ((key[3] as usize) << 8);
    idx1 %= len * 8;
    idx2 %= len * 8;
    bits[idx1 / 8] |= 1 << (idx1 & 7);
    bits[idx2 / 8] |= 1 << (idx2 & 7);
}

fn count_zero_bits(bits: &[u8], len: usize) -> usize {
    const BIT_COUNTS: [usize; 16] = [
        // 0000, 0001, 0010, 0011, 0100, 0101, 0110, 0111,
        // 1000, 1001, 1010, 1011, 1100, 1101, 1110, 1111
        4, 3, 3, 2, 3, 2, 2, 1, 3, 2, 2, 1, 2, 1, 1, 0,
    ];

    let mut ret = 0;
    for i in 0..len {
        ret += BIT_COUNTS[(bits[i] & 0xf) as usize];
        ret += BIT_COUNTS[((bits[i] >> 4) & 0xf) as usize];
    }
    ret
}

#[cfg(test)]
mod test {
    #![allow(unused)]
    use super::*;
    use crate::sha1::Sha1Hash;

    bloom_filter!(BloomFilter32, 32);
    bloom_filter!(BloomFilter4, 4);

    #[test]
    fn test_set_and_get() {
        let mut filter = BloomFilter32::new();
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
            assert!(!has_bits(&t, &bits, 6));
        }

        for i in (0_u8..4 * 8).step_by(2) {
            let t = [i, 0, i, 0];
            assert!(!has_bits(&t, &bits, 4));
            set_bits(&t, &mut bits, 4);
            assert!(has_bits(&t, &bits, 4));
        }

        let compare = [0x55_u8; 4];
        assert_eq!(compare, bits);
    }

    #[test]
    fn test_count_zeroes() {
        let mut bits = [0x00_u8, 0xff, 0x55, 0xaa];

        assert_eq!(count_zero_bits(&bits, 4), 16);

        let t = [4_u8, 0, 4, 0];
        set_bits(&t, &mut bits, 4);
        assert_eq!(count_zero_bits(&bits, 4), 15);

        let compare = [0x10_u8, 0xff, 0x55, 0xaa];
        assert_eq!(compare, bits);
    }

    #[test]
    fn test_to_from_string() {
        let bits = [0x10_u8, 0xff, 0x55, 0xaa];
        let mut filter = BloomFilter4::from(bits);
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
