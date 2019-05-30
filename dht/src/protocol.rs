use crate::id::Id;
use crate::node::Node;
use crate::routing::RoutingTable;
use crate::storage::Storage;
use crypto::sha1::Sha1;
use crypto::digest::Digest;

pub struct Protocol {
    router: RoutingTable,
    storage: Storage,
    source_node: Node,
}

impl Protocol {
    pub fn new(source_node: Node, storage: Storage, ksize: usize) -> Protocol {
        Protocol {
            router: RoutingTable::new(source_node, ksize),
            source_node,
            storage,
        }
    }

    fn get_refresh_ids(&mut self) -> Vec<Id> {
        self.router
            .stale_buckets()
            .iter()
            .map(|bucket| Id::ranged_random(&bucket.range))
            .collect()
    }

    fn welcome_if_new(&mut self, node: &Node) {
        if !self.router.is_new_node(node) {
            return;
        }

        for (k, v) in self.storage.iter() {
            let node = Node::with_id(digest(k));
        }
    }
}

fn digest(text: &str) -> Id {
    let mut hasher = Sha1::new();
    hasher.input_str(text);
    let mut buf = [0u8; 20];
    hasher.result(&mut buf);
    buf.into()
}