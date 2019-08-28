use std::net::SocketAddr;

use bencode::Value;
use bitflags::bitflags;

pub struct Msg<'a> {
    message: &'a Value,
    addr: SocketAddr,
}

pub struct KeyDesc<'a> {
    name: &'a str,
    kind: usize,
    size: usize,
    flags: Flags,
}

bitflags! {
    struct Flags: u8 {
        const OPTIONAL = 1;
        const PARSE_CHILDREN = 2;
        const LAST_CHILD = 4;
        const SIZE_DIVISIBLE = 8;
    }
}

pub fn verify_message_impl(msg: &Value, desc: &[KeyDesc]) -> Result<Vec<Value>, String> {
    unimplemented!()
}
