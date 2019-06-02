use std::fmt;
use std::ops::{BitXor, Deref, DerefMut};
use std::str::FromStr;

use hex::ToHex;
use num_bigint::{BigUint, RandBigInt};
use rand::prelude::*;
use std::rc::Rc;

const SIZE: usize = 20;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default)]
pub struct Id([u8; SIZE]);

impl Id {
    pub fn new() -> Id {
        let mut buf = [0; SIZE];
        rand::thread_rng().fill_bytes(&mut buf);
        buf.into()
    }

    /// Generate a random ID in given range of IDs
    pub fn ranged_random(range: &(Rc<Id>, Rc<Id>)) -> Id {
        let (lo, hi) = range;
        let lo = BigUint::from_bytes_be(&lo.0);
        let hi = BigUint::from_bytes_be(&hi.0);
        let random = rand::thread_rng().gen_biguint_range(&lo, &hi)
                                       .to_bytes_be();
        let mut buf = [0; SIZE];
        buf[SIZE - random.len()..].copy_from_slice(&random);
        buf.into()
    }

    pub fn at_dist(&self, bits: usize) -> Id {
        assert!(bits < SIZE * 8);

        let mut buf = [0; SIZE];
        let idx = (SIZE * 8 - bits) / 8;

        let clear_bits = 8 - (bits % 8) as u8;
        let mut byte = 0xFFu8;
        if clear_bits < 8 {
            byte >>= clear_bits;
        }
        buf[idx] = byte;

        buf.iter_mut()
           .skip(idx + 1)
           .for_each(|v| *v = 0xFF);

        self ^ &buf.into()
    }

    pub fn dist_to(&self, to: &Id) -> usize {
        let node = self ^ to;
        let mut zeros = 0;

        for byte in node.iter() {
            match byte {
                0 => zeros += 8,
                0xFF => break,
                v => {
                    let mut v = *v;
                    while v & 0x80 == 0 {
                        v <<= 1;
                        zeros += 1;
                    }
                    break;
                }
            }
        }
        SIZE * 8 - zeros
    }
}

#[derive(Debug)]
pub struct ParseError(String);

impl FromStr for Id {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Id, ParseError> {
        if s.len() != SIZE {
            return Err(ParseError(format!("Incorrect length. Expected {}, actual: {}", SIZE, s.len())));
        }

        let mut buf = [0; SIZE];
        buf.copy_from_slice(s.as_bytes());
        Ok(buf.into())
    }
}

impl From<[u8; 20]> for Id {
    fn from(buf: [u8; 20]) -> Id {
        Id(buf)
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.0.write_hex(f)?;
        Ok(())
    }
}

impl Deref for Id {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Id {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl BitXor for &Id {
    type Output = Id;

    fn bitxor(self, rhs: &Id) -> Id {
        let mut buf = [0; SIZE];
        buf.iter_mut()
           .zip(self.iter().zip(rhs.iter()))
           .for_each(|(a, (b, c))| *a = b ^ c);
        buf.into()
    }
}

#[cfg(test)]
mod test {
    use super::Id;
    use super::ParseError;

    #[test]
    fn does_parse() {
        let hash = "00000000000000000000";
        let id = hash.parse().unwrap();
        assert_eq!(Id([48; 20]), id);
    }

    #[test]
    fn does_not_parse() {
        let hash = "0000";
        let id: Result<Id, ParseError> = hash.parse();
        assert!(id.is_err());
    }

    #[test]
    fn xor() {
        let a = Id([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let b = Id([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2]);
        let expected: Id = Id([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 3]);
        assert_eq!(expected, &a ^ &b);
    }

    #[test]
    fn at_dist() {
        let id = Id([0; 20]);
        let far = id.at_dist(6);
        let mut buf = [0; 20];
        buf[19] = 0x3F;
        let expected = Id(buf);
        assert_eq!(expected, far);
    }

    #[test]
    fn get_dist() {
        let id = Id([0; 20]);
        let far = id.at_dist(6);
        assert_eq!(6, far.dist_to(&id));
    }

    #[test]
    fn rand() {
        let lo = [0; 20].into();
        let hi = [1; 20].into();

        let random = Id::ranged_random(&(lo, hi));
        assert!(random >= lo);
        assert!(random < hi);
    }
}
