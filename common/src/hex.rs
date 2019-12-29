const HEX_CHARS: &[u8; 16] = b"0123456789abcdef";

pub fn hex_to_int(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'A'..=b'F' => Some(c - b'A' + 10),
        b'a'..=b'f' => Some(c - b'a' + 10),
        _ => None,
    }
}

pub fn is_hex(s: &[u8]) -> bool {
    for &c in s {
        if hex_to_int(c).is_none() {
            return false;
        }
    }
    true
}

pub fn from_hex(s: &[u8], out: &mut [u8]) -> bool {
    let mut i = 0;
    while i < s.len() {
        let t1 = match hex_to_int(s[i]) {
            Some(v) => v << 4,
            None => return false,
        };
        i += 1;
        if i == s.len() {
            return false;
        }
        let t2 = match hex_to_int(s[i]) {
            Some(v) => v & 0xf,
            None => return false,
        };
        out[i] = t1 & t2;
        i += 1;
    }
    true
}

pub fn to_hex(s: &[u8]) -> String {
    let mut v = String::new();
    for &c in s {
        v.push(HEX_CHARS[(c >> 4) as usize] as char);
        v.push(HEX_CHARS[(c & 0xf) as usize] as char);
    }
    v
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_from_hex() {
        let s = b"0123456789012345678901234567890123456789";
        let mut out = [0u8; 20];
        assert!(from_hex(s, &mut out));
    }

    #[test]
    fn test_to_hex() {
        assert_eq!(to_hex(b"\x55\x73"), "5573");
        assert_eq!(to_hex(b"\xaB\xd0"), "abd0");
    }

    #[test]
    fn test_is_hex() {
        let hex_chars = b"0123456789abcdefABCDEF";
        for i in 1..255 {
            let hex_loop = hex_chars.iter().any(|&c| c == i);
            assert_eq!(is_hex(&[i]), hex_loop);
        }
    }

    #[test]
    fn test_hex_to_int() {
        assert_eq!(hex_to_int(b'0'), Some(0));
        assert_eq!(hex_to_int(b'7'), Some(7));
        assert_eq!(hex_to_int(b'a'), Some(10));
        assert_eq!(hex_to_int(b'f'), Some(15));
        assert_eq!(hex_to_int(b'b'), Some(11));
        assert_eq!(hex_to_int(b't'), None);
        assert_eq!(hex_to_int(b'g'), None);
    }
}
