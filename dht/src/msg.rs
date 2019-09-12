use std::net::SocketAddr;

use bencode::ValueRef;
use bitflags::bitflags;

pub struct Msg<'a> {
    message: &'a ValueRef<'a>,
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

pub fn verify_message_impl<'a, 'b>(
    msg: &'b ValueRef<'a>,
    desc: &[KeyDesc],
    ret: &mut [&'b ValueRef<'a>],
) -> Result<(), &'static str> {
    debug_assert_eq!(desc.len(), ret.len());

    if !msg.is_dict() {
        return Err("Not a dictionary");
    }

    let stack = [msg; 5];
    let size = ret.len();

    for i in 0..size {
        let k = &desc[i];
        ret[i] = msg.dict_find(k.name).ok_or_else(|| "Key not found")?;
    }
    Ok(())
}
