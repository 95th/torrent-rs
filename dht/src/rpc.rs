use crate::id::NodeId;

enum Op {
    FindNode(NodeId),
    Ping()
}