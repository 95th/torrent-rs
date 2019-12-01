use std::cmp;
use std::net::IpAddr;

use common::sha1::Sha1Hash;

pub type NodeId = Sha1Hash;
pub type NodeIds = Vec<(IpAddr, NodeId)>;

pub struct Node;

/// returns the distance between the two nodes
/// using the kademlia XOR-metric
fn distance(n1: &NodeId, n2: &NodeId) -> NodeId {
    n1 ^ n2
}

/// returns n in: 2^n <= distance(n1, n2) < 2^(n+1)
/// useful for finding out which bucket a node belongs to
pub fn distance_exp(n1: &NodeId, n2: &NodeId) -> usize {
    // TODO: it's a little bit weird to return 159 - leading zeroes. It should
    // probably be 160 - leading zeroes, but all other code in here is tuned to
    // this expectation now, and it doesn't really matter (other than complexity)
    cmp::max(159 - distance(n1, n2).leading_zeros(), 0)
}

pub fn min_distance_exp(n1: &NodeId, ids: &[NodeId]) -> usize {
    debug_assert!(!ids.is_empty());

    let mut min = 160; // see distance_exp for the why of this constant
    for node_id in ids {
        min = cmp::min(min, distance_exp(n1, node_id));
    }
    min
}
