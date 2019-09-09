use bencode::Value;
use std::collections::BTreeMap;

macro_rules! assert_bytes_eq {
    ($expected: expr, $actual: expr) => {
        assert_eq!($expected.len(), $actual.len());
        assert!(!$expected.iter().zip($actual.iter()).any(|(a, b)| a != b));
    };
}

#[test]
fn simple_test() {
    let value: Value = "d1:ad1:bi1e1:c4:abcde1:di3ee".parse().unwrap();
    let map = value.as_dict().unwrap();
    let a = &map["a"];
    let sub_map = a.as_dict().unwrap();
    assert!(sub_map["b"].is_int());
    assert!(sub_map["c"].is_string());
    assert!(map["d"].is_int());
}

#[test]
fn encode_str() {
    let s: Value = Value::with_str("Hello world");
    let mut w = vec![];
    s.encode(&mut w).unwrap();
    assert_bytes_eq!(b"11:Hello world", w);
}

#[test]
fn encode_i64() {
    let s: Value = Value::with_int(100);
    let mut w = vec![];
    s.encode(&mut w).unwrap();
    assert_bytes_eq!(b"i100e", w);
}

#[test]
fn encode_list() {
    let v = Value::with_list(vec![
        Value::with_int(100),
        Value::with_str("hello"),
        Value::with_str("world"),
    ]);
    assert_eq!("li100e5:hello5:worlde", v.to_string());
}

#[test]
fn encode_dict() {
    let mut m = BTreeMap::new();
    m.insert(String::from("hello"), Value::with_str("world"));
    let v = Value::with_dict(m);
    assert_eq!("d5:hello5:worlde", v.to_string());
}

#[test]
fn decode_str() {
    let v: Value = "10:helloworld".parse().unwrap();
    let s = v.as_str_bytes().unwrap();
    assert_eq!(b"helloworld", s);
}

#[test]
fn decode_i64() {
    let v: Value = "i100e".parse().unwrap();
    let s: i64 = v.as_int().unwrap();
    assert_eq!(100, s);
}

#[test]
fn decode_list() {
    let v: Value = "li100e10:helloworldli100e2:24ee".parse().unwrap();

    let list = v.as_list().unwrap();
    assert_eq!(100, list[0].as_int().unwrap());
    assert_eq!(b"helloworld", list[1].as_str_bytes().unwrap());

    let sublist = list[2].as_list().unwrap();
    assert_eq!(100, sublist[0].as_int().unwrap());
    assert_eq!(b"24", sublist[1].as_str_bytes().unwrap());
}

#[test]
fn decode_dict() {
    let v: Value = "d5:hello5:worlde".parse().unwrap();
    let map = v.as_dict().unwrap();
    assert_eq!(1, map.len());
    assert_eq!(b"world", map["hello"].as_str_bytes().unwrap());
}

#[test]
fn decode_dict_2() {
    let v: Value = "d3:cow3:moo4:spam4:eggse".parse().unwrap();
    let map = v.as_dict().unwrap();
    assert_eq!(2, map.len());
    assert_eq!(b"moo", map["cow"].as_str_bytes().unwrap());
    assert_eq!(b"eggs", map["spam"].as_str_bytes().unwrap());
}

#[test]
fn borrow() {
    let v: Value = "d3:cow3:moo4:spam4:eggse".parse().unwrap();
    assert_eq!("d3:cow3:moo4:spam4:eggse", v.to_string());
    let v = v.as_ref();
    assert_eq!("d3:cow3:moo4:spam4:eggse", v.to_string());
}

#[test]
fn decode_str_02() {
    let v: Value = "26:abcdefghijklmnopqrstuvwxyz".parse().unwrap();
    let s = v.as_str().unwrap();
    assert_eq!("abcdefghijklmnopqrstuvwxyz", s);
}

#[test]
fn decode_large_str() {
    let s = String::from("1000000:") + &"x".repeat(1_000_000);
    let v: Value = s.parse().unwrap();
    assert_eq!(&s[8..], v.as_str().unwrap());
}

#[test]
fn decode_list_02() {
    let v: Value = "li12345e3:aaae".parse().unwrap();
    let list = v.as_list().unwrap();
    assert_eq!(2, list.len());

    assert_eq!(12345, list[0].as_int().unwrap());
    assert_eq!("i12345e", list[0].to_string());

    assert_eq!("aaa", list[1].as_str().unwrap());
    assert_eq!("3:aaa", list[1].to_string());
}

