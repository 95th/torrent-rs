use dht::node::NodeId;
use dht::state::DhtState;

fn main() {
    let mut state = DhtState::default();
    state.nids.push((
        "100.100.100.100".parse().unwrap(),
        NodeId::from_bytes(b"aaaaabbbbbcccccddddd"),
    ));

    let value = state.save();
    println!("{}", value);

    let state = DhtState::read(&value).unwrap();
    println!("{:?}", state);
}
