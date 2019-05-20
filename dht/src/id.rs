use std::fmt;
use std::ops::{BitXor, Deref, DerefMut};
use std::str::FromStr;

use hex::ToHex;
use rand::prelude::*;

const SIZE: usize = 20;

#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct NodeId([u8; SIZE]);

impl NodeId {
    pub fn new() -> NodeId {
        let mut buf = [0; SIZE];
        rand::thread_rng().fill_bytes(&mut buf);
        NodeId(buf)
    }

    pub fn at_dist(&self, bits: usize) -> NodeId {
        assert_eq!(true, bits < SIZE * 8);

        let mut buf = [0; SIZE];
        let idx = (SIZE * 8 - bits) / 8;

        let clear_bits = 8 - (bits % 8);
        let mut byte = 0xFFu8;
        if clear_bits < 8 {
            byte >>= clear_bits;
        }
        buf[idx] = byte;

        buf.iter_mut()
           .skip(idx + 1)
           .for_each(|v| *v = 0xFF);

        *self ^ NodeId(buf)
    }

    pub fn get_dist(&self, to: NodeId) -> usize {
        let node = *self ^ to;
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

impl FromStr for NodeId {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<NodeId, ParseError> {
        if s.len() != SIZE {
            return Err(ParseError(format!("Incorrect length. Expected {}, actual: {}", SIZE, s.len())));
        }

        let mut buf = [0; SIZE];
        buf.copy_from_slice(s.as_bytes());
        Ok(NodeId(buf))
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.0.write_hex(f)?;
        Ok(())
    }
}

impl Deref for NodeId {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for NodeId {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl BitXor for NodeId {
    type Output = NodeId;

    fn bitxor(self, rhs: NodeId) -> Self::Output {
        let mut buf = [0; SIZE];
        buf.iter_mut()
           .zip(self.iter().zip(rhs.iter()))
           .for_each(|(a, (b, c))| *a = b ^ c);
        NodeId(buf)
    }
}

#[cfg(test)]
mod test {
    use super::NodeId;
    use super::ParseError;

    #[test]
    fn does_parse() {
        let hash = "00000000000000000000";
        let id = hash.parse().unwrap();
        assert_eq!(NodeId([48; 20]), id);
    }

    #[test]
    fn does_not_parse() {
        let hash = "0000";
        let id: Result<NodeId, ParseError> = hash.parse();
        assert_eq!(true, id.is_err());
    }

    #[test]
    fn xor() {
        let a = NodeId([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let b = NodeId([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2]);
        let expected: NodeId = NodeId([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 3]);
        assert_eq!(expected, a ^ b);
    }

    #[test]
    fn at_dist() {
        let id = NodeId([0; 20]);
        let far = id.at_dist(6);
        let mut buf = [0; 20];
        buf[19] = 0x3F;
        let expected = NodeId(buf);
        assert_eq!(expected, far);
    }

    #[test]
    fn get_dist() {
        let id = NodeId([0; 20]);
        let far = id.at_dist(6);
        assert_eq!(6, far.get_dist(id));
    }
}