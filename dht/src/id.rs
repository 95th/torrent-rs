use std::fmt;
use std::ops::{BitXor, Deref, DerefMut};
use std::rc::Rc;
use std::str::FromStr;

use hex::ToHex;
use num_bigint::{BigUint, RandBigInt};
use rand::prelude::*;

const SIZE: usize = 20;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Id([u8; SIZE]);

impl Id {
    pub fn new() -> Id {
        let mut buf = [0; SIZE];
        rand::thread_rng().fill_bytes(&mut buf);
        buf.into()
    }

    /// Generate a random ID in given range of IDs
    pub fn ranged_random(lo: &Id, hi: &Id) -> Id {
        let lo = BigUint::from_bytes_be(&lo.0);
        let hi = BigUint::from_bytes_be(&hi.0);
        let gen = rand::thread_rng().gen_biguint_range(&lo, &hi).to_bytes_be();
        let mut buf = [0; SIZE];
        buf[SIZE - gen.len()..].copy_from_slice(&gen);
        buf.into()
    }

    pub fn at_dist(&self, bits: usize) -> Rc<Id> {
        assert!(bits < SIZE * 8);

        let mut buf = [0; SIZE];
        let idx = (SIZE * 8 - bits) / 8;

        let clear_bits = 8 - (bits % 8) as u8;
        let mut byte = 0xFFu8;
        if clear_bits < 8 {
            byte >>= clear_bits;
        }
        buf[idx] = byte;

        buf.iter_mut().skip(idx + 1).for_each(|v| *v = 0xFF);

        Rc::new(self ^ &buf.into())
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
            return Err(ParseError(format!(
                "Incorrect length. Expected {}, actual: {}",
                SIZE,
                s.len()
            )));
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
