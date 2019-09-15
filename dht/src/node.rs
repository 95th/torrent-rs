use std::net::IpAddr;

use common::sha1::Sha1Hash;

pub type NodeId = Sha1Hash;
pub type NodeIds = Vec<(IpAddr, NodeId)>;

pub struct Node;
