use std::net::SocketAddr;
use std::sync::Arc;

use crate::node::{Node, NodeId};
use crate::node_entry::NodeEntry;
use crate::observer::Observer;

pub struct FindData;

impl FindData {
    pub fn find_data(&self, dht_node: &Node, target: &NodeId) -> Vec<(NodeEntry, String)> {
        unimplemented!()
    }

    pub fn got_write_token(&self, n: &NodeId, write_token: String) {
        unimplemented!();
    }

    pub fn start(&mut self) {
        unimplemented!();
    }

    pub fn name(&self) -> &str {
        unimplemented!();
    }

    fn done(&self) {
        unimplemented!()
    }

    fn new_observer(&self, ep: &SocketAddr, node_id: &NodeId) -> Arc<&dyn Observer> {
        unimplemented!();
    }
}
