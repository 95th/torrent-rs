use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::collections::HashSet;
use std::mem;
use std::net::IpAddr;
use std::net::SocketAddr;
use std::rc::Rc;

use crate::id::Id;

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct Node {
    pub(crate) id: Rc<Id>,
    addr: Option<(IpAddr, u16)>,
}

impl Node {
    pub fn new(id: Rc<Id>, addr: IpAddr, port: u16) -> Node {
        Node {
            id,
            addr: Some((addr, port)),
        }
    }

    pub fn with_id(id: Rc<Id>) -> Node {
        Node { id, addr: None }
    }

    pub fn same_home_as(&self, other: &Node) -> bool {
        self.addr == other.addr
    }

    pub fn dist_to(&self, other: &Node) -> usize {
        self.id.dist_to(&other.id)
    }

    pub fn socket_addr(&self) -> Option<SocketAddr> {
        match self.addr {
            Some((addr, port)) => Some(SocketAddr::new(addr, port)),
            None => None,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct NodeHeapItem {
    dist: usize,
    node: Rc<Node>,
}

impl Ord for NodeHeapItem {
    fn cmp(&self, other: &NodeHeapItem) -> Ordering {
        other
            .dist
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
    heap: BinaryHeap<NodeHeapItem>,
    node: Rc<Node>,
    contacted: HashSet<Rc<Id>>,
    max_size: usize,
}

impl NodeHeap {
    pub fn new(node: Rc<Node>, max_size: usize) -> NodeHeap {
        NodeHeap {
            node,
            heap: BinaryHeap::new(),
            contacted: HashSet::new(),
            max_size,
        }
    }

    pub fn remove_all(&mut self, nodes: &[Node]) {
        let peers: HashSet<&Node> = nodes.iter().collect();
        let old_heap = mem::replace(&mut self.heap, BinaryHeap::new());
        self.heap = old_heap
            .into_iter()
            .filter(|item| !peers.contains(item.node.as_ref()))
            .collect();
    }

    pub fn get_node(&self, id: &Id) -> Option<Rc<Node>> {
        self.heap
            .iter()
            .find(|item| item.node.id.as_ref() == id)
            .map(|item| item.node.clone())
    }

    pub fn push(&mut self, node: Rc<Node>) {
        if !self.contains(&node) {
            let dist = self.node.dist_to(&node);
            self.heap.push(NodeHeapItem { dist, node });
        }
    }

    pub fn push_all(&mut self, nodes: &[Rc<Node>]) {
        for node in nodes {
            if !self.contains(node) {
                let dist = self.node.dist_to(node);
                self.heap.push(NodeHeapItem {
                    dist,
                    node: node.clone(),
                });
            }
        }
    }

    pub fn pop(&mut self) -> Option<Rc<Node>> {
        self.heap.pop().map(|item| item.node.clone())
    }

    pub fn contains(&self, node: &Node) -> bool {
        self.heap.iter().any(|item| item.node.as_ref() == node)
    }

    pub fn mark_contacted(&mut self, node: &Node) {
        self.contacted.insert(node.id.clone());
    }

    pub fn closest(&self) -> Vec<Rc<Node>> {
        let mut items = vec![];
        let mut heap = self.heap.clone();
        while let Some(item) = heap.pop() {
            if items.len() < self.max_size {
                items.push(item.node.clone());
            }
        }
        items
    }

    pub fn get_uncontacted(&self) -> Vec<Rc<Node>> {
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

    pub fn get_ids(&self) -> Vec<Rc<Id>> {
        self.closest().iter().map(|node| node.id.clone()).collect()
    }

    pub fn len(&self) -> usize {
        self.heap.len().min(self.max_size)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
