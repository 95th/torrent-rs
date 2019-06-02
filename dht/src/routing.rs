use std::collections::btree_map::BTreeMap;
use std::rc::Rc;
use std::time::Duration;
use std::time::Instant;

use crate::id::Id;
use crate::node::Node;
use crate::node::NodeHeap;

pub struct RoutingTable {
    node: Rc<Node>,
    ksize: usize,
    buckets: Vec<Bucket>,
}

impl RoutingTable {
    pub fn new(node: Rc<Node>, ksize: usize) -> RoutingTable {
        let initial_bucket =
            Bucket::new(Rc::new([0; 20].into()), Rc::new([0xFF; 20].into()), ksize);
        let buckets = vec![initial_bucket];
        RoutingTable {
            node,
            ksize,
            buckets,
        }
    }

    pub fn split_bucket(&mut self, index: usize) {
        let (left, right) = self.buckets[index].split();
        self.buckets[index] = left;
        self.buckets.insert(index, right);
    }

    pub fn stale_buckets(&self) -> Vec<&Bucket> {
        let hour_ago = Instant::now() - Duration::from_secs(3600);
        self.buckets
            .iter()
            .filter(|b| b.last_updated < hour_ago)
            .collect()
    }

    pub fn remove_contact(&mut self, node: &Node) {
        self.bucket_of_mut(node).remove_node(node);
    }

    pub fn is_new_node(&self, node: &Node) -> bool {
        self.bucket_of(node).is_new_node(node)
    }

    pub fn add_contact(&mut self, node: Rc<Node>) {
        let index = self.bucket_index_of(&node);
        let bucket = &mut self.buckets[index];
        if bucket.add_node(node.clone()) {
            return;
        }

        if bucket.has_in_range(&self.node) || bucket.depth() % 5 != 0 {
            self.split_bucket(index);
            self.add_contact(node);
        } else {
            // TODO ping the bucket head
        }
    }

    pub fn find_neighbours(
        &mut self,
        node: Rc<Node>,
        k: Option<usize>,
        exclude: Option<&Node>,
    ) -> Vec<Rc<Node>> {
        let k = k.unwrap_or(self.ksize);

        let mut heap = NodeHeap::new(node.clone(), k);
        for neighbour in TableIterator::new(self, node.clone()) {
            if let Some(exclude) = exclude {
                if neighbour.same_home_as(exclude) {
                    continue;
                }
            }

            if neighbour.id != node.id {
                heap.push(neighbour);
                if heap.len() == k {
                    break;
                }
            }
        }

        heap.closest()
    }

    fn bucket_index_of(&self, node: &Node) -> usize {
        self.buckets
            .iter()
            .enumerate()
            .find(|(_, b)| node.id < b.upper)
            .map(|(i, _)| i)
            .unwrap() // A node always has a bucket. So, unwrapping is OK.
    }

    fn bucket_of(&self, node: &Node) -> &Bucket {
        self.buckets.iter().find(|b| node.id < b.upper).unwrap() // A node always has a bucket. So, unwrapping is OK.
    }

    fn bucket_of_mut(&mut self, node: &Node) -> &mut Bucket {
        self.buckets.iter_mut().find(|b| node.id < b.upper).unwrap() // A node always has a bucket. So, unwrapping is OK.
    }
}

pub struct Bucket {
    pub(crate) lower: Rc<Id>,
    pub(crate) upper: Rc<Id>,
    nodes: BTreeMap<Rc<Id>, Rc<Node>>,
    extra_nodes: BTreeMap<Rc<Id>, Rc<Node>>,
    ksize: usize,
    last_updated: Instant,
}

impl Bucket {
    pub fn new(lower: Rc<Id>, upper: Rc<Id>, ksize: usize) -> Bucket {
        Bucket {
            lower,
            upper,
            nodes: BTreeMap::new(),
            extra_nodes: BTreeMap::new(),
            ksize,
            last_updated: Instant::now(),
        }
    }

