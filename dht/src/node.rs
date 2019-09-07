use std::net::IpAddr;

use crate::Sha1Hash;

pub type NodeId = Sha1Hash;
pub type NodeIds = Vec<(IpAddr, NodeId)>;

pub struct Node;
