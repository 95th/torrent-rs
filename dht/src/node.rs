use std::cmp::Ordering;
use std::collections::binary_heap::BinaryHeap;
use std::collections::HashSet;
use std::mem;
use std::net::IpAddr;
use std::net::SocketAddr;

use crate::id::Id;

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct Node {
    id: Id,
    addr: IpAddr,
    port: u16,
}

impl Node {
    pub fn new(id: Id, addr: IpAddr, port: u16) -> Node {
        Node { id, addr, port }
    }

    pub fn get_id(&self) -> &Id {
        &self.id
    }

    pub fn same_home_as(&self, other: &Node) -> bool {
        self.addr == other.addr && self.port == self.port
    }

    pub fn dist_to(&self, other: &Node) -> usize {
        self.id.dist_to(other.id)
    }

    pub fn socket_addr(&self) -> SocketAddr {
        SocketAddr::new(self.addr, self.port)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
struct NodeHeapItem {
    dist: usize,
    node: Node
}

impl Ord for NodeHeapItem {
    fn cmp(&self, other: &NodeHeapItem) -> Ordering {
        other.dist
             .cmp(&self.dist)
             .then_with(|| self.node.cmp(&other.node))
    }
}

impl PartialOrd for NodeHeapItem {
    fn partial_cmp(&self, other: &NodeHeapItem) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct NodeHeap {
    // TODO: change it to priority queue
    heap: BinaryHeap<NodeHeapItem>,
    node: Node,
    contacted: HashSet<Id>,
    max_size: usize,
}

impl NodeHeap {
    pub fn new(node: Node, max_size: usize) -> NodeHeap {
        NodeHeap {
            node,
            heap: BinaryHeap::new(),
            contacted: HashSet::new(),
            max_size
        }
    }

    pub fn remove_all(&mut self, nodes: &[Node]) {
        let peers: HashSet<&Node> = nodes.iter().collect();
        let old_heap = mem::replace(&mut self.heap, BinaryHeap::new());
        self.heap = old_heap.into_iter()
                            .filter(|item| !peers.contains(&item.node))
                            .collect();
    }

    pub fn get_node(&self, id: Id) -> Option<Node> {
        self.heap
            .iter()
            .find(|item| item.node.id == id)
            .map(|item| item.node)
    }

    pub fn push(&mut self, node: Node) {
        if !self.contains(&node) {
            let dist = self.node.dist_to(&node);
            self.heap.push(NodeHeapItem { dist, node });
        }
    }

    pub fn push_all(&mut self, nodes: &[Node]) {
        for node in nodes {
            if !self.contains(node) {
                let dist = self.node.dist_to(node);
                self.heap.push(NodeHeapItem { dist, node: *node });
            }
        }
    }

    pub fn pop(&mut self) -> Option<Node> {
        self.heap
            .pop()
            .map(|item| item.node)
    }

    pub fn contains(&self, node: &Node) -> bool {
        self.heap
            .iter()
            .find(|item| &item.node == node)
            .is_some()
    }

    pub fn mark_contacted(&mut self, node: &Node) {
        self.contacted.insert(node.id);
    }

    pub fn closest(&self) -> Vec<Node> {
        let mut items = vec![];
        let mut heap = self.heap.clone();
        while let Some(item) = heap.pop() {
            if items.len() < self.max_size {
                items.push(item.node);
            }
        }
        items
    }

    pub fn get_uncontacted(&self) -> Vec<Node> {
        self.closest()
            .into_iter()
            .filter(|item| !self.contacted.contains(&item.id))
            .collect()
    }

    pub fn have_contacted_all(&self) -> bool {
        self.closest()
            .iter()
            .find(|item| !self.contacted.contains(&item.id))
            .is_none()
    }

    pub fn get_ids(&self) -> Vec<Id> {
        self.closest()
            .iter()
            .map(|node| node.id)
            .collect()
    }

    pub fn len(&self) -> usize {
        self.heap.len().min(self.max_size)
    }
}

#[cfg(test)]
mod node_test {
    use std::net::IpAddr;
    use std::net::SocketAddr;

    use crate::id::Id;

    use super::Node;

    #[test]
    fn socket_addr() {
        let addr: IpAddr = "127.0.0.1".parse().unwrap();
        let port = 1234;
        let node = Node::new(Id::new(), addr, port);
        let expected: SocketAddr = "127.0.0.1:1234".parse().unwrap();
        assert_eq!(expected, node.socket_addr());
    }
}

#[cfg(test)]
mod heap_test {
    use std::net::IpAddr;

    use crate::id::Id;

    use super::Node;
    use super::NodeHeap;

    fn build_heap(id: Id, addr: IpAddr, max_size: usize) -> NodeHeap {
        let node = Node::new(id, addr, 1200);
        let mut nodes = vec![];
        for i in 1..=10 {
            nodes.push(Node::new(id.at_dist(i), addr, 1200 + i as u16));
        }

        let mut heap = NodeHeap::new(node, max_size);
        heap.push_all(&nodes);
        heap
    }

    #[test]
    fn ids() {
        let id = [0; 20].into();
        let addr = "127.0.0.1".parse().unwrap();
        let heap = build_heap(id, addr, 4);
        let expected = vec![id.at_dist(1), id.at_dist(2), id.at_dist(3), id.at_dist(4)];
        assert_eq!(expected, heap.get_ids());
    }

    #[test]
    fn pop() {
        let id = [0; 20].into();
        let addr = "127.0.0.1".parse().unwrap();
        let mut heap = build_heap(id, addr, 4);
        let expected = Some(Node::new(id.at_dist(1), addr, 1201));
        assert_eq!(expected, heap.pop());
    }
}