    pub fn touch(&mut self) {
        self.last_updated = Instant::now();
    }

    pub fn get_nodes(&self) -> Vec<Rc<Node>> {
        self.nodes.values().cloned().collect()
    }

    pub fn split(&mut self) -> (Bucket, Bucket) {
        let distance = self.lower.dist_to(&self.upper);
        let middle = self.lower.at_dist(distance / 2);
        let mut left = Bucket::new(self.lower.clone(), middle.clone(), self.ksize);
        let mut right = Bucket::new(middle.at_dist(1), self.upper.clone(), self.ksize);

        let nodes = self.nodes.values().chain(self.extra_nodes.values());

        for node in nodes {
            if node.id <= middle {
                left.add_node(node.clone());
            } else {
                right.add_node(node.clone());
            }
        }

        (left, right)
    }

    pub fn add_node(&mut self, node: Rc<Node>) -> bool {
        if self.nodes.contains_key(&node.id) || self.nodes.len() < self.ksize {
            self.nodes.insert(node.id.clone(), node.clone());
        } else {
            self.extra_nodes.insert(node.id.clone(), node.clone());
            return false;
        }
        true
    }

    pub fn get_node(&self, id: &Id) -> Option<Rc<Node>> {
        self.nodes.get(id).cloned()
    }

    pub fn remove_node(&mut self, node: &Node) {
        self.extra_nodes.remove(&node.id);

        if self.nodes.contains_key(&node.id) {
            self.nodes.remove(&node.id);

            let key = match self.extra_nodes.keys().next() {
                Some(key) => key.clone(),
                None => return,
            };

            let value = self.extra_nodes.remove(&key).unwrap();
            self.nodes.insert(key, value);
        }
    }

    pub fn is_new_node(&self, node: &Node) -> bool {
        self.nodes.contains_key(&node.id)
    }

    pub fn has_in_range(&self, node: &Node) -> bool {
        node.id >= self.lower && node.id <= self.upper
    }

    pub fn head(&self) -> Option<Rc<Node>> {
        self.nodes.values().next().cloned()
    }

    pub fn depth(&self) -> usize {
        let origin = Id::from([0xFF; 20]);
        self.nodes
            .values()
            .map(|n| n.id.clone())
            .fold(0, |acc, id| origin.dist_to(&id).max(acc))
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

struct TableIterator<'t> {
    table: &'t mut RoutingTable,
    start: Rc<Node>,
    nodes: Vec<Rc<Node>>,
    node_index: usize,
    left_bucket_index: usize,
    right_bucket_index: usize,
    left: bool,
}

impl<'t> TableIterator<'t> {
    fn new(table: &'t mut RoutingTable, start: Rc<Node>) -> TableIterator<'t> {
        let bucket_index = table.bucket_index_of(&start);
        let nodes = table.buckets[bucket_index].get_nodes();
        TableIterator {
            table,
            start,
            nodes,
            node_index: 0,
            left_bucket_index: bucket_index,
            right_bucket_index: bucket_index,
            left: true,
        }
    }
}

impl Iterator for TableIterator<'_> {
    type Item = Rc<Node>;

    fn next(&mut self) -> Option<Rc<Node>> {
        loop {
            if let Some(v) = self.nodes.get(self.node_index) {
                self.node_index += 1;
                return Some(v.clone());
            }

            self.node_index = 0;
            if self.left && self.left_bucket_index > 0 {
                self.left_bucket_index -= 1;
                self.nodes = self.table.buckets[self.left_bucket_index].get_nodes();
                self.left = false;
            } else if self.right_bucket_index < self.table.buckets.len() - 1 {
                self.right_bucket_index += 1;
                self.nodes = self.table.buckets[self.right_bucket_index].get_nodes();
                self.left = true;
            } else {
                break;
            }
        }

        None
    }
}
