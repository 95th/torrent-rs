use crate::id::Id;
use crate::node::Node;
use crate::routing::RoutingTable;
use crate::storage::Storage;
use crypto::digest::Digest;
use crypto::sha1::Sha1;
use std::rc::Rc;

pub struct Protocol {
    router: RoutingTable,
    storage: Storage,
    source_node: Rc<Node>,
}

impl Protocol {
    pub fn new(source_node: Rc<Node>, storage: Storage, ksize: usize) -> Protocol {
        Protocol {
            router: RoutingTable::new(source_node.clone(), ksize),
            source_node,
            storage,
        }
    }

    pub fn get_refresh_ids(&mut self) -> Vec<Id> {
        self.router
            .stale_buckets()
            .iter()
            .map(|bucket| Id::ranged_random(&bucket.lower, &bucket.upper))
            .collect()
    }

    pub fn welcome_if_new(&mut self, node: Rc<Node>) {
        if !self.router.is_new_node(&node) {
            return;
        }

        for (k, _) in self.storage.iter() {
            let key_node = Rc::new(Node::with_id(Rc::new(digest(k))));
            let neighbours = self.router.find_neighbours(key_node.clone(), None, None);

            if neighbours.is_empty() {
                // TODO: call rpc store
            } else {
                let last = neighbours.last().unwrap().dist_to(&key_node);
                let new_close = node.dist_to(&key_node) < last;
                let first = neighbours.first().unwrap().dist_to(&key_node);
                let this_closest = self.source_node.dist_to(&key_node) < first;

                if new_close && this_closest {
                    // TODO: call rpc store
                }
            }
        }
        self.router.add_contact(node);
    }
}

fn digest(text: &str) -> Id {
    let mut sha1 = Sha1::new();
    sha1.input_str(text);
    let mut buf = [0u8; 20];
    sha1.result(&mut buf);
    buf.into()
}
