use bencode::Value;
use std::collections::BTreeMap;

#[test]
fn encode_dict() {
    let mut m = BTreeMap::new();
    m.insert(String::from("hello"), Value::with_str("world"));
    let v = Value::with_dict(m);
    assert_eq!("d5:hello5:worlde", v.to_string());
}

#[test]
fn borrow() {
    let v: Value = "d3:cow3:moo4:spam4:eggse".parse().unwrap();
    assert_eq!("d3:cow3:moo4:spam4:eggse", v.to_string());
    let v = v.as_ref();
    assert_eq!("d3:cow3:moo4:spam4:eggse", v.to_string());
}