#[test]
fn decode_dict_02() {
    let v: Value = "d1:ai12453e1:b3:aaa1:c3:bbb1:X10:0123456789e"
        .parse()
        .unwrap();
    let dict = v.as_dict().unwrap();
    assert_eq!(4, dict.len());

    assert_eq!("i12453e", dict["a"].to_string());
    assert_eq!(12453, dict["a"].as_int().unwrap());

    assert_eq!("3:aaa", dict["b"].to_string());
    assert_eq!("aaa", dict["b"].as_str().unwrap());

    assert_eq!("3:bbb", dict["c"].to_string());
    assert_eq!("bbb", dict["c"].as_str().unwrap());

    assert_eq!("10:0123456789", dict["X"].to_string());
    assert_eq!("0123456789", dict["X"].as_str().unwrap());
}

#[test]
fn decode_dict_key_novalue() {
    let e = "d1:ai1e1:be".parse::<Value>().unwrap_err();
    assert_eq!(bencode::Error::ParseDict, e);
}

#[test]
fn decode_dict_non_str_key() {
    let e = "di5e1:ae".parse::<Value>().unwrap_err();
    assert_eq!(bencode::Error::ParseDict, e);
}

#[test]
fn decode_dict_null_key() {
    let v: Value = "d3:a\0bi1ee".parse().unwrap();
    let dict = v.as_dict().unwrap();
    assert_eq!(1, dict.len());

    assert_eq!(1, dict["a\0b"].as_int().unwrap());
}

#[test]
fn decode_dict_non_sorted_key_01() {
    let v: Value = "d2:abi1e2:aai2ee".parse().unwrap();
    let dict = v.as_dict().unwrap();
    assert_eq!(2, dict.len());

    assert_eq!(1, dict["ab"].as_int().unwrap());
    assert_eq!(2, dict["aa"].as_int().unwrap());
}

#[test]
fn decode_64_bit_int() {
    let v: Value = "i9223372036854775807e".parse().unwrap();
    assert_eq!(9223372036854775807, v.as_int().unwrap());
}

#[test]
fn decode_64_bit_int_negative() {
    let v: Value = "i-9223372036854775807e".parse().unwrap();
    assert_eq!(-9223372036854775807, v.as_int().unwrap());
}

#[test]
fn decode_int_invalid_digit() {
    let e = "i92337203t854775807e".parse::<Value>().unwrap_err();
    assert_eq!(bencode::Error::ParseInt, e);
}

#[test]
fn decode_invalid_encoding() {
    let buf = [
        0x64_u8, 0x31, 0x3a, 0x61, 0x64, 0x32, 0x3a, 0x69, 0x64, 0x32, 0x30, 0x3a, 0x2a, 0x21,
        0x19, 0x89, 0x9f, 0xcd, 0x5f, 0xc9, 0xbc, 0x80, 0xc1, 0x76, 0xfe, 0xe0, 0xc6, 0x84, 0x2d,
        0xf6, 0xfc, 0xb8, 0x39, 0x3a, 0x69, 0x6e, 0x66, 0x6f, 0x5f, 0x68, 0x61, 0xae, 0x68, 0x32,
        0x30, 0x3a, 0x14, 0x78, 0xd5, 0xb0, 0xdc, 0xf6, 0x82, 0x42, 0x32, 0xa0, 0xd6, 0x88, 0xeb,
        0x48, 0x57, 0x01, 0x89, 0x40, 0x4e, 0xbc, 0x65, 0x31, 0x3a, 0x71, 0x39, 0x3a, 0x67, 0x65,
        0x74, 0x5f, 0x70, 0x65, 0x65, 0x72, 0x78, 0xff, 0x3a, 0x74, 0x38, 0x3a, 0xaa, 0xd4, 0xa1,
        0x88, 0x7a, 0x8d, 0xc3, 0xd6, 0x31, 0x3a, 0x79, 0x31, 0xae, 0x71, 0x65, 0,
    ];

    let e = Value::decode(&buf).unwrap_err();
    assert_eq!(bencode::Error::ParseDict, e);
}

#[test]
fn decode_depth_limit() {
    let mut buf = [0u8; 2048];

    // 1024 level nested lists
    for i in 0..1024 {
        buf[i] = b'l';
    }
    for i in 1024..2048 {
        buf[i] = b'e';
    }

    let e = Value::decode_with_limits(&buf, Some(1000), None).unwrap_err();
    assert_eq!(bencode::Error::DepthLimit, e);

    Value::decode_with_limits(&buf, Some(1024), None).unwrap();
    Value::decode_with_limits(&buf, Some(1025), None).unwrap();
}

