use crate::node::Node;
use crate::storage::ForgetfulStorage;

pub struct Server {
    ksize: usize,
    alpha: u8,
    node: Node,
    storage: ForgetfulStorage
}

impl Server {
    pub async fn listen(&self) {}
}