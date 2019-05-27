use crate::node::Node;
use crate::storage::Storage;

pub struct Server {
    ksize: usize,
    alpha: u8,
    node: Node,
    storage: Storage
}