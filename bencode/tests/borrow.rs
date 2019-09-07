use bencode::BorrowValue as Value;
use std::collections::BTreeMap;

macro_rules! assert_bytes_eq {
    ($expected: expr, $actual: expr) => {
        assert_eq!($expected.len(), $actual.len());
        assert!(!$expected.iter().zip($actual.iter()).any(|(a, b)| a != b));
    };
}

#[test]
fn simple_test() {
    let value = Value::decode(b"d1:ad1:bi1e1:c4:abcde1:di3ee").unwrap();
    match value {
        Value::Dict(map) => {
            let a = &map["a"];
            match a {
                Value::Dict(sub_map) => {
                    assert!(sub_map["b"].is_int());
                    assert!(sub_map["c"].is_string());
                }
                _ => panic!("Expected Dict"),
            }
            assert!(map["d"].is_int());
        }
        _ => panic!("Expected Dict"),
    }
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
    m.insert("hello", Value::with_str("world"));
    let v = Value::with_dict(m);
    assert_eq!("d5:hello5:worlde", v.to_string());
}

#[test]
fn decode_str() {
    let v = Value::decode(b"10:helloworld").unwrap();
    let s = v.as_str_bytes().unwrap();
    assert_eq!(b"helloworld", s);
}

#[test]
fn decode_i64() {
    let v = Value::decode(b"i100e").unwrap();
    let s: i64 = v.as_int().unwrap();
    assert_eq!(100, s);
}

#[test]
fn decode_list() {
    let v = Value::decode(b"li100e10:helloworldli100e2:24ee").unwrap();

    let list = v.as_list().unwrap();
    assert_eq!(100, list[0].as_int().unwrap());
    assert_eq!(b"helloworld", list[1].as_str_bytes().unwrap());

    let sublist = list[2].as_list().unwrap();
    assert_eq!(100, sublist[0].as_int().unwrap());
    assert_eq!(b"24", sublist[1].as_str_bytes().unwrap());
}

#[test]
fn decode_dict() {
    let v = Value::decode(b"d5:hello5:worlde").unwrap();
    let map = v.as_dict().unwrap();
    assert_eq!(1, map.len());
    assert_eq!(b"world", map["hello"].as_str_bytes().unwrap());
}

#[test]
fn decode_dict_2() {
    let v = Value::decode(b"d3:cow3:moo4:spam4:eggse").unwrap();
    let map = v.as_dict().unwrap();
    assert_eq!(2, map.len());
    assert_eq!(b"moo", map["cow"].as_str_bytes().unwrap());
    assert_eq!(b"eggs", map["spam"].as_str_bytes().unwrap());
}

#[test]
fn to_owned() {
    let v = Value::decode(b"d3:cow3:moo4:spam4:eggse").unwrap();
    assert_eq!("d3:cow3:moo4:spam4:eggse", v.to_string());
    let v = v.to_owned();
    assert_eq!("d3:cow3:moo4:spam4:eggse", v.to_string());
}
