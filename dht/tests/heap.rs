use std::net::IpAddr;
use std::rc::Rc;

use dht::id::Id;
use dht::node::{Node, NodeHeap};

fn build_heap(id: Rc<Id>, addr: IpAddr, max_size: usize) -> NodeHeap {
    let mut nodes = vec![];
    for i in 1..=10 {
        nodes.push(Rc::new(Node::new(id.at_dist(i), addr, 1200 + i as u16)));
    }

    let node = Rc::new(Node::new(id, addr, 1200));
    let mut heap = NodeHeap::new(node, max_size);
    heap.push_all(&nodes);
    heap
}

#[test]
fn ids() {
    let id = Rc::new(Id::from([0; 20]));
    let addr = "127.0.0.1".parse().unwrap();
    let expected = vec![id.at_dist(1), id.at_dist(2), id.at_dist(3), id.at_dist(4)];
    let heap = build_heap(id, addr, 4);
    assert_eq!(expected, heap.get_ids());
}

#[test]
fn pop() {
    let id = Rc::new(Id::from([0; 20]));
    let addr = "127.0.0.1".parse().unwrap();
    let expected = Some(Rc::new(Node::new(id.at_dist(1), addr, 1201)));
    let mut heap = build_heap(id, addr, 4);
    assert_eq!(expected, heap.pop());
}
