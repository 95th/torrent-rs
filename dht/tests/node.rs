use std::net::IpAddr;
use std::net::SocketAddr;
use std::rc::Rc;

use dht::id::Id;
use dht::node::Node;

#[test]
fn socket_addr() {
    let addr: IpAddr = "127.0.0.1".parse().unwrap();
    let port = 1234;
    let node = Node::new(Rc::new(Id::new()), addr, port);
    let expected: SocketAddr = "127.0.0.1:1234".parse().unwrap();
    assert_eq!(Some(expected), node.socket_addr());
}
