use std::net::IpAddr;
use std::time::Instant;

use bitflags::bitflags;

use crate::node::NodeId;

pub struct Observer {
    id: NodeId,
    addr: IpAddr,
    port: u16,
    sent: Instant,
    flags: ObserverFlags,
}

bitflags! {
    struct ObserverFlags: u8 {
        const QUERIED = 1;
        const INITIAL = 2;
        const NO_ID = 4;
        const SHORT_TIMEOUT = 8;
        const FAILED = 16;
        const IPV6_ADDRESS = 32;
        const ALIVE = 64;
        const DONE = 128;
    }
}
