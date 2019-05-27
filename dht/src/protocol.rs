use crate::id::Id;
use crate::node::Node;
use crate::routing::RoutingTable;
use crate::storage::Storage;

pub struct Protocol {
    router: RoutingTable,
    storage: Storage,
    source_node: Node
}

impl Protocol {
    pub fn new(source_node: Node, storage: Storage, ksize: usize) -> Protocol {
        Protocol {
            router: RoutingTable::new(source_node, ksize),
            source_node,
            storage
        }
    }

    fn get_refresh_ids(&mut self) -> Vec<Id> {
        self.router
            .stale_buckets()
            .iter()
            .map(|bucket| Id::ranged_random(&bucket.range))
            .collect()
    }
}
