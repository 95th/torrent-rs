use crate::error::{Error, Result};
use std::io::{Cursor, Write};

pub fn split_string(s: &[u8], separator: u8) -> (&[u8], &[u8]) {
    if s.is_empty() {
        return (s, s);
    }

    let mut pos = 0;
    if s[0] == b'"' && separator != b'"' {
        for &c in &s[1..] {
            pos += 1;
            if c == b'"' {
                break;
            }
        }
    }

    let mut found_sep = 0;
    for &c in &s[pos..] {
        if c == separator {
            found_sep = 1;
            break;
        }
        pos += 1;
    }

    (&s[..pos], &s[pos + found_sep..])
}

pub fn unescape_string(s: &[u8]) -> Result<String> {
    let v = unescape_bytes(s)?;
    String::from_utf8(v).map_err(|_| Error::InvalidEscapedString)
}

pub fn unescape_bytes<'a>(s: &'a [u8]) -> Result<Vec<u8>> {
    let mut v = vec![];
    let mut i = 0;
    while i < s.len() {
        let c = s[i];
        if c == b'+' {
            v.push(b' ');
        } else if c != b'%' {
            v.push(c);
        } else {
            i += 1;
            if i == s.len() {
                return Err(Error::InvalidEscapedString);
            }
            let hi = match s[i] {
                c @ b'0'..=b'9' => c - b'0',
                c @ b'A'..=b'F' => c + 10 - b'A',
                c @ b'a'..=b'f' => c + 10 - b'a',
                _ => return Err(Error::InvalidEscapedString),
            };

            i += 1;
            if i == s.len() {
                return Err(Error::InvalidEscapedString);
            }

            let lo = match s[i] {
                c @ b'0'..=b'9' => c - b'0',
                c @ b'A'..=b'F' => c + 10 - b'A',
                c @ b'a'..=b'f' => c + 10 - b'a',
                _ => return Err(Error::InvalidEscapedString),
            };

            v.push(hi * 16 + lo);
        }
        i += 1;
    }
    Ok(v)
}

pub fn to_upper(c: u8) -> u8 {
    match c {
        b'a'..=b'z' => c - b'a' + b'A',
        c => c,
    }
}

const INPUT_OUTPUT_MAPPING: [usize; 9] = [5, 1, 1, 2, 2, 3, 4, 4, 5];

pub fn base32_decode(s: &[u8]) -> Vec<u8> {
    let mut in_buf = [0u8; 8];
    let mut out_buf = [0u8; 5];

    let mut v = vec![];
    let mut c = Cursor::new(&mut v);
    let mut i = 0;
    while i < s.len() {
        let available = in_buf.len().min(s.len() - i);
        let mut pad_start = 0;
        if available < 8 {
            pad_start = available;
        }

        in_buf.iter_mut().for_each(|c| *c = 0);
        for j in 0..available {
            let c = to_upper(s[i]);
            i += 1;
            in_buf[j] = match c {
                b'A'..=b'Z' => c - b'A',
                b'2'..=b'7' => c - b'2' + (b'Z' - b'A') + 1,
                b'=' => {
                    if pad_start == 0 {
                        pad_start = j;
                    }
                    0
                }
                b'1' => b'I' - b'A',
                _ => return vec![],
            };
            debug_assert_eq!(in_buf[j], in_buf[j] & 0x1f);
        }

        out_buf[0] = in_buf[0] << 3;
        out_buf[0] |= in_buf[1] >> 2;
        out_buf[1] = (in_buf[1] & 0x3) << 6;
        out_buf[1] |= in_buf[2] << 1;
        out_buf[1] |= (in_buf[3] & 0x10) >> 4;
        out_buf[2] = (in_buf[3] & 0x0f) << 4;
        out_buf[2] |= (in_buf[4] & 0x1e) >> 1;
        out_buf[3] = (in_buf[4] & 0x01) << 7;
        out_buf[3] |= (in_buf[5] & 0x1f) << 2;
        out_buf[3] |= (in_buf[6] & 0x18) >> 3;
        out_buf[4] = (in_buf[6] & 0x07) << 5;
        out_buf[4] |= in_buf[7];

        let num_out = INPUT_OUTPUT_MAPPING[pad_start];
        c.write_all(&out_buf[..num_out]).unwrap();
    }
    v
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! assert_tuple_eq {
        ($a: expr, $b: expr) => {
            let a: (&[u8], &[u8]) = $a;
            let b: (&[u8], &[u8]) = $b;
            assert_eq!(a, b);
        };
    }

    #[test]
    fn test_split_string() {
        assert_tuple_eq!(split_string(b"a b", b' '), (b"a", b"b"));
        assert_tuple_eq!(split_string(b"\"a b\" c", b' '), (b"\"a b\"", b"c"));
        assert_tuple_eq!(
            split_string(b"\"a b\"foobar c", b' '),
            (b"\"a b\"foobar", b"c")
        );
        assert_tuple_eq!(split_string(b"a\nb foobar", b' '), (b"a\nb", b"foobar"));
        assert_tuple_eq!(split_string(b"a b\"foo\"bar", b'"'), (b"a b", b"foo\"bar"));
        assert_tuple_eq!(split_string(b"a", b' '), (b"a", b""));
        assert_tuple_eq!(split_string(b"\"a b", b' '), (b"\"a b", b""));
        assert_tuple_eq!(split_string(b"", b' '), (b"", b""));
    }

    #[test]
    fn test_unescape_string() {
        let test_string = "!%40%23%24%25%5e%26*()-_%3d%2b%2f%2c.%20%25%3f";
        let expected = "!@#$%^&*()-_=+/,. %?";
        let actual = unescape_string(test_string.as_bytes()).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_base32_decode() {
        assert_eq!(base32_decode(b""), b"");
        assert_eq!(base32_decode(b"MY======"), b"f");
        assert_eq!(base32_decode(b"MZXQ===="), b"fo");
        assert_eq!(base32_decode(b"MZXW6==="), b"foo");
        assert_eq!(base32_decode(b"MZXW6YQ="), b"foob");
        assert_eq!(base32_decode(b"MZXW6YTB"), b"fooba");
        assert_eq!(base32_decode(b"MZXW6YTBOI======"), b"foobar");

        assert_eq!(base32_decode(b"MY"), b"f");
        assert_eq!(base32_decode(b"MZXW6YQ"), b"foob");
        assert_eq!(base32_decode(b"MZXW6YTBOI"), b"foobar");
        assert_eq!(base32_decode(b"mZXw6yTBO1======"), b"foobar");

        // make sure invalid encoding returns the empty string
        assert_eq!(base32_decode(b"mZXw6yTBO1{#&*()="), b"");
    }
}
