use std::net::IpAddr;

pub type NodeId = Vec<u8>;
pub type NodeIds = Vec<(IpAddr, NodeId)>;
