use std::net::{IpAddr, SocketAddr};
use std::time::Instant;

use bitflags::bitflags;

use crate::dht_observer::DhtObserver;
use crate::msg::Msg;
use crate::node::NodeId;
use crate::traversal_algorithm::TraversalAlgorithm;

pub trait Observer {
    fn reply(&mut self, msg: &Msg);

    fn short_timeout(&mut self);

    fn flags(&self) -> ObserverFlags;

    fn has_short_timeout(&self) -> bool {
        self.flags().contains(ObserverFlags::SHORT_TIMEOUT)
    }

    fn timeout(&mut self);

    fn abort(&mut self);

    fn get_observer(&self) -> &DhtObserver;

    fn algorithm(&self) -> &dyn TraversalAlgorithm;

    fn send(&self) -> Instant;

    fn set_target(&mut self, addr: &SocketAddr);

    fn target_addr(&self) -> IpAddr;

    fn target_endpoint(&self) -> SocketAddr;

    fn set_id(&mut self, id: &NodeId);

    fn id(&self) -> &NodeId;
}

bitflags! {
    pub struct ObserverFlags: u8 {
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
