use std::net::IpAddr;
use std::net::SocketAddr;

use crate::id::NodeId;

pub struct Node {
    node_id: NodeId,
    addr: IpAddr,
    port: u16,
}

impl Node {
    pub fn new(node_id: NodeId, addr: IpAddr, port: u16) -> Node {
        Node {
            node_id,
            addr,
            port,
        }
    }

    pub fn socket_addr(&self) -> SocketAddr {
        SocketAddr::new(self.addr, self.port)
    }

    pub fn get_id(&self) -> &NodeId {
        &self.node_id
    }
}

#[cfg(test)]
mod test {
    use std::net::IpAddr;
    use std::net::SocketAddr;

    use crate::id::NodeId;

    use super::Node;

    #[test]
    fn socket_addr() {
        let addr: IpAddr = "127.0.0.1".parse().unwrap();
        let port = 1234;
        let node = Node::new(NodeId::new(), addr, port);
        let expected: SocketAddr = "127.0.0.1:1234".parse().unwrap();
        assert_eq!(expected, node.socket_addr());
    }
}