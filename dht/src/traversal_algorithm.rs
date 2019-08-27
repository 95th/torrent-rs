use std::net::SocketAddr;
use std::sync::Arc;

use crate::node::NodeId;
use crate::observer::Observer;

pub trait TraversalAlgorithm {
    fn traverse(&self, id: &NodeId, addr: &SocketAddr);

    fn done(&self);

    fn finished(&self, observer: Arc<Observer>);

    fn id(&self) -> &NodeId;
}