#[test]
fn decode_item_limit() {
    let mut buf = [0u8; 1024];

    buf[0] = b'l';
    for i in (1..1023).step_by(2) {
        buf[i..i + 2].copy_from_slice(b"0:");
    }
    buf[1023] = b'e';

    let e = Value::decode_with_limits(&buf, None, Some(510)).unwrap_err();
    assert_eq!(bencode::Error::ItemLimit, e);

    Value::decode_with_limits(&buf, None, Some(511)).unwrap();
    Value::decode_with_limits(&buf, None, Some(512)).unwrap();
}

#[test]
fn decode_expected_colon_dict() {
    let e = "d1000".parse::<Value>().unwrap_err();
    assert_eq!(bencode::Error::ExpectedChar(b':'), e);
}

#[test]
fn decode_empty_string() {
    let e = "".parse::<Value>().unwrap_err();
    assert_eq!(bencode::Error::EOF, e);
}

#[test]
fn decode_partial_string() {
    let e = "100:..".parse::<Value>().unwrap_err();
    assert_eq!(bencode::Error::EOF, e);
}

#[test]
fn decode_list_of_ints() {
    let mut buf = vec![];
    buf.push(b'l');
    for i in 0..1000 {
        let s = format!("i{}e", i);
        buf.append(&mut s.into_bytes());
    }
    buf.push(b'e');

    let v: Value = Value::decode(&buf).unwrap();
    let list = v.as_list().unwrap();
    assert_eq!(1000, list.len());
    for i in 0..1000 {
        assert_eq!(i as i64, list[i].as_int().unwrap());
    }
}

#[test]
fn decode_dict_of_ints() {
    let mut buf = vec![];
    buf.push(b'd');
    for i in 0..1000 {
        let s = format!("4:{:04}i{}e", i, i);
        buf.append(&mut s.into_bytes());
    }
    buf.push(b'e');

    let v: Value = Value::decode(&buf).unwrap();
    let dict = v.as_dict().unwrap();
    assert_eq!(1000, dict.len());
    for i in 0..1000 {
        let key = format!("{:04}", i);
        assert_eq!(i as i64, dict[&key[..]].as_int().unwrap());
    }
}

#[test]
fn decode_parse_int_overflow() {
    let e = "i9223372036854775808e".parse::<Value>().unwrap_err();
    assert_eq!(bencode::Error::ParseInt, e);
}

#[test]
fn decode_parse_length_overflow() {
    let arr = [
        "d1:a1919191010:11111",
        "d2143289344:a4:aaaae",
        "d214328934114:a4:aaaae",
        "d9205357638345293824:a4:aaaae",
        "d1:a9205357638345293824:11111",
    ];
    for s in arr.iter() {
        let e = s.parse::<Value>().unwrap_err();
        assert_eq!(bencode::Error::EOF, e);
    }
}

#[test]
fn decode_dict_find_funs() {
    // a: int
    // b: string
    // c: list
    // d: dict
    let v: Value = "d1:ai1e1:b3:foo1:cli1ei2ee1:dd1:xi1eee".parse().unwrap();
    assert!(v.is_dict());

    assert_eq!(Some(1), v.dict_find_int_value("a"));
    assert_eq!(None, v.dict_find_int("b"));
    assert_eq!(None, v.dict_find_int_value("b"));
    assert_eq!(None, v.dict_find_int("x"));
    assert_eq!(None, v.dict_find_int_value("x"));

    assert_eq!(Some("foo"), v.dict_find_str_value("b"));
    assert_eq!(None, v.dict_find_str("c"));
    assert_eq!(None, v.dict_find_str_value("c"));
    assert_eq!(None, v.dict_find_str("x"));
    assert_eq!(None, v.dict_find_str_value("x"));

    let c = v.dict_find_list("c").unwrap();
    assert_eq!(Some(2), c.list_len());
    assert_eq!(Some(1), c.list_int_value_at(0));
    assert_eq!(Some(2), c.list_int_value_at(1));
    assert!(v.dict_find_dict("c").is_none());

    let d = v.dict_find_dict("d").unwrap();
    assert_eq!(Some(1), d.dict_find_int_value("x"));
    assert_eq!(None, d.dict_find_int_value("y"));
    assert!(v.dict_find_dict("c").is_none());

    assert_eq!(Some(4), v.dict_len());
}
