use std::net::IpAddr;

const SIZE: usize = 20;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct Sha1Hash {
    data: [u8; SIZE],
}

impl Sha1Hash {
    pub const fn new() -> Self {
        Self::min()
    }

    pub const fn max() -> Self {
        Self {
            data: [u8::max_value(); SIZE],
        }
    }

    pub const fn min() -> Self {
        Self {
            data: [u8::min_value(); SIZE],
        }
    }

    pub fn update(bytes: &[u8]) -> Self {
        let mut h = sha1::Sha1::new();
        h.update(bytes);
        let digest = h.digest();
        digest.bytes().into()
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        assert_eq!(bytes.len(), SIZE);
        let mut buf = [0; SIZE];
        buf.copy_from_slice(bytes);
        buf.into()
    }

    pub fn from_address(addr: &IpAddr) -> Sha1Hash {
        match addr {
            IpAddr::V4(addr4) => Sha1Hash::update(&addr4.octets()),
            IpAddr::V6(addr6) => Sha1Hash::update(&addr6.octets()),
        }
    }

    pub fn data(&self) -> &[u8; SIZE] {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut [u8; SIZE] {
        &mut self.data
    }

    pub fn clear(&mut self) {
        self.data.iter_mut().for_each(|v| *v = 0);
    }

    pub fn all_zeroes(&self) -> bool {
        self.data.iter().all(|v| *v == 0)
    }

    pub fn leading_zeros(&self) -> usize {
        let mut count = 0;
        for v in &self.data {
            if *v == 0 {
                count += 8;
            } else {
                count += v.leading_zeros() as usize;
                break;
            }
        }
        count
    }

    pub fn iter(&self) -> impl Iterator<Item = &u8> {
        self.data.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut u8> {
        self.data.iter_mut()
    }
}

impl From<&[u8; SIZE]> for Sha1Hash {
    fn from(data: &[u8; SIZE]) -> Self {
        Sha1Hash::from(*data)
    }
}

impl From<[u8; SIZE]> for Sha1Hash {
    fn from(data: [u8; SIZE]) -> Self {
        Sha1Hash { data }
    }
}

impl std::ops::ShlAssign<usize> for Sha1Hash {
    #[allow(clippy::suspicious_op_assign_impl)]
    fn shl_assign(&mut self, mut shift: usize) {
        let shift_bytes = shift / 8;
        if shift_bytes >= SIZE {
            self.clear();
            return;
        }

        if shift_bytes > 0 {
            debug_assert!(shift_bytes < self.data.len());
            unsafe {
                std::ptr::copy(
                    self.data[shift_bytes..].as_mut_ptr(),
                    self.data.as_mut_ptr(),
                    (SIZE - shift_bytes) * 8,
                );
            }
            self.data[(SIZE - shift_bytes)..]
                .iter_mut()
                .for_each(|v| *v = 0);
            shift -= shift_bytes * 8;
        }

        debug_assert!(shift < 8);
        if shift > 0 {
            let mut carry = 0_u8;
            for i in (0..SIZE).rev() {
                let last_carry = carry;
                carry = (self.data[i] & 0xff << (8 - shift)) >> (8 - shift);
                self.data[i] <<= shift;
                self.data[i] |= last_carry;
            }
        }
    }
}

impl std::ops::ShrAssign<usize> for Sha1Hash {
    #[allow(clippy::suspicious_op_assign_impl)]
    fn shr_assign(&mut self, mut shift: usize) {
        let shift_bytes = shift / 8;
        if shift_bytes >= SIZE {
            self.clear();
            return;
        }

        if shift_bytes > 0 {
            debug_assert!(shift_bytes < self.data.len());
            unsafe {
                std::ptr::copy(
                    self.data.as_mut_ptr(),
                    self.data[shift_bytes..].as_mut_ptr(),
                    (SIZE - shift_bytes) * 8,
                );
            }
            self.data[..shift_bytes].iter_mut().for_each(|v| *v = 0);
            shift -= shift_bytes * 8;
        }

        debug_assert!(shift < 8);
        if shift > 0 {
            let mut carry = 0_u8;
            for i in 0..SIZE {
                let last_carry = carry;
                carry = (self.data[i] & 0xff >> (8 - shift)) << (8 - shift);
                self.data[i] >>= shift;
                self.data[i] |= last_carry;
            }
        }
    }
}

impl std::ops::Not for Sha1Hash {
    type Output = Sha1Hash;

    fn not(mut self) -> Sha1Hash {
        self.data.iter_mut().for_each(|v| *v = !*v);
        self
    }
}

impl std::ops::Index<usize> for Sha1Hash {
    type Output = u8;

    fn index(&self, index: usize) -> &u8 {
        &self.data[index]
    }
}

impl std::ops::IndexMut<usize> for Sha1Hash {
    fn index_mut(&mut self, index: usize) -> &mut u8 {
        &mut self.data[index]
    }
}

macro_rules! impl_binary_op {
    ($op: ident, $op_fn: ident, $op_assign: ident, $op_assign_fn: ident, $sign: tt, $sign_assign: tt) => {
        impl std::ops::$op_assign<&Self> for Sha1Hash {
            fn $op_assign_fn(&mut self, other: &Self) {
                self.data
                    .iter_mut()
                    .zip(other.data.iter())
                    .for_each(|(a, b)| *a $sign_assign b);
            }
        }

        impl std::ops::$op_assign for Sha1Hash {
            fn $op_assign_fn(&mut self, other: Self) {
                *self $sign_assign &other;
            }
        }

        impl std::ops::$op for Sha1Hash {
            type Output = Self;

            fn $op_fn(mut self, other: Self) -> Self {
                self $sign_assign other;
                self
            }
        }

        impl std::ops::$op<&Self> for Sha1Hash {
            type Output = Self;

            fn $op_fn(mut self, other: &Self) -> Self {
                self $sign_assign other;
                self
            }
        }

        impl std::ops::$op for &Sha1Hash {
            type Output = Sha1Hash;

            fn $op_fn(self, other: Self) -> Sha1Hash {
                let mut s = Sha1Hash::new();
                s.data
                    .iter_mut()
                    .zip(self.data.iter())
                    .zip(other.data.iter())
                    .for_each(|((a, b), c)| *a = b $sign c);
                s
            }
        }

        impl std::ops::$op<Sha1Hash> for &Sha1Hash {
            type Output = Sha1Hash;

            fn $op_fn(self, other: Sha1Hash) -> Sha1Hash {
                self $sign &other
            }
        }
    };
}

impl_binary_op!(BitXor, bitxor, BitXorAssign, bitxor_assign, ^, ^=);
impl_binary_op!(BitAnd, bitand, BitAndAssign, bitand_assign, &, &=);
impl_binary_op!(BitOr, bitor, BitOrAssign, bitor_assign, |, |=);

#[cfg(test)]
mod test {
    use super::Sha1Hash;

    #[test]
    fn shift_left_01() {
        let mut s = Sha1Hash::max();
        s <<= 5;
        assert_eq!(
            &[
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                0xff, 0xff, 0xff, 0xff, 0xff, 0xe0
            ],
            s.data()
        )
    }

    #[test]
    fn shift_left_02() {
        let mut s = Sha1Hash::max();
        s <<= 16;
        assert_eq!(
            &[
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                0xff, 0xff, 0xff, 0xff, 0x00, 0x00
            ],
            s.data()
        )
    }

    #[test]
    fn shift_left_03() {
        let mut s = Sha1Hash::max();
        s <<= 17;
        assert_eq!(
            &[
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                0xff, 0xff, 0xff, 0xfe, 0x00, 0x00
            ],
            s.data()
        )
    }

    #[test]
    fn shift_left_04() {
        let mut s = Sha1Hash::max();
        s <<= 160;
        assert_eq!(&[0; 20], s.data())
    }

    #[test]
    fn shift_left_05() {
        let mut s = Sha1Hash::max();
        s <<= 20000;
        assert_eq!(&[0; 20], s.data())
    }
    #[test]
    fn shift_left_06() {
        let mut s = Sha1Hash::max();
        s <<= 0;
        assert_eq!(&[0xff; 20], s.data())
    }

    #[test]
    fn shift_right_01() {
        let mut s = Sha1Hash::max();
        s >>= 5;
        assert_eq!(
            &[
                0x07, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff
            ],
            s.data()
        )
    }

    #[test]
    fn shift_right_02() {
        let mut s = Sha1Hash::max();
        s >>= 16;
        assert_eq!(
            &[
                0x00, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff
            ],
            s.data()
        )
    }

    #[test]
    fn shift_right_03() {
        let mut s = Sha1Hash::max();
        s >>= 17;
        assert_eq!(
            &[
                0x00, 0x00, 0x7f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            ],
            s.data()
        )
    }

    #[test]
    fn shift_right_04() {
        let mut s = Sha1Hash::max();
        s >>= 160;
        assert_eq!(&[0x00; 20], s.data())
    }

    #[test]
    fn shift_right_05() {
        let mut s = Sha1Hash::max();
        s >>= 20000;
        assert_eq!(&[0x00; 20], s.data())
    }

    #[test]
    fn shift_right_06() {
        let mut s = Sha1Hash::max();
        s >>= 0;
        assert_eq!(&[0xff; 20], s.data())
    }

    #[test]
    fn count_leading_zeros_01() {
        assert_eq!(0, Sha1Hash::max().leading_zeros());
        assert_eq!(160, Sha1Hash::min().leading_zeros());
    }

    #[test]
    fn count_leading_zeros_02() {
        let mut s = Sha1Hash::from([0x01; 20]);
        assert_eq!(7, s.leading_zeros());
        s.data_mut()[0] = 0;
        assert_eq!(15, s.leading_zeros());
        s.data_mut()[1] = 0;
        assert_eq!(23, s.leading_zeros());
    }

    #[test]
    fn not() {
        assert_eq!(Sha1Hash::min(), !Sha1Hash::max());
        assert_eq!(Sha1Hash::from([0xfe; 20]), !Sha1Hash::from([0x01; 20]));
    }

    #[test]
    fn xor() {
        assert_eq!(Sha1Hash::min(), Sha1Hash::max() ^ Sha1Hash::max());
        assert_eq!(Sha1Hash::max(), Sha1Hash::min() ^ Sha1Hash::max());
        assert_eq!(Sha1Hash::max(), Sha1Hash::max() ^ Sha1Hash::min());
        assert_eq!(Sha1Hash::min(), Sha1Hash::min() ^ Sha1Hash::min());
        assert_eq!(
            Sha1Hash::from([0x01; 20]),
            Sha1Hash::from([0x01; 20]) ^ Sha1Hash::min()
        );
        assert_eq!(
            Sha1Hash::from([0xfe; 20]),
            Sha1Hash::from([0x01; 20]) ^ Sha1Hash::max()
        );
        assert_eq!(
            Sha1Hash::max(),
            Sha1Hash::from([0x01; 20]) ^ Sha1Hash::from([0xfe; 20])
        );
    }

    #[test]
    fn and() {
        assert_eq!(Sha1Hash::min(), Sha1Hash::min() & Sha1Hash::max());
        assert_eq!(
            Sha1Hash::from([0x01; 20]),
            Sha1Hash::from([0x01; 20]) & Sha1Hash::max()
        );
        assert_eq!(
            Sha1Hash::min(),
            Sha1Hash::from([0x01; 20]) & Sha1Hash::from([0xfe; 20])
        );
    }

    #[test]
    fn or() {
        assert_eq!(Sha1Hash::max(), Sha1Hash::min() | Sha1Hash::max());
        assert_eq!(
            Sha1Hash::max(),
            Sha1Hash::from([0x01; 20]) | Sha1Hash::max()
        );
        assert_eq!(
            Sha1Hash::from([0x81; 20]),
            Sha1Hash::from([0x01; 20]) | Sha1Hash::from([0x80; 20])
        );
    }

    #[test]
    fn index() {
        assert_eq!(0, Sha1Hash::min()[0]);
        assert_eq!(0xff, Sha1Hash::max()[0]);
        let mut s = Sha1Hash::new();
        s[0] = 0xff;
        let mut data = [0x00; 20];
        data[0] = 0xff;
        assert_eq!(Sha1Hash::from(data), s);
    }
}
