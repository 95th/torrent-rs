use bencode::Value;
use std::collections::BTreeMap;

macro_rules! assert_bytes_eq {
    ($expected: expr, $actual: expr) => {
        assert_eq!($expected.len(), $actual.len());
        assert!(!$expected.iter().zip($actual.iter()).any(|(a, b)| a != b));
    };
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
    let v = Value::with_map(m);
    assert_eq!("d5:hello5:worlde", v.to_string());
}

#[test]
fn decode_str() {
    let v: Value = "10:helloworld".parse().unwrap();
    let s = v.as_str().unwrap();
    assert_eq!("helloworld", s);
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
    assert_eq!("helloworld", list[1].as_str().unwrap());

    let sublist = list[2].as_list().unwrap();
    assert_eq!(100, sublist[0].as_int().unwrap());
    assert_eq!("24", sublist[1].as_str().unwrap());
}

#[test]
fn decode_dict() {
    let v: Value = "d5:hello5:worlde".parse().unwrap();
    let map = v.as_map().unwrap();
    assert_eq!(1, map.len());
    assert_eq!("world", map["hello"].as_str().unwrap());
}

#[test]
fn decode_dict_2() {
    let v: Value = "d3:cow3:moo4:spam4:eggse".parse().unwrap();
    let map = v.as_map().unwrap();
    assert_eq!(2, map.len());
    assert_eq!("moo", map["cow"].as_str().unwrap());
    assert_eq!("eggs", map["spam"].as_str().unwrap());
}